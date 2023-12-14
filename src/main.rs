use chrono::Local;
use futures::future::join_all;
use gettextrs::*;
use mlx9061x::Error as MlxError;
use std::panic;
use std::sync::Arc;
use systemstat::Duration;
use tokio::spawn;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;

use opensesame::audio::{Audio, AudioEvent};
use opensesame::bat::Bat;
use opensesame::buttons::{Buttons, CommandToButtons};
use opensesame::clima_sensor_us::ClimaSensorUS;
use opensesame::config::Config;
use opensesame::environment::{EnvEvent, Environment};
use opensesame::garage::Garage;
use opensesame::haustuer::Haustuer;
use opensesame::mod_ir_temp::ModIR;
use opensesame::nextcloud::{Nextcloud, NextcloudChat, NextcloudEvent};
use opensesame::ping::{Ping, PingEvent};
use opensesame::pwr::Pwr;
use opensesame::sensors::Sensors;
use opensesame::signals::Signals;
use opensesame::types::ModuleError;
use opensesame::validator::Validator;
use opensesame::watchdog::Watchdog;

const CONFIG_PARENT: &str = "/sw/libelektra/opensesame/#0/current";
const STATE_PARENT: &str = "/state/libelektra/opensesame/#0/current";

#[tokio::main]
async fn main() -> Result<(), ModuleError> {
	TextDomain::new("opensesame").init().unwrap();

	let mut config = Config::new(CONFIG_PARENT);
	let config_mutex = Arc::new(Mutex::new(Config::new(CONFIG_PARENT)));
	let state_mutex = Arc::new(Mutex::new(Config::new(STATE_PARENT)));

	let date_time_format = config.get::<String>("nextcloud/format/datetime");
	let startup_time = Local::now().format(&date_time_format);

	// Sender and receiver to open doors/lights etc via Nextcloud
	let (command_sender, command_receiver) = mpsc::channel::<CommandToButtons>(32);
	// Info to send to next cloud
	let (nextcloud_sender, nextcloud_receiver) = mpsc::channel::<NextcloudEvent>(32);
	// Sender and receiver to set status of System and Send it to Nextcloud
	let (ping_sender, ping_receiver) = mpsc::channel::<PingEvent>(32);
	// Sender and receiver to play audio
	let (audio_sender, audio_receiver) = mpsc::channel::<AudioEvent>(32);

	let (environment_sender, environment_receiver) = mpsc::channel::<EnvEvent>(32);

	let buttons_enabled = config.get_bool("buttons/enable");
	let garage_enabled = config.get_bool("garage/enable");
	let haustuer_enabled = config.get_bool("haustuer/enable");
	let sensors_enabled = config.get_bool("sensors/enable");
	let modir_enabled = config.get_bool("ir/enable");
	let env_enabled = config.get_bool("environment/enable");
	let weatherstation_enabled = config.get_bool("weatherstation/enable");
	let bat_enabled = config.get_bool("bat/enable");
	let watchdog_enabled = config.get_bool("watchdog/enable");
	let ping_enabled = config.get_bool("ping/enable");

	let mut pwr = Pwr::new(&mut config);

	nextcloud_sender.send(
		NextcloudEvent::Chat(NextcloudChat::Ping,
			gettext!("Enabled Modules:\n\tButtons: {},\n\tGarage: {},\n\tHaustür: {},\n\tPWR: {},\n\tSensors: {},\n\tModIR: {},\n\tEnvironment: {},\n\tWeatherstation: {},\n\tBattery: {},\n\tWatchdog: {},\n\tPing: {}\n",
	buttons_enabled,
	garage_enabled,
	haustuer_enabled,
	pwr.enabled(),
	sensors_enabled,
	modir_enabled,
	env_enabled,
	weatherstation_enabled,
	bat_enabled,
	watchdog_enabled,
	ping_enabled,
))).await?;

	// use std::ops::Deref;
	// // https://stackoverflow.com/questions/42456497/stdresultresult-panic-to-log
	// // Alternative: https://github.com/sfackler/rust-log-panics
	// panic::set_hook(Box::new(|panic_info| {
	// 			let (filename, line) = panic_info
	// 			.location()
	// 			.map(|loc| (loc.file(), loc.line()))
	// 			.unwrap_or(("<unknown>", 0));
	// 			let cause = panic_info
	// 			.payload()
	// 			.downcast_ref::<String>()
	// 			.map(String::deref);
	// 			let cause = cause.unwrap_or_else(|| {
	// 					panic_info
	// 					.payload()
	// 					.downcast_ref::<&str>()
	// 					.map(|s| *s)
	// 					.unwrap_or("<cause unknown>")
	// 					});
	// 			let text = gettext!("A panic occurred at {}:{}: {}", filename, line, cause);
	// 			nextcloud_sender // <--- Doesn't live long enough
	// 				.send(NextcloudEvent::Chat(
	// 					NextcloudChat::Ping,
	// 					text.clone(),
	// 				));
	// 			eprintln!("{}", text);
	// 			}));

	let _ = pwr.do_reset(nextcloud_sender.clone()).await;

	let mut tasks = vec![];

	tasks.push(spawn(Nextcloud::get_background_task(
		Nextcloud::new(&mut config),
		nextcloud_receiver,
		nextcloud_sender.clone(),
		command_sender.clone(),
		audio_sender.clone(),
		startup_time.to_string(),
	)));

	if garage_enabled {
		if !buttons_enabled {
			panic!("Garage depends on buttons!");
		}
		tasks.push(spawn(Garage::get_background_task(
			Garage::new(&mut config),
			command_sender.clone(),
			nextcloud_sender.clone(),
		)));
	}

	if haustuer_enabled {
		if !buttons_enabled {
			panic!("Haustuer depends on buttons!");
		}
		tasks.push(spawn(Haustuer::get_background_task(
			Haustuer::new(&mut config),
			command_sender.clone(),
			nextcloud_sender.clone(),
		)));
	}

	if buttons_enabled {
		let time_format = config.get::<String>("nextcloud/format/time");
		let location_latitude = config.get::<f64>("location/latitude");
		let location_longitude = config.get::<f64>("location/longitude");
		tasks.push(spawn(Buttons::get_background_task(
			Buttons::new(&mut config),
			Validator::new(&mut config),
			time_format.to_string(),
			command_receiver,
			nextcloud_sender.clone(),
			audio_sender.clone(),
			location_latitude,
			location_longitude,
		)));
	}

	if sensors_enabled {
		let device_path = config.get::<String>("sensors/device");
		tasks.push(spawn(Sensors::get_background_task(
			Sensors::new(&mut config),
			device_path.to_string(),
			nextcloud_sender.clone(),
			/*state_mutex.clone(),
			id(),*/
		)));
	}

	if modir_enabled {
		let mod_ir_result = ModIR::new(&mut config);
		match mod_ir_result {
			Ok(mod_ir) => {
				let interval = interval(Duration::from_secs(config.get::<u64>("ir/data/interval")));
				tasks.push(spawn(ModIR::get_background_task(
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
					.send(NextcloudEvent::Chat(
						NextcloudChat::Ping,
						gettext!("⚠️ Failed to init ModIR: {}", reason),
					))
					.await?;
			}
		}
	}

	if env_enabled {
		let interval = interval(Duration::from_secs(
			config.get::<u64>("environment/data/interval"),
		));
		let garage_enabled = config.get_bool("garage/enable");
		tasks.push(spawn(Environment::get_background_task(
			Environment::new(&mut config, state_mutex.clone()),
			interval,
			nextcloud_sender.clone(),
			command_sender.clone(),
			audio_sender.clone(),
			environment_receiver,
			garage_enabled,
		)));
	}

	// if env_enabled || buttons_enabled {
	let audio_bell = config.get::<String>("audio/bell");
	let audio_alarm = config.get::<String>("audio/alarm");
	tasks.push(spawn(Audio::get_background_task(
		Audio::new(audio_bell, audio_alarm),
		audio_receiver,
		nextcloud_sender.clone(),
	)));
	// }

	if weatherstation_enabled {
		let clima_sensor_result = ClimaSensorUS::new(&mut config);
		match clima_sensor_result {
			Ok(clima_sensor) => {
				tasks.push(spawn(ClimaSensorUS::get_background_task(
					clima_sensor,
					nextcloud_sender.clone(),
				)));
			}
			Err(error) => {
				nextcloud_sender
					.send(NextcloudEvent::Chat(
						NextcloudChat::Ping,
						gettext!("⚠️ Failed to init libmodbus connection: {}", error),
					))
					.await?;
			}
		}
	}

	if bat_enabled {
		tasks.push(spawn(Bat::get_background_task(
			Bat::new(),
			nextcloud_sender.clone(),
			ping_sender.clone(),
		)));
	}

	if watchdog_enabled {
		let interval = interval(Duration::from_secs(config.get::<u64>("watchdog/interval")));
		let path = config.get::<String>("watchdog/path");
		tasks.push(spawn(Watchdog::get_background_task(path, interval)));
	}

	if ping_enabled {
		tasks.push(spawn(Ping::get_background_task(
			Ping::new(startup_time.to_string()),
			ping_receiver,
			nextcloud_sender.clone(),
		)))
	}

	let signals = Signals::new(
		config_mutex.clone(),
		state_mutex.clone(),
		ping_enabled,
		buttons_enabled,
		env_enabled,
		startup_time.to_string(),
		ping_sender.clone(),
		command_sender.clone(),
		nextcloud_sender.clone(),
		environment_sender.clone(),
		audio_sender.clone(),
	);

	tasks.push(spawn(signals.get_background_task()));

	nextcloud_sender
		.send(NextcloudEvent::Chat(
			NextcloudChat::Ping,
			gettext("⚠️ Startup Complete"),
		))
		.await?;

	join_all(tasks).await;
	Ok(())
}
