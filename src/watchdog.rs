use futures::never::Never;

use tokio::{fs::File, io::AsyncWriteExt, time::Interval};

use crate::types::ModuleError;

pub const SAFE_TIMEOUT: u64 = 15 * 1000; // safe to wait if trigger was done just before

pub struct Watchdog {}

impl Watchdog {
	pub async fn get_background_task(
		path: String,
		mut interval: Interval,
	) -> Result<Never, ModuleError> {
		let mut handle = File::create(path)
			.await
			.map_err(|_| ModuleError::new(String::from("could not open watchdog")))?;
		loop {
			interval.tick().await;
			handle
				.write_all(b"a")
				.await
				.map_err(|_| ModuleError::new(String::from("could not write to watchdog")))?;
		}
	}

	//This function is only used to test if the system reboots after this module stops writing to the watchdog file.
	//For testing change the get_background_task to test_get_background_task in main.rs
	pub async fn test_get_background_task(
		path: String,
		mut interval: Interval,
	) -> Result<Never, ModuleError> {
		let mut handle = File::create(path)
			.await
			.map_err(|_| ModuleError::new(String::from("could not open watchdog")))?;
		for _i in 1..=100 {
			interval.tick().await;
			handle
				.write_all(b"a")
				.await
				.map_err(|_| ModuleError::new(String::from("could not write to watchdog")))?;
		}
		Err(ModuleError::new(String::from("doesn't run for loop")))
	}
}
