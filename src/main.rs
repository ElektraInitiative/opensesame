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
mod types;
mod validator;
mod watchdog;

use futures::future::join_all;
use mlx9061x::Error as MlxError;
use reqwest::Url;
use std::io::{Error, ErrorKind};
use std::panic;
use std::{thread, time};

use gettextrs::*;

use sunrise::sunrise_sunset;
use systemstat::Duration;

use buttons::Buttons;
use buttons::StateChange;
use chrono::prelude::*;
use clima_sensor_us::{ClimaSensorUS, TempWarningStateChange};
use config::Config;
use environment::AirQualityChange;
use environment::Environment;
use garage::Garage;
use garage::GarageChange;
use mod_ir_temp::{IrTempStateChange, ModIR};
use nextcloud::Nextcloud;
use pwr::Pwr;
use sensors::{Sensors, SensorsChange};
use ssh::exec_ssh_command;
use validator::Validation;
use validator::Validator;
use watchdog::Watchdog;
use types::OpensesameError;

use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{interval, sleep, Interval};
use tokio::process::Command;

const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";
const STATE_PARENT: &'static str = "/state/libelektra/opensesame/#0/current";

// play audio file with argument. If you do not have an argument, simply pass --quiet again
async fn play_audio_file(file: String, arg: String) -> Result<(), OpensesameError> {
	if file != "/dev/null" {
		Command::new("ogg123")
			.arg("--quiet")
			.arg(arg)
			.arg(file)
			.status().await.unwrap();
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

async fn nextcloud_chat_loop(
	mut nextcloud: Nextcloud,
	command_sender: Sender<CommandToButtons>,
	mut nextcloud_receiver: Receiver<NextcloudEvent>,
) -> Result<(), OpensesameError> {
	nextcloud
		.send_message(String::from("Nextcloud stated..."))
		.await;
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
	Ok(())
	// TODO: use try_recv and listen to chat commands
	// Or should we use a seperate long running thread
	// to receive commands?
}

async fn nextcloud_commands_loop() -> Result<(), OpensesameError> {
	let url =
		Url::parse("wss://your-nextcloud-instance.com/ocs/v2.php/apps/spreed/api/v1/chat/user");
	// while let Some(event) = nextcloud_receiver.recv().await {
	// 	match event {
	// 		NextcloudEvent::Chat(message) => nextcloud.send_message(message).await,
	// 		NextcloudEvent::Ping(message) => nextcloud.ping(message).await,
	// 		NextcloudEvent::Licht(message) => nextcloud.licht(message).await,
	// 		NextcloudEvent::SetStatusOnline(message) => nextcloud.set_info_online(message).await,
	// 		NextcloudEvent::SetStatusEnv(message) => nextcloud.set_info_environment(message).await,
	// 		NextcloudEvent::SetStatusDoor(message) => nextcloud.set_info_door(message).await,
	// 	}
	// }
	Ok(())
}
/// This function could be triggered by state changes on GPIO, because the pins are connected with the olimex board
/// So we dont need to run it all few seconds.
async fn garage_loop(
	mut garage: Garage,
	command_sender: Sender<CommandToButtons>,
	nextcloud_sender: Sender<NextcloudEvent>,
) -> Result<(), OpensesameError> {
	match garage.handle() {
		GarageChange::None => (),
		GarageChange::PressedTasterEingangOben => {
			// muss in buttons implementiert werden, damit button dann an nextcloud weiter gibt!

			/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
				"💡 Pressed at entrance top switch. Switch lights in garage. {}",
				buttons.switch_lights(true, false)
			)));*/
			command_sender
				.send(CommandToButtons::SwitchLights(true, false))
				.await?;
		}
		GarageChange::PressedTasterTorOben => {
			/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
				"💡 Pressed top switch at garage door. Switch lights in and out garage. {}",
				buttons.switch_lights(true, true)
			)));*/
			command_sender
				.send(CommandToButtons::SwitchLights(true, true))
				.await?;
		}
		GarageChange::PressedTasterEingangUnten | GarageChange::PressedTasterTorUnten => {
			//buttons.open_door();
			command_sender.send(CommandToButtons::OpenDoor).await?;
		}

		GarageChange::ReachedTorEndposition => {
			nextcloud_sender
				.send(NextcloudEvent::SetStatusDoor(String::from("🔒 Open")))
				.await?;
			nextcloud_sender
				.send(NextcloudEvent::Chat(String::from("🔒 Garage door closed.")))
				.await?;
		}
		GarageChange::LeftTorEndposition => {
			nextcloud_sender
				.send(NextcloudEvent::SetStatusDoor(String::from("🔓 Closed")))
				.await?;
			nextcloud_sender
				.send(NextcloudEvent::Chat(String::from("🔓 Garage door open")))
				.await?;
		}
		interval.tick().await;
	}
	Ok(())
}

async fn sensors_loop(
	mut sensors: Sensors,
	device_path: String,
	nextcloud_sender: Sender<NextcloudEvent>,
) -> Result<(), OpensesameError> {
	let device_file = File::open(device_path).await.expect("error here");
	let reader = BufReader::new(device_file);
	println!("In sensor loop");

	let mut lines = reader.lines();
	while let Some(line) = lines.next_line().await? {
		match sensors.update(line.clone()) {
			SensorsChange::None => {println!("None - Sensors - {}", line);()},
			SensorsChange::Alarm(w) => {
				nextcloud_sender
					.send(NextcloudEvent::Chat(gettext!("Fire Alarm {}", w)))
					.await?;
				println!("Alarm - Sensors - {}", line);
				/*
				state.set("alarm/fire", &w.to_string());
				sighup.store(true, Ordering::Relaxed);
				exec_ssh_command(format!("kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"", w));
				*/
			}
			SensorsChange::Chat(w) => {
				nextcloud_sender
					.send(NextcloudEvent::Chat(gettext!("Fire Chat {}", w)))
					.await?;
				println!("Chat - Sensors - {}", line);
			}
		}
	}
	Ok(())
}

async fn modir_loop(
	mut ir_temp: ModIR,
	mut interval: Interval,
	nextcloud_sender: Sender<NextcloudEvent>,
) -> Result<(), OpensesameError> {
	loop {
		match ir_temp.handle() {
			Ok(state) => match state {
				IrTempStateChange::None => (),
				IrTempStateChange::ChanedToBothToHot => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"🌡️🌡️ ModIR both sensors too hot! Ambient: {} °C, Object: {} °C",
							ir_temp.ambient_temp,
							ir_temp.object_temp
						)))
						.await?;
				}
				IrTempStateChange::ChangedToAmbientToHot => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"🌡️ ModIR ambient sensors too hot! Ambient: {} °C",
							ir_temp.ambient_temp
						)))
						.await?;
				}
				IrTempStateChange::ChangedToObjectToHot => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"🌡️ ModIR object sensors too hot! Object: {} °C",
							ir_temp.object_temp
						)))
						.await?;
				}
				IrTempStateChange::ChangedToCancelled => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"🌡 ModIR cancelled warning! Ambient: {} °C, Object: {} °C",
							ir_temp.ambient_temp,
							ir_temp.object_temp
						)))
						.await?;
				}
			},
			Err(error_typ) => match error_typ {
				MlxError::I2C(error) => {
					nextcloud_sender
						.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							error
						)))
						.await?;
				}
				MlxError::ChecksumMismatch => {
					nextcloud_sender
						.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							"ChecksumMismatch"
						)))
						.await?;
				}
				MlxError::InvalidInputData => {
					nextcloud_sender
						.send(NextcloudEvent::Ping(gettext!(
							"⚠️ Error while handling ModIR: {}",
							"InvalidInputData"
						)))
						.await?;
				}
			},
		}
		interval.tick().await;
	}
	Ok(())
}

// morgen nochmal überarbeiten; felx
async fn env_loop(
	mut environment: Environment,
	mut interval: Interval,
	nextcloud_sender: Sender<NextcloudEvent>,
	command_sender: Sender<CommandToButtons>,
) -> Result<(), Error> {
	let mut old_airquality = AirQualityChange::Error;
	if environment.board5a.is_some() {
		sleep(Duration::from_secs(1)).await;
		environment.init_ccs811();
	}

	loop {
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
			println!("Testing air quality new: {:?} - old: {:?}",environment.air_quality, old_airquality);
			if environment.air_quality != old_airquality {
				old_airquality = environment.air_quality;
				nextcloud_sender.send(NextcloudEvent::SetStatusEnv(format!("💨 {:?}",environment.air_quality))).await;

				match environment.air_quality {
					AirQualityChange::Error => {
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"⚠️ Error {:#02b} reading environment! Status: {:#02b}. {}",
								environment.error,
								environment.status,
								environment
							)))
							.await;
						break;
					}
					AirQualityChange::Ok => {
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"💨 Airquality is ok. {}",
								environment
							)))
							.await;
					}
					AirQualityChange::Moderate => {
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"💩 Airquality is moderate. {}",
								environment
							)))
							.await;
					}
					AirQualityChange::Bad => {
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"💩 Airquality is bad! {}",
								environment
							)))
							.await;
					}

					AirQualityChange::FireAlarm => {
						() //wofür ist dieser return value? bzw. was sollte er im alten bewirken??
					}
					AirQualityChange::FireBell => {
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"🚨 Possible fire alarm! Ring bell once! ⏰. {}",
								environment
							)))
							.await;

						// buttons.ring_bell(20, 0); where is it called, and how does it increment the counter???
						command_sender.send(CommandToButtons::RingBell(20, 0)).await;
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
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"🚨 Possible fire alarm! (don't ring yet). {}",
								environment.to_string()
							)))
							.await;
					}
				};
			}
		}
		interval.tick().await;
	}
	return Ok(());
}

async fn weatherstation_loop(
	mut clima_sensor: ClimaSensorUS,
	mut interval: Interval,
	nextcloud_sender: Sender<NextcloudEvent>,
) -> Result<(), OpensesameError> {
	loop {
		let f = clima_sensor.handle();
		match f.await {
			Ok(Some(temp_warning)) => {
				let message = match temp_warning {
					TempWarningStateChange::ChangeToCloseWindow => gettext!(
						"🌡️ Temperature above {} °C, close the window",
						ClimaSensorUS::CLOSE_WINDOW_TEMP
					),
					TempWarningStateChange::ChangeToWarningTempNoWind => gettext!(
						"🌡️ Temperature above {} °C and no Wind",
						ClimaSensorUS::NO_WIND_TEMP
					),
					TempWarningStateChange::ChangeToWarningTemp => {
						gettext!("🌡️ Temperature above {} °C", ClimaSensorUS::WARNING_TEMP)
					}
					TempWarningStateChange::ChangeToRemoveWarning => gettext!(
						"🌡 Temperature again under {} °C, warning was removed",
						ClimaSensorUS::CANCLE_TEMP
					),
				};
				nextcloud_sender.send(NextcloudEvent::Chat(message)).await?;
			}
			Ok(None) => (),
			Err(error) => {
				nextcloud_sender
					.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Error from weather station: {}",
						error
					)))
					.await?;
			}
		};
		interval.tick().await;
	}
}


async fn bat_loop(nextcloud_sender: Sender<NextcloudEvent>) -> Result<(), OpensesameError> {
	Ok(())
}

async fn button_loop(
	mut buttons: Buttons,
	mut validator: Validator,
	time_format: String,
	startup_time: String,
	mut command_receiver: Receiver<CommandToButtons>,
	nextcloud_sender: Sender<NextcloudEvent>,
	garage_enabled: bool,
	audio_bell: String,
	location_latitude: f64,
	location_longitude: f64,
) -> Result<(), OpensesameError> {
	let mut interval = interval(Duration::from_millis(10));
	let mut bell_task = Option::None;
	loop {
		if let Ok(command) = command_receiver.try_recv() {
			match command {
				CommandToButtons::OpenDoor => {
					buttons.open_door();
				},
				CommandToButtons::RingBell(period,counter) =>{
					buttons.ring_bell(period, counter);
				},
				CommandToButtons::SwitchLights(inside, outside, text) => {
					nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
						"{}. {}",
						text,
						buttons.switch_lights(inside, outside)
					))).await;
				},
				CommandToButtons::TurnOnLight => (),
			}
		}

		match buttons.handle() {
			Ok(StateChange::Pressed(button)) => match button {
				buttons::BUTTON_BELL => {
					let now = Local::now();
					if now.hour() >= 7 && now.hour() <= 21 {
						buttons.ring_bell(2, 5);
						if garage_enabled {
							bell_task = Some(play_audio_file(audio_bell.clone(), "--quiet".to_string().clone()));
							thread::Builder::new()
								.name(String::from("killall to ring bell"))
								.spawn(move || {
									exec_ssh_command("killall -SIGUSR2 opensesame".to_string());
								})
								.unwrap();
						}
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext("🔔 Pressed button bell.")))
							.await?;
					} else {
						buttons.show_wrong_input();
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"🔕 Did not ring bell (button was pressed) because the time 🌜 is {}, {}",
								now.format(&time_format)
							)))
							.await?;
					}
				}
				buttons::TASTER_INNEN => {
					nextcloud_sender
						.send(NextcloudEvent::Licht(gettext!(
							"💡 Pressed switch inside. {}.",
							buttons.switch_lights(true, true)
						)))
						.await?;
				}
				buttons::TASTER_AUSSEN => {
					nextcloud_sender
						.send(NextcloudEvent::Licht(gettext!(
							"💡 Pressed switch outside or light button. {}.",
							buttons.switch_lights(false, true),
						)))
						.await?;
				}
				buttons::TASTER_GLOCKE => {
					let now = Local::now();
					if now.hour() >= 7 && now.hour() <= 21 {
						buttons.ring_bell(5, 5);
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext("🔔 Pressed switch bell.")))
							.await?;
					} else {
						buttons.show_wrong_input();
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"🔕 Did not ring bell (taster outside) because the time 🌜 is {}, {}",
								now.format(&time_format)
							)))
							.await?;
					}
				}
				_ => panic!("🔘 Pressed {}", button),
			},
			Ok(StateChange::Released(_button)) => (),
			Ok(StateChange::LightsOff) => {
				nextcloud_sender
					.send(NextcloudEvent::Licht(gettext("🕶️ Light was turned off.")))
					.await?;
			}
			Ok(StateChange::None) => (),
			Ok(StateChange::Err(_)) => (),
			Err(error) => {
				return Err(Error::new(ErrorKind::ConnectionRefused, error));
			}
		}
		// Validation benötigt button, somit threads abhängig!!!; channel zwischen buttons und validator? damit validator nur getriggert ist wenn buttons sich ändert?
		// Validation start
		let sequence = buttons.sequence.to_vec();
		match validator.validate(&mut buttons.sequence) {
			Validation::Validated(user) => {
				buttons.open_door();
				nextcloud_sender
					.send(NextcloudEvent::Chat(gettext!("🤗 Opened for {}", user)))
					.await?;
				let now = Local::now();
				let (sunrise, sunset) = sunrise_sunset(
					location_latitude,
					location_longitude,
					now.year(),
					now.month(),
					now.day(),
				);
				if now.timestamp() < sunrise || now.timestamp() > sunset {
					nextcloud_sender
						.send(NextcloudEvent::Licht(gettext!(
							"💡 Switch lights in and out. {}",
							buttons.switch_lights(true, true)
						)))
						.await?;
				} else {
					nextcloud_sender
						.send(NextcloudEvent::Licht(gettext!(
							"🕶️ Don't switch lights as its day. Now: {} Sunrise: {} Sunset: {}",
							now.timestamp(),
							sunrise,
							sunset
						)))
						.await?;
				}
			}
			Validation::Timeout => {
				if sequence != vec![0, 15] {
					buttons.show_wrong_input();
					buttons.ring_bell(20, 0);
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"⌛ Timeout with sequence {}",
							format!("{:?}", sequence)
						)))
						.await?;
				}
			}
			Validation::SequenceTooLong => {
				buttons.show_wrong_input();
				buttons.ring_bell(20, 0);
				nextcloud_sender
					.send(NextcloudEvent::Chat(gettext!(
						"⌛ Sequence {} too long",
						format!("{:?}", sequence)
					)))
					.await?;
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
	RingBell(u32, u32), // maybe implement it with interval
	SwitchLights(bool, bool, String), // This also need to implement the sending of a Message to nextcloud, which is now in Garage
	                          // TODO Add more
}

#[tokio::main]
async fn main() -> Result<(), OpensesameError> {
	let mut config = Config::new(CONFIG_PARENT);
	let state = Config::new(STATE_PARENT);

	let date_time_format = &config.get::<String>("nextcloud/format/datetime");
	let startup_time = &Local::now().format(date_time_format);

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

	let nextcloud = Nextcloud::new(&mut config);

	let mut tasks = vec![];

	tasks.push(tokio::spawn(nextcloud_chat_loop(
		nextcloud,
		command_sender.clone(),
		nextcloud_receiver,
	)));

	if garage_enabled {
		if !buttons_enabled {
			panic!("Garage depends on buttons!");
		}
		let garage = Garage::new(&mut config);
		tasks.push(tokio::spawn(garage_loop(
			garage,
			command_sender.clone(),
			nextcloud_sender.clone(),
		)));
	}

	if buttons_enabled {
		let time_format = config.get::<String>("nextcloud/format/time");
		let garage_enabled = config.get_bool("garage/enable");
		let audio_bell = config.get::<String>("audio/bell");
		let location_latitude = config.get::<f64>("location/latitude");
		let location_longitude = config.get::<f64>("location/longitude");
		let mut buttons = Buttons::new(&mut config);
		let mut validator = Validator::new(&mut config);
		tasks.push(tokio::spawn(button_loop(
			buttons,
			validator,
			time_format.to_string(),
			startup_time.to_string(),
			command_receiver,
			nextcloud_sender.clone(),
			garage_enabled,
			audio_bell.to_string(),
			location_latitude,
			location_longitude,
		)));
	}

	if sensors_enabled {
		println!("In Sensor enabled if");
		let sensors = Sensors::new(&mut config);
		let device_path = config.get::<String>("sensors/device");
		tasks.push(tokio::spawn(sensors_loop(
			sensors,
			device_path.to_string(),
			nextcloud_sender.clone(),
		)));
	}

	if modir_enabled {
		let mod_ir_result = ModIR::new(&mut config);
		match mod_ir_result {
			Ok(mod_ir) => {
				let mut interval =
					interval(Duration::from_secs(config.get::<u64>("ir/data/interval")));
				tasks.push(tokio::spawn(modir_loop(
					mod_ir,
					interval,
					nextcloud_sender.clone(),
				)));
			}
			// TODO: Streamline consistent error handling!
			Err(error_typ) => {
				let reason = match error_typ {
					MlxError::I2C(error) => error.to_string(),
					MlxError::ChecksumMismatch | MlxError::InvalidInputData => {
						format!("{:?}", error_typ)
					}
				};
				nextcloud_sender
					.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init ModIR: {}",
						reason
					)))
					.await?;
			}
		}
	}

	if env_enabled {
		let mut interval = interval(Duration::from_secs(
			config.get::<u64>("environment/data/interval"),
		));
		let mut environment = Environment::new(&mut config);
		tasks.push(tokio::spawn(env_loop(
			environment,
			interval,
			nextcloud_sender.clone(),
			command_sender.clone(),
		)));
	}

	/*if weatherstation_enabled {
		let clima_sensor_result = ClimaSensorUS::new(&config);
		let interval = interval(Duration::from_secs(config.get::<u64>("weatherstation/data/interval")));
		match clima_sensor_result {
			Ok(clima_sensor) => {
				tasks.push(tokio::spawn(weatherstation_loop(clima_sensor, interval, nextcloud_sender.clone()));
			},
			Err(error) => {
				nextcloud_sender
					.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init libmodbus connection: {}",
						error
					)))
					.await;
			}
		}

	}*/

	if bat_enabled {
		tasks.push(tokio::spawn(bat_loop(nextcloud_sender.clone())));
	}

	join_all(tasks).await.unwrap();
	Ok(())
}
