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
}
