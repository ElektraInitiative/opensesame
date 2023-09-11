// workaround until https://github.com/svartalf/rust-battery/issues/96 is solved

const CAPACITY_FILE: &str = "/sys/class/power_supply/axp20x-battery/capacity";

use futures::never::Never;
use std::fmt;
use std::fs;
use tokio::sync::mpsc::Sender;

use crate::nextcloud::NextcloudEvent;
use crate::types::ModuleError;

pub struct Bat {}

impl Bat {
	pub fn new() -> Self {
		Self {}
	}

	pub fn capacity(&self) -> u8 {
		match fs::read_to_string(CAPACITY_FILE) {
			Ok(str) => str.trim_end().parse::<u8>().unwrap(),
			Err(_err) => 100, // TTODO: Do not return full bat in error case!
		}
	}

	pub async fn get_background_task(
		_nextcloud_sender: Sender<NextcloudEvent>,
		//interval: Interval,
	) -> Result<Never, ModuleError> {
		// loop {
		// 	nextcloud_sender.send()
		// }
		Err(ModuleError::new(String::from("bat_loop not implemented")))
	}
}

impl fmt::Display for Bat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}%", self.capacity())
	}
}
