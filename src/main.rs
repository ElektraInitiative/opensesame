// opensesame

mod bat;
mod buttons;
mod clima_sensor_us;
mod config;
mod environment;
mod garage;
mod mod_ir_temp;
mod nextcloud;
mod pwr;
mod sensors;
mod ssh;
mod validator;
mod watchdog;

use mlx9061x::Error as MlxError;
use std::io::{Error,ErrorKind};
use std::panic;
use std::sync::Arc;
use std::{thread, time};

use gettextrs::*;

use sunrise::sunrise_sunset;
use systemstat::Duration;

use chrono::prelude::*;
use buttons::Buttons;
use buttons::StateChange;
use sensors::{Sensors, SensorsChange};
use clima_sensor_us::{ClimaSensorUS, TempWarningStateChange};
use config::Config;
use environment::AirQualityChange;
use environment::Environment;
use garage::Garage;
use garage::GarageChange;
use mod_ir_temp::{IrTempStateChange, ModIR};
use nextcloud::Nextcloud;
use pwr::Pwr;
use ssh::exec_ssh_command;
use validator::Validation;
use validator::Validator;
use watchdog::Watchdog;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{interval, sleep};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, self};

const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";
const STATE_PARENT: &'static str = "/state/libelektra/opensesame/#0/current";

// play audio file with argument. If you do not have an argument, simply pass --quiet again
fn play_audio_file(file: String, arg: String) -> Result<(), Error> {
	if file != "/dev/null" {
		thread::Builder::new()
			.name("ogg123".to_string())
			.spawn(move || {
				std::process::Command::new("ogg123")
					.arg("--quiet")
					.arg(arg)
					.arg(file)
					.status()
					.expect(&gettext("failed to execute process"));
			})?;
	}
	Ok(())
}

fn do_reset(watchdog: &mut Watchdog, nextcloud_sender: Sender<NextcloudEvent>, pwr: &mut Pwr) {
	if pwr.enabled() {
		watchdog.trigger();
		pwr.switch(false);
		nextcloud_sender.send(NextcloudEvent::Ping(gettext("👋 Turned PWR_SWITCH off")));
		watchdog.trigger();
		thread::sleep(time::Duration::from_millis(watchdog::SAFE_TIMEOUT));

		watchdog.trigger();
		pwr.switch(true);
		nextcloud_sender.send(NextcloudEvent::Ping(gettext("👋 Turned PWR_SWITCH on")));
		watchdog.trigger();
		thread::sleep(time::Duration::from_millis(watchdog::SAFE_TIMEOUT));

		watchdog.trigger();
	}
}

enum NextcloudEvent {
	Ping(String),
	Licht(String),
	Chat(String),
	SetStatusOnline(String),
	SetStatusEnv(String),
	SetStatusDoor(String),
}

async fn nextcloud_loop(
	mut nextcloud: Nextcloud,
	command_sender: Sender<CommandToButtons>,
	mut nextcloud_receiver: Receiver<NextcloudEvent>,
) {
	while let Some(event) = nextcloud_receiver.recv().await {
		match event {
			NextcloudEvent::Chat(message) => nextcloud.send_message(message).await,
			NextcloudEvent::Ping(message) => nextcloud.ping(message).await,
			NextcloudEvent::Licht(message) => nextcloud.licht(message).await,
			NextcloudEvent::SetStatusOnline(message) => nextcloud.set_info_online(message).await,
			NextcloudEvent::SetStatusEnv(message) => nextcloud.set_info_environment(message).await,
			NextcloudEvent::SetStatusDoor(message) => nextcloud.set_info_door(message).await,
		}
	}
	// TODO: use try_recv and listen to chat commands
	// Or should we use a seperate long running thread
	// to receive commands?
}
/// This function could be triggered by state changes on GPIO, because the pins are connected with the olimex board
/// So we dont need to run it all few seconds.
async fn garage_loop(
	mut garage: Garage,
	command_sender: Sender<CommandToButtons>,
	nextcloud_sender: Sender<NextcloudEvent>,
) {
	match garage.handle() {
		GarageChange::None => (),
		GarageChange::PressedTasterEingangOben => {
			// muss in buttons implementiert werden, damit button dann an nextcloud weiter gibt!

			/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
				"💡 Pressed at entrance top switch. Switch lights in garage. {}",
				buttons.switch_lights(true, false)
			)));*/
			command_sender.send(CommandToButtons::SwitchLights(true, false)).await;
		}
		GarageChange::PressedTasterTorOben => {
			/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
				"💡 Pressed top switch at garage door. Switch lights in and out garage. {}",
				buttons.switch_lights(true, true)
			)));*/
			command_sender.send(CommandToButtons::SwitchLights(true, true)).await;
		}
		GarageChange::PressedTasterEingangUnten | GarageChange::PressedTasterTorUnten => {
			//buttons.open_door();
			command_sender.send(CommandToButtons::OpenDoor).await;
		}

		GarageChange::ReachedTorEndposition => {
			nextcloud_sender.send(NextcloudEvent::SetStatusDoor(String::from("🔒 Open"))).await;
			nextcloud_sender.send(NextcloudEvent::Chat(String::from("🔒 Garage door closed."))).await;
		}
		GarageChange::LeftTorEndposition => {
			nextcloud_sender.send(NextcloudEvent::SetStatusDoor(String::from("🔓 Closed"))).await;
			nextcloud_sender.send(NextcloudEvent::Chat(String::from("🔓 Garage door open"))).await;
		}
	}
}

async fn sensors_loop(nextcloud_sender: Sender<NextcloudEvent>) -> Result<(), Error>{
	let device_file = File::open(device_path).await?;
	let reader = BufReader::new(device_file);

	let mut lines = reader.lines();
	while let Some(line) = lines.next_line().await? {
		match sensors.update(line) {
			SensorsChange::None => (),
			SensorsChange::Alarm(w) => {
				nextcloud_sender.send(NextcloudEvent::Chat(gettext!("Fire Alarm {}", w)));
				/*
				state.set("alarm/fire", &w.to_string());
				sighup.store(true, Ordering::Relaxed);
				exec_ssh_command(format!("kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"", w));
				*/
			}
			SensorsChange::Chat(w) => {
				nextcloud_sender.send(NextcloudEvent::Chat(gettext!("Fire Chat {}", w)));
			}
		}
	}
	Ok(())
}

async fn modir_loop(nextcloud_sender: Sender<NextcloudEvent>) {
	let mut config = Config::new(CONFIG_PARENT);
	let mut interval = interval(Duration::from_secs(config.get::<u64>("ir/data/interval")));

	match ModIR::new(&mut config).as_mut() {
		Ok(ir_temp) => 
		loop {
			match ir_temp.handle() {
				Ok(state) => match state {
					IrTempStateChange::None => (),
					IrTempStateChange::ChanedToBothToHot => {
						nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
							"🌡️🌡️ ModIR both sensors too hot! Ambient: {} °C, Object: {} °C",
							ir_temp.ambient_temp,
							ir_temp.object_temp
						)));
					}
					IrTempStateChange::ChangedToAmbientToHot => {
						nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
							"🌡️ ModIR ambient sensors too hot! Ambient: {} °C",
							ir_temp.ambient_temp
						)));
					}
					IrTempStateChange::ChangedToObjectToHot => {
						nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
							"🌡️ ModIR object sensors too hot! Object: {} °C",
							ir_temp.object_temp
						)));
					}
					IrTempStateChange::ChangedToCancelled => {
						nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
							"🌡 ModIR cancelled warning! Ambient: {} °C, Object: {} °C",
							ir_temp.ambient_temp,
							ir_temp.object_temp
						)));
					}
				},
				Err(error_typ) => match error_typ {
					MlxError::I2C(error) => {
						nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							error
						)));
					}
					MlxError::ChecksumMismatch => {
						nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							"ChecksumMismatch"
						)));
					}
					MlxError::InvalidInputData => {
						nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							"InvalidInputData"
						)));
					}
				},
			}
			interval.tick().await;
		},
		Err(error_typ) => {
			match error_typ {
				MlxError::I2C(error) => {
					nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init ModIR: {}",
						error
					)));
				}
				MlxError::ChecksumMismatch => {
					nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init ModIR: {}",
						"ChecksumMismatch"
					)));
				}
				MlxError::InvalidInputData => {
					nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init ModIR: {}",
						"InvalidInputData"
					)));
				}
			};
		}
	}
}

// morgen nochmal überarbeiten; felx
async fn env_loop(
	nextcloud_sender: Sender<NextcloudEvent>,
	command_sender: Sender<CommandToButtons>,
) -> Result<(), Error> {
	let mut config = Config::new(CONFIG_PARENT);
	let mut interval = interval(Duration::from_secs(config.get::<u64>("environment/data/interval")));
	let mut enabled = true;

	let mut environment = Environment::new(&mut config);
	if environment.board5a.is_some() {
		sleep(Duration::from_secs(1)).await;
		environment.init_ccs811();
	}

	while enabled {
		/* From buttons_loop
		if environment.handle() {
			if handle_environment(
				&mut environment,
				&mut nextcloud,
				Some(&mut buttons),
				&mut config,
			)? {
				state.set("alarm/fire", &environment.name);
				sighup.store(true, Ordering::Relaxed);
			}
		}*/
		if environment.handle() {
			nextcloud_sender.send(NextcloudEvent::SetStatusEnv(format!(
				"💨 {:?}",
				environment.air_quality
			)));
			match environment.air_quality {
				AirQualityChange::Error => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"⚠️ Error {:#02b} reading environment! Status: {:#02b}. {}",
						environment.error,
						environment.status,
						environment.to_string()
					)));
					enabled = false;
				}
				AirQualityChange::Ok => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"💨 Airquality is ok. {}",
						environment.to_string()
					)));
				}
				AirQualityChange::Moderate => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"💩 Airquality is moderate. {}",
						environment.to_string()
					)));
				}
				AirQualityChange::Bad => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"💩 Airquality is bad! {}",
						environment.to_string()
					)));
				}

				AirQualityChange::FireAlarm => {
					() //wofür ist dieser return value? bzw. was sollte er im alten bewirken??
				}
				AirQualityChange::FireBell => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🚨 Possible fire alarm! Ring bell once! ⏰. {}",
						environment.to_string()
					)));

					// buttons.ring_bell(20, 0); where is it called, and how does it increment the counter???
					command_sender.try_send(CommandToButtons::RingBell(20, 0));
					/*if config.get_bool("garage/enable") {
						let file = config.get::<String>("audio/alarm");
						let arg = "--quiet".to_string();
						//play_audio_file(config.get::<String>("audio/alarm"), "--quiet".to_string())?;
						if file != "/dev/null" {
							thread::Builder::new()
								.name("ogg123".to_string())
								.spawn(move || {
									std::process::Command::new("ogg123")
										.arg("--quiet")
										.arg(arg)
										.arg(file)
										.status()
										.expect(&gettext("failed to execute process"));
								})?;
						}
						// build thread in other thread ??? maybe ssh in external thread/task?
						thread::Builder::new()
							.name("killall to ring bell".to_string())
							.spawn(move || {
								exec_ssh_command("killall -SIGUSR2 opensesame".to_string());
							})?;
					}*/
				}
				AirQualityChange::FireChat => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🚨 Possible fire alarm! (don't ring yet). {}",
						environment.to_string()
					)));
				}
			}
		};
		interval.tick().await;
	}
	return Ok(());
}

async fn weatherstation_loop(nextcloud_sender: Sender<NextcloudEvent>) {
	let mut config = Config::new(CONFIG_PARENT);
	let mut interval = interval(Duration::from_secs(config.get::<u64>("weatherstation/data/interval")));

	match ClimaSensorUS::new(&mut config).as_mut() {
		Ok(weath_st) => loop {
			match weath_st.handle().await {
				Ok(TempWarningStateChange::ChangeToCloseWindow) => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🌡️ Temperature above {} °C, close the window",
						ClimaSensorUS::CLOSE_WINDOW_TEMP
					)));
				}
				Ok(TempWarningStateChange::ChangeToWarningTempNoWind) => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🌡️ Temperature above {} °C and no Wind",
						ClimaSensorUS::NO_WIND_TEMP
					)));
				}
				Ok(TempWarningStateChange::ChangeToWarningTemp) => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🌡️ Temperature above {} °C",
						ClimaSensorUS::WARNING_TEMP
					)));
				}
				Ok(TempWarningStateChange::ChangeToRemoveWarning) => {
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"🌡 Temperature again under {} °C, warning was removed",
						ClimaSensorUS::CANCLE_TEMP
					)));
				}
				Ok(TempWarningStateChange::None) => (),
				Err(error) => {
					nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Error from weather station: {}",
						error.to_string()
					)));
				}
			}
			interval.tick().await;
		},
		Err(error) => {
			nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
				"⚠️ Failed to init libmodbus connection: {}",
				error
			)));
		}
	}
}

async fn bat_loop(nextcloud_sender: Sender<NextcloudEvent>) {}
async fn button_loop(
	mut buttons: Buttons,
	mut validator: Validator,
	time_format: &str,
	startup_time: &str,
	command_receiver: Receiver<CommandToButtons>,
	nextcloud_sender: Sender<NextcloudEvent>,
) -> Result<(), Error> {
	let mut interval = interval(Duration::from_millis(10));
	let enabled = true;
	while enabled{
		let changes = buttons.handle();
		match changes {
			Ok(StateChange::Pressed(button)) => {
				match button {
					buttons::BUTTON_BELL => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							buttons.ring_bell(2, 5);
							if config.get_bool("garage/enable") {
								play_audio_file(
									config.get::<String>("audio/bell"),
									"--quiet".to_string(),
								)?;
								thread::Builder::new()
									.name("killall to ring bell".to_string())
									.spawn(move || {
										exec_ssh_command("killall -SIGUSR2 opensesame".to_string());
									})
									.unwrap();
							}
							nextcloud_sender.send(NextcloudEvent::Chat(gettext(
								"🔔 Pressed button bell.",
							)));
						} else {
							buttons.show_wrong_input();
							nextcloud_sender.send(NextcloudEvent::Chat(gettext!("🔕 Did not ring bell (button was pressed) because the time 🌜 is {}, {}", now.format(&time_format))));
						}
					}
					buttons::TASTER_INNEN => {
						nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
							"💡 Pressed switch inside. {}.",
							buttons.switch_lights(true, true)
						)));
					}
					buttons::TASTER_AUSSEN => {
						nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
							"💡 Pressed switch outside or light button. {}.",
							buttons.switch_lights(false, true),
						)));
					}
					buttons::TASTER_GLOCKE => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							buttons.ring_bell(5, 5);
							nextcloud_sender.send(NextcloudEvent::Chat(gettext(
								"🔔 Pressed switch bell."
							)));
						} else {
							buttons.show_wrong_input();
							nextcloud_sender.send(NextcloudEvent::Chat(gettext!("🔕 Did not ring bell (taster outside) because the time 🌜 is {}, {}", now.format(&time_format))));
						}
					}
					_ => panic!("🔘 Pressed {}", button),
				}
			}
			Ok(StateChange::Released(_button)) => (),
			Ok(StateChange::LightsOff) => {
				nextcloud_sender.send(NextcloudEvent::Licht(gettext(
				"🕶️ Light was turned off."
			)));
			},
			Ok(StateChange::None) => (),
			Ok(_) => (),
			Err(error) => {
				return Err(Error::new(ErrorKind::ConnectionRefused, error));
			}
		}
		// Button end

		// Validation benötigt button, somit threads abhängig!!!; channel zwischen buttons und validator? damit validator nur getriggert ist wenn buttons sich ändert?
		// Validation start
		let sequence = buttons.sequence.to_vec();
		match validator.validate(&mut buttons.sequence) {
			Validation::Validated(user) => {
				buttons.open_door();
				nextcloud_sender.send(NextcloudEvent::Chat(gettext!("🤗 Opened for {}", user)));
				let now = Local::now();
				let (sunrise, sunset) = sunrise_sunset(
					config.get::<f64>("location/latitude"),
					config.get::<f64>("location/longitude"),
					now.year(),
					now.month(),
					now.day(),
				);
				if now.timestamp() < sunrise || now.timestamp() > sunset {
					nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
						"💡 Switch lights in and out. {}",
						buttons.switch_lights(true, true)
					)));
				} else {
					nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
						"🕶️ Don't switch lights as its day. Now: {} Sunrise: {} Sunset: {}",
						now.timestamp(),
						sunrise,
						sunset
					)));
				}
			}
			Validation::Timeout => {
				if sequence != vec![0, 15] {
					buttons.show_wrong_input();
					buttons.ring_bell(20, 0);
					nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
						"⌛ Timeout with sequence {}",
						format!("{:?}", sequence)
					)));
				}
			}
			Validation::SequenceTooLong => {
				buttons.show_wrong_input();
				buttons.ring_bell(20, 0);
				nextcloud_sender.send(NextcloudEvent::Chat(gettext!(
					"⌛ Sequence {} too long",
					format!("{:?}", sequence)
				)));
			}
			Validation::None => (),
		}
		interval.tick().await;
	}
	Ok(())

	/*Validation end

	remember_baseline_counter += 1;
	if remember_baseline_counter == wait_for_remember_baseline {
		environment.remember_baseline(&mut state);
		remember_baseline_counter = 0;
	}

	thread::sleep(time::Duration::from_millis(10));
	*/
}
/* 
	environment.remember_baseline(&mut state);
	nextcloud.set_info_online(gettext("📴 OFF"));
	nextcloud_sender.send(NextcloudEvent::Ping(gettext!(
		"👋 opensesame {} bye-bye {}",
		env!("CARGO_PKG_VERSION"),
		Local::now().format(&date_time_format).to_string()
	)));
	Ok(())
}*/

enum CommandToButtons {
	OpenDoor,
	TurnOnLight,
	RingBell(u16, u16),// maybe implement it with interval
	SwitchLights(bool, bool) // This also need to implement the sending of a Message to nextcloud, which is now in Garage
	                    // TODO Add more
}

#[tokio::main]
async fn main() -> io::Result<()> {
	let config = Config::new(CONFIG_PARENT);
	let state = Config::new(STATE_PARENT);

	let date_time_format = config.get::<String>("nextcloud/format/datetime");
	let startup_time = &format!("{}", Local::now().format(&date_time_format));

	// Sender and receiver to open doors/lights etc via Nextcloud
	let (command_sender, mut command_receiver) = mpsc::channel::<CommandToButtons>(32);
	// Info to send to next cloud
	let (nextcloud_sender, mut nextcloud_receiver) = mpsc::channel::<NextcloudEvent>(32);

	let buttons_enabled = config.get_bool("buttons/enable");
	let garage_enabled = config.get_bool("garage/enable");
	let sensors_enabled = config.get_bool("sensors/enable");
	let modir_enabled = config.get_bool("ir/enable");
	let env_enabled = config.get_bool("environment/enable");
	let weatherstation_enabled = config.get_bool("weatherstation/enable");
	let bat_enabled = config.get_bool("bat/enable");
	let watchdog_enabled = config.get_bool("watchdog/enable");

	let mut nextcloud = Nextcloud::new(&config);
	tokio::spawn(nextcloud_loop(nextcloud, command_sender.clone(), nextcloud_receiver));

	if garage_enabled {
		if !buttons_enabled {
			panic!("Garage depends on buttons!");
		}
		let garage = Garage::new(&config);
		tokio::spawn(garage_loop(
			garage,
			command_sender.clone(),
			nextcloud_sender.clone(),
		));
	}

	if buttons_enabled {
		let time_format = config.get::<String>("nextcloud/format/time");
		let mut buttons = Buttons::new(&config);
		let mut validator = Validator::new(&config);
		tokio::spawn(button_loop(
			buttons, validator, &time_format,
			&startup_time,
			command_receiver,
			nextcloud_sender.clone(),
		));
	}

	if sensors_enabled {
		let mut sensors = Sensors::new(&config);
		let device_path = config.get::<String>("sensors/device");
		tokio::spawn(sensors_loop(nextcloud_sender.clone()));
	}

	if modir_enabled {
		tokio::spawn(modir_loop(nextcloud_sender.clone()));
	}

	if env_enabled {
		tokio::spawn(env_loop(nextcloud_sender.clone(), command_sender.clone()));
	}

	if weatherstation_enabled {
		tokio::spawn(weatherstation_loop(nextcloud_sender.clone()));
	}

	if bat_enabled {
		tokio::spawn(bat_loop(nextcloud_sender.clone()));
	}
	Ok(())
}
