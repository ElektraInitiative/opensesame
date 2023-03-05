// opensesame

mod bat;
mod buttons;
mod config;
mod environment;
mod garage;
mod nextcloud;
mod pwr;
mod sensors;
mod ssh;
mod validator;
mod watchdog;

use std::fs::File;
use std::io::{prelude::*, BufReader, Error};
use std::ops::Deref;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, thread, time};

use gettextrs::*;

use chrono::prelude::*;
use sunrise::sunrise_sunset;
use systemstat::{Platform, System};

use bat::Bat;
use buttons::Buttons;
use buttons::StateChange;
use config::Config;
use environment::AirQualityChange;
use environment::Environment;
use garage::Garage;
use garage::GarageChange;
use nextcloud::Nextcloud;
use pwr::Pwr;
use sensors::Sensors;
use sensors::SensorsChange;
use ssh::exec_ssh_command;
use validator::Validation;
use validator::Validator;
use watchdog::Watchdog;

const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";
const STATE_PARENT: &'static str = "/state/libelektra/opensesame/#0/current";

// play audio file with argument. If you do not have an argument, simply pass --quiet again
fn play_audio_file(file: String, arg: String) {
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
			})
			.unwrap();
	}
}

fn do_reset(watchdog: &mut Watchdog, pwr: &mut Pwr) {
	if pwr.enabled() {
		pwr.switch(false);
		watchdog.trigger();
		thread::sleep(time::Duration::from_millis(watchdog::SAFE_TIMEOUT));

		pwr.switch(true);
		watchdog.trigger();
		thread::sleep(time::Duration::from_millis(watchdog::SAFE_TIMEOUT));

		watchdog.trigger();
	}
}

fn handle_environment(
	environment: &mut Environment,
	nc: &mut Nextcloud,
	buttons: Option<&mut Buttons>,
	config: &mut Config,
) -> bool {
	nc.set_info_environment(format!("💨 {:?}", environment.air_quality));
	match environment.air_quality {
		AirQualityChange::Error => nc.send_message(gettext!(
			"⚠️ Error {:#02b} reading environment! Status: {:#02b}. {}",
			environment.error,
			environment.status,
			environment.to_string()
		)),

		AirQualityChange::Ok => {
			nc.send_message(gettext!("💨 Airquality is ok. {}", environment.to_string()))
		}
		AirQualityChange::Moderate => nc.send_message(gettext!(
			"💩 Airquality is moderate. {}",
			environment.to_string()
		)),
		AirQualityChange::Bad => nc.send_message(gettext!(
			"💩 Airquality is bad! {}",
			environment.to_string()
		)),

		AirQualityChange::FireAlarm => {
			return true;
		}
		AirQualityChange::FireBell => {
			nc.send_message(gettext!(
				"🚨 Possible fire alarm! Ring bell once! ⏰. {}",
				environment.to_string()
			));
			if let Some(buttons) = buttons {
				buttons.ring_bell(20, 0);
			}
			if config.get_bool("garage/enable") {
				play_audio_file(config.get::<String>("audio/alarm"), "--quiet".to_string());
				thread::Builder::new()
					.name("killall to ring bell".to_string())
					.spawn(move || {
						exec_ssh_command("killall -SIGUSR2 opensesame".to_string());
					})
					.unwrap();
			}
		}
		AirQualityChange::FireChat => nc.send_message(gettext!(
			"🚨 Possible fire alarm! (don't ring yet). {}",
			environment.to_string()
		)),
	}
	return false;
}

fn main() -> Result<(), Error> {
	let mut config: Config = Config::new(CONFIG_PARENT);
	env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
	let mut watchdog = Watchdog::new(&mut config);

	let term = Arc::new(AtomicBool::new(false));
	for signal in signal_hook::consts::TERM_SIGNALS {
		signal_hook::flag::register(*signal, Arc::clone(&term))?;
	}

	let sigalrm = Arc::new(AtomicBool::new(false));
	signal_hook::flag::register(signal_hook::consts::SIGALRM, Arc::clone(&sigalrm))?;

	let sigusr1 = Arc::new(AtomicBool::new(false));
	signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&sigusr1))?;

	let sigusr2 = Arc::new(AtomicBool::new(false));
	signal_hook::flag::register(signal_hook::consts::SIGUSR2, Arc::clone(&sigusr2))?;

	let sighup = Arc::new(AtomicBool::new(false));
	signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(&sighup))?;

	// https://stackoverflow.com/questions/42456497/stdresultresult-panic-to-log
	// Alternative: https://github.com/sfackler/rust-log-panics
	panic::set_hook(Box::new(|panic_info| {
		let (filename, line) = panic_info
			.location()
			.map(|loc| (loc.file(), loc.line()))
			.unwrap_or(("<unknown>", 0));
		let cause = panic_info
			.payload()
			.downcast_ref::<String>()
			.map(String::deref);
		let cause = cause.unwrap_or_else(|| {
			panic_info
				.payload()
				.downcast_ref::<&str>()
				.map(|s| *s)
				.unwrap_or("<cause unknown>")
		});
		let mut config: Config = Config::new(CONFIG_PARENT);
		let nc: Nextcloud = Nextcloud::new(&mut config);
		let text = gettext!("A panic occurred at {}:{}: {}", filename, line, cause);
		nc.ping(text.clone());
		eprintln!("{}", text);
	}));

	let mut nc: Nextcloud = Nextcloud::new(&mut config);

	let time_format = config.get::<String>("nextcloud/format/time");
	let date_time_format = config.get::<String>("nextcloud/format/datetime");
	let startup_time = Local::now().format(&date_time_format);

	TextDomain::new("opensesame").init().unwrap();

	nc.ping(gettext!(
		"👋 opensesame {} init {}",
		env!("CARGO_PKG_VERSION"),
		startup_time
	));

	let mut state: Config = Config::new(STATE_PARENT);

	let mut started_message_timeout = 10000;
	let enable_ping = config.get_bool("debug/ping/enable");
	let wait_for_ping_timeout = 300000 * config.get::<u32>("debug/ping/timeout");
	let mut wait_for_ping = 0;
	let mut ping_counter = 0u64;

	let mut remember_baseline_counter = 0;
	let wait_for_remember_baseline = 300000 * 24 * 7; // 7 days

	if config.get_option::<String>("sensors/#0/loc").is_some() {
		// first sensor is present
		let mut environment1 = Environment::new(&mut config);
		let mut config2 = Config::new(CONFIG_PARENT);
		config2.set("environment/device", "/dev/i2c-1");
		config2.set("environment/name", "Yvo Zimmer");
		let mut environment2 = Environment::new(&mut config2);

		let path = std::path::Path::new("/home/olimex/data.log");
		let mut outfile;
		if path.exists() {
			outfile = std::fs::OpenOptions::new()
				.write(true)
				.append(true)
				.open(path)
				.unwrap();
		} else {
			outfile = File::create(path).unwrap();
		}

		let file = File::open("/dev/ttyACM0").unwrap();
		let reader = BufReader::new(file);

		let mut sensors = Sensors::new(&mut config);

		for l in reader.lines() {
			if environment1.handle() {
				handle_environment(&mut environment1, &mut nc, None, &mut config);
			}
			if environment2.handle() {
				handle_environment(&mut environment2, &mut nc, None, &mut config);
			}

			let line = l.unwrap();

			// record data
			writeln!(
				&mut outfile,
				"{}	{}	Env1:	{}	{}	{}	{}	{}	{}	Env2:	{}	{}	{}	{}	{}	{}",
				Local::now().format(&date_time_format).to_string(),
				line.to_string(),
				environment1.co2,
				environment1.voc,
				environment1.temperature,
				environment1.humidity,
				environment1.pressure,
				environment1.baseline,
				environment2.co2,
				environment2.voc,
				environment2.temperature,
				environment2.humidity,
				environment2.pressure,
				environment2.baseline,
			)
			.unwrap();

			match sensors.update(line) {
				SensorsChange::None => (),
				SensorsChange::Alarm(w) => {
					nc.send_message(gettext!("Fire Alarm {}", w));
					/*
					state.set("alarm/fire", &w.to_string());
					sighup.store(true, Ordering::Relaxed);
					exec_ssh_command(format!("kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"", w));
					*/
				}
				SensorsChange::Chat(w) => {
					nc.send_message(gettext!("Fire Chat {}", w));
				}
			}

			if term.load(Ordering::Relaxed) {
				environment1.remember_baseline(&mut state);
				return Ok(());
			}

			if sighup.load(Ordering::Relaxed) {
				sighup.store(false, Ordering::Relaxed);
				config.sync();
				state.sync();
				environment1.restore_baseline(&mut state);
				environment2.restore_baseline(&mut state);
				nc.ping(gettext!(
					"👋 reloaded config&state in sensor mode for opensesame {} {}",
					env!("CARGO_PKG_VERSION"),
					startup_time
				));
			}
		}
	}

	let mut pwr = Pwr::new(&mut config);
	do_reset(&mut watchdog, &mut pwr);
	if pwr.enabled() {
		nc.ping(gettext("👋 Turned PWR_SWITCH on"));
	}
	let mut validator = Validator::new(&mut config);
	let mut buttons = Buttons::new(&mut config);
	let mut environment = Environment::new(&mut config);
	let mut garage = Garage::new(&mut config);
	let bat = Bat::new();
	let mut alarm_not_active = true;

	nc.set_info_online(gettext!("🪫 ON {}", bat));

	while !term.load(Ordering::Relaxed) {
		watchdog.trigger();

		if sigalrm.load(Ordering::Relaxed) {
			sigalrm.store(false, Ordering::Relaxed);
			buttons.ring_bell_alarm(20);
			play_audio_file(config.get::<String>("audio/alarm"), "--repeat".to_string());
			nc.send_message(gettext("🚨 Received alarm"));
		}

		if sigusr1.load(Ordering::Relaxed) {
			sigusr1.store(false, Ordering::Relaxed);
			wait_for_ping = wait_for_ping_timeout + 1;
		}

		if sigusr2.load(Ordering::Relaxed) {
			sigusr2.store(false, Ordering::Relaxed);
			buttons.ring_bell(20, 0);
			nc.send_message(gettext("🔔 Received bell"));
			play_audio_file(config.get::<String>("audio/bell"), "--quiet".to_string());
		}

		if sighup.load(Ordering::Relaxed) {
			nc.ping(gettext!(
				"👋reloading config&state for opensesame {} {}",
				env!("CARGO_PKG_VERSION"),
				startup_time
			));
			sighup.store(false, Ordering::Relaxed);
			config.sync();
			state.sync();
			environment.restore_baseline(&mut state);
			if let Some(alarm) = state.get_option::<String>("alarm/fire") {
				if alarm_not_active {
					nc.send_message(gettext!(
						"🚨 Fire Alarm! Fire Alarm! Fire ALARM! ⏰. {}",
						alarm
					));
					buttons.ring_bell_alarm(10);
					if config.get_bool("garage/enable") {
						play_audio_file(
							config.get::<String>("audio/alarm"),
							"--repeat".to_string(),
						);
						thread::Builder::new().name("killall to ring ALARM".to_string()).spawn(move || {
							exec_ssh_command(format!("kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"", alarm));
						}).unwrap();
					};
					alarm_not_active = false;
				}
			} else {
				// config option removed, go out of alarm mode
				alarm_not_active = true;
			}
		}

		if started_message_timeout > 1 {
			started_message_timeout -= 1;
		} else if started_message_timeout == 1 {
			nc.ping(gettext!(
				"👋 opensesame {} started {}",
				env!("CARGO_PKG_VERSION"),
				startup_time
			));
			started_message_timeout = 0; // job done, disable
			nc.set_info_online(gettext!("🔋 ON {}", bat));
		}

		if environment.handle() {
			if handle_environment(&mut environment, &mut nc, Some(&mut buttons), &mut config) {
				state.set("alarm/fire", &environment.name);
				sighup.store(true, Ordering::Relaxed);
			}
		}

		if enable_ping {
			wait_for_ping += 1;
		}
		if wait_for_ping > wait_for_ping_timeout {
			let sys = System::new();
			let loadavg = sys.load_average().unwrap();
			nc.ping (format!("{} Ping! Version {}, Watchdog {}, {}, Status {}, Error {}, Load {} {} {}, Memory usage {}, Swap {}, CPU temp {}, Startup {} Bat {}", ping_counter, env!("CARGO_PKG_VERSION"), watchdog.wait_for_watchdog_trigger, environment.to_string(), environment.status, environment.error, loadavg.one, loadavg.five, loadavg.fifteen, sys.memory().unwrap().total, sys.swap().unwrap().total, sys.cpu_temp().unwrap(), startup_time, bat));
			ping_counter += 1;
			wait_for_ping = 0; // restart
		}

		match garage.handle() {
			GarageChange::None => (),
			GarageChange::PressedTasterEingangOben => {
				nc.licht(gettext!(
					"💡 Pressed at entrance top switch. Switch lights in garage. {}",
					buttons.switch_lights(true, false)
				));
			}
			GarageChange::PressedTasterTorOben => {
				nc.licht(gettext!(
					"💡 Pressed top switch at garage door. Switch lights in and out garage. {}",
					buttons.switch_lights(true, true)
				));
			}
			GarageChange::PressedTasterEingangUnten | GarageChange::PressedTasterTorUnten => {
				buttons.open_door();
			}
			GarageChange::AutoClose => {
				buttons.open_door();
				nc.send_message(gettext!(
					"🔓 Garage door closes automatically. {}",
					environment.to_string()
				));
			}

			GarageChange::ReachedTorEndposition => {
				nc.set_info_door(gettext("🔒 Open"));
				nc.send_message(gettext!(
					"🔒 Garage door closed. {}",
					environment.to_string()
				));
			}
			GarageChange::LeftTorEndposition => {
				nc.set_info_door(gettext("🔓 Closed"));
				nc.send_message(gettext!("🔓 Garage door open. {}", environment.to_string()));
			}
		}

		let changes = buttons.handle();
		match changes {
			StateChange::Pressed(button) => {
				match button {
					buttons::BUTTON_BELL => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							buttons.ring_bell(2, 5);
							if config.get_bool("garage/enable") {
								play_audio_file(
									config.get::<String>("audio/bell"),
									"--quiet".to_string(),
								);
								thread::Builder::new()
									.name("killall to ring bell".to_string())
									.spawn(move || {
										exec_ssh_command("killall -SIGUSR2 opensesame".to_string());
									})
									.unwrap();
							}
							nc.send_message(gettext!(
								"🔔 Pressed button bell. {}",
								environment.to_string()
							));
						} else {
							buttons.show_wrong_input();
							nc.send_message(gettext!("🔕 Did not ring bell (button was pressed) because the time 🌜 is {}, {}", now.format(&time_format), environment.to_string()));
						}
					}
					buttons::TASTER_INNEN => {
						nc.licht(gettext!(
							"💡 Pressed switch inside. {}. {}",
							buttons.switch_lights(true, true),
							environment.to_string()
						));
					}
					buttons::TASTER_AUSSEN => {
						nc.licht(gettext!(
							"💡 Pressed switch outside or light button. {}. {}",
							buttons.switch_lights(false, true),
							environment.to_string()
						));
					}
					buttons::TASTER_GLOCKE => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							buttons.ring_bell(5, 5);
							nc.send_message(gettext!(
								"🔔 Pressed switch bell. {}",
								environment.to_string()
							));
						} else {
							buttons.show_wrong_input();
							nc.send_message(gettext!("🔕 Did not ring bell (taster outside) because the time 🌜 is {}, {}", now.format(&time_format), environment.to_string()));
						}
					}
					_ => panic!("🔘 Pressed {}, {}", button, environment.to_string()),
				}
			}
			StateChange::Released(_button) => (),
			StateChange::LightsOff => nc.licht(gettext!(
				"🕶️ Light was turned off. {}",
				environment.to_string()
			)),
			StateChange::None => (),
			StateChange::Err(board) => {
				let sys = System::new();
				let loadavg = sys.load_average().unwrap();
				nc.ping(gettext!("⚠️ Error reading buttons of board {}. Environment: {}, Load average: {} {} {}, Memory usage: {}, Swap: {}, CPU temp: {}, Bat: {}", board, environment.to_string(), loadavg.one, loadavg.five, loadavg.fifteen, sys.memory().unwrap().total, sys.swap().unwrap().total, sys.cpu_temp().unwrap(), bat));
				do_reset(&mut watchdog, &mut pwr);
			}
		}

		let sequence = buttons.sequence.to_vec();
		match validator.validate(&mut buttons.sequence) {
			Validation::Validated(user) => {
				buttons.open_door();
				nc.send_message(gettext!("🤗 Opened for {}", user));
				let now = Local::now();
				let (sunrise, sunset) = sunrise_sunset(
					config.get::<f64>("location/latitude"),
					config.get::<f64>("location/longitude"),
					now.year(),
					now.month(),
					now.day(),
				);
				if now.timestamp() < sunrise || now.timestamp() > sunset {
					nc.licht(gettext!(
						"💡 Switch lights in and out. {}",
						buttons.switch_lights(true, true)
					));
				} else {
					nc.licht(gettext!(
						"🕶️ Don't switch lights as its day. Now: {} Sunrise: {} Sunset: {}",
						now.timestamp(),
						sunrise,
						sunset
					));
				}
			}
			Validation::Timeout => {
				if sequence != vec![0, 15] {
					buttons.show_wrong_input();
					buttons.ring_bell(20, 0);
					nc.send_message(gettext!(
						"⌛ Timeout with sequence {}",
						format!("{:?}", sequence)
					));
				}
			}
			Validation::SequenceTooLong => {
				buttons.show_wrong_input();
				buttons.ring_bell(20, 0);
				nc.send_message(gettext!(
					"⌛ Sequence {} too long",
					format!("{:?}", sequence)
				));
			}
			Validation::None => (),
		}

		remember_baseline_counter += 1;
		if remember_baseline_counter == wait_for_remember_baseline {
			environment.remember_baseline(&mut state);
			remember_baseline_counter = 0;
		}

		thread::sleep(time::Duration::from_millis(10));
	}

	environment.remember_baseline(&mut state);
	nc.set_info_online(gettext("📴 OFF"));
	nc.ping(gettext!(
		"👋 opensesame {} bye-bye {:?}",
		env!("CARGO_PKG_VERSION"),
		Local::now()
	));

	Ok(())
}
