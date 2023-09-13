use chrono::Local;
use futures::future::join_all;
use gettextrs::*;
use mlx9061x::Error as MlxError;
use opensesame::audio::AudioEvent;
use opensesame::bat::Bat;
use opensesame::buttons::{Buttons, CommandToButtons};
use opensesame::clima_sensor_us::ClimaSensorUS;
use opensesame::config::Config;
use opensesame::environment::Environment;
use opensesame::garage::Garage;
use opensesame::mod_ir_temp::ModIR;
use opensesame::nextcloud::{Nextcloud, NextcloudEvent};
use opensesame::ping::{PingEvent, Ping};
use opensesame::pwr::Pwr;
use opensesame::sensors::Sensors;
use opensesame::types::ModuleError;
use opensesame::validator::Validator;
use opensesame::watchdog::Watchdog;
use opensesame::signals::Signals;
use std::panic;
use std::sync::Arc;
use systemstat::Duration;
use tokio::spawn;
use tokio::sync::{mpsc, Mutex};
use tokio::task::LocalSet;
use tokio::time::interval;

const CONFIG_PARENT: &str = "/sw/libelektra/opensesame/#0/current";
const STATE_PARENT: &str = "/state/libelektra/opensesame/#0/current";

#[tokio::main]
async fn main() -> Result<(), ModuleError> {
	TextDomain::new("opensesame").init().unwrap();

	let mut config = Config::new(CONFIG_PARENT);
	let _state = Arc::new(Mutex::new(Config::new(STATE_PARENT))); //Use Mutex for ENV and Signals

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

	let buttons_enabled = config.get_bool("buttons/enable");
	let garage_enabled = config.get_bool("garage/enable");
	let sensors_enabled = config.get_bool("sensors/enable");
	let modir_enabled = config.get_bool("ir/enable");
	let env_enabled = config.get_bool("environment/enable");
	let weatherstation_enabled = config.get_bool("weatherstation/enable");
	let bat_enabled = config.get_bool("bat/enable");
	let watchdog_enabled = config.get_bool("watchdog/enable");
	let ping_enabled = config.get_bool("ping/enable");

	let mut tasks = vec![];
	let local_set = LocalSet::new();

	tasks.push(spawn(Nextcloud::get_background_task(
		Nextcloud::new(&mut config),
		nextcloud_receiver,
		nextcloud_sender.clone(),
		command_sender.clone(),
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

	if buttons_enabled {
		let time_format = config.get::<String>("nextcloud/format/time");
		let garage_enabled = config.get_bool("garage/enable");
		let location_latitude = config.get::<f64>("location/latitude");
		let location_longitude = config.get::<f64>("location/longitude");
		tasks.push(spawn(Buttons::get_background_task(
			Buttons::new(&mut config),
			Validator::new(&mut config),
			Pwr::new(&mut config),
			time_format.to_string(),
			command_receiver,
			nextcloud_sender.clone(),
			garage_enabled,
			audio_bell.to_string(),
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
					.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init ModIR: {}",
						reason
					)))
					.await?;
			}
		}
	}

	if env_enabled {
		let interval = interval(Duration::from_secs(
			config.get::<u64>("environment/data/interval"),
		));
		tasks.push(spawn(Environment::get_background_task(
			Environment::new(&mut config),
			interval,
			nextcloud_sender.clone(),
			command_sender.clone(),
		)));
	}

	if env_enabled || buttons_enabled {
		let audio_bell = config.get::<String>("audio/bell");

		Audio::get_background_task(Audio::new(), audio_receiver);
	}

	if weatherstation_enabled {
		let clima_sensor_result = ClimaSensorUS::new(&mut config);
		let interval = interval(Duration::from_secs(
			config.get::<u64>("weatherstation/data/interval"),
		));
		match clima_sensor_result {
			Ok(clima_sensor) => {
				tasks.push(local_set.spawn_local(ClimaSensorUS::get_background_task(
					clima_sensor,
					interval,
					nextcloud_sender.clone(),
				)));
			}
			Err(error) => {
				nextcloud_sender
					.send(NextcloudEvent::Ping(gettext!(
						"⚠️ Failed to init libmodbus connection: {}",
						error
					)))
					.await?;
			}
		}
	}

	if bat_enabled {
		tasks.push(spawn(Bat::get_background_task(nextcloud_sender.clone())));
	}

	if watchdog_enabled {
		let interval = interval(Duration::from_secs(config.get::<u64>("watchdog/interval")));
		let path = config.get::<String>("watchdog/path");
		tasks.push(spawn(Watchdog::get_background_task(path, interval)));
	}

	//TODO: create Task for Signals and Ping!
	if ping_enabled {
		tasks.push(spawn(Ping::get_background_task(Ping::new(startup_time.to_string()), ping_receiver, nextcloud_sender.clone())))
	}

	tasks.push(spawn(Signals::get_background_task(ping_sender.clone(), ping_enabled, command_sender.clone(), buttons_enabled, nextcloud_sender.clone())));

	nextcloud_sender.send(
		NextcloudEvent::Chat(
			gettext!("Enabled Modules: Buttons: {}, Garage: {}, Sensors: {}, ModIR: {}, Environment: {}, Weatherstation: {}, Battery: {}, Watchdog: {}",
	buttons_enabled, 
	garage_enabled,
	sensors_enabled,
	modir_enabled,
	env_enabled,
	weatherstation_enabled,
	bat_enabled,
	watchdog_enabled,
))).await?;

	join_all(tasks).await;
	println!("hallo");
	Ok(())
}
