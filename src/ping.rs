use futures::never::Never;
use gettextrs::gettext;
use systemstat::{Platform, System};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{nextcloud::NextcloudEvent, types::ModuleError};

pub enum PingEvent {
	UpadeEnv(String),
	UpdateEnvStatus(u8),
	UpdateEnvError(u8),
	UpdateBatCapacity(u8),
	SendPing,
}

pub struct Ping {
	ping_counter: u64,
	environment: String,
	environment_status: u8,
	environment_error: u8,
	bat_capacity: u8,
	startup_time: String,
}

impl Ping {
	pub fn new(startup_time: String) -> Self {
		Self {
			ping_counter: 0,
			environment: String::from(""),
			environment_status: 0,
			environment_error: 0,
			bat_capacity: 0,
			startup_time,
		}
	}

	pub async fn get_background_task(
		mut self,
		mut ping_receiver: Receiver<PingEvent>,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		while let Some(event) = ping_receiver.recv().await {
			match event {
				PingEvent::UpadeEnv(value) => {
					self.environment = value;
				}
				PingEvent::UpdateEnvStatus(value) => {
					self.environment_status = value;
				}
				PingEvent::UpdateEnvError(value) => {
					self.environment_error = value;
				}
				PingEvent::UpdateBatCapacity(value) => {
					self.bat_capacity = value;
				}
				PingEvent::SendPing => {
					let sys = System::new();
					let loadavg = sys.load_average().unwrap();

					nextcloud_sender.send(NextcloudEvent::Ping(gettext!("{} Ping! Version {}, {}, Status {}, Error {}, Load {} {} {}, Memory usage {}, Swap {}, CPU temp {}, Startup {} Bat {}", 
						self.ping_counter,
						env!("CARGO_PKG_VERSION"),
						self.environment,
						self.environment_status,
						self.environment_error,
						loadavg.one,
						loadavg.five,
						loadavg.fifteen,
						sys.memory().unwrap().total,
						sys.swap().unwrap().total,
						sys.cpu_temp().unwrap(),
						self.startup_time,
						self.bat_capacity))).await?;

					self.ping_counter += 1;
				}
			}
		}

		Err(ModuleError::new(String::from("Exit Ping loop!")))
	}
}
