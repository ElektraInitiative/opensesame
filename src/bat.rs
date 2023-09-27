// workaround until https://github.com/svartalf/rust-battery/issues/96 is solved

const CAPACITY_FILE: &str = "/sys/class/power_supply/axp20x-battery/capacity";

use futures::never::Never;
use gettextrs::gettext;
use std::fmt;
use std::fs;
use systemstat::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

use crate::nextcloud::NextcloudChat;
use crate::nextcloud::NextcloudEvent;
use crate::types::ModuleError;

const START_CAPACITY_THRESHOLD: u8 = 50;

pub struct Bat {
	capacity: u8,
	capacity_threshold: u8,
}

impl Bat {
	pub fn new() -> Self {
		Self {
			capacity: 0,
			capacity_threshold: START_CAPACITY_THRESHOLD,
		}
	}

	pub fn capacity(&self) -> u8 {
		match fs::read_to_string(CAPACITY_FILE) {
			Ok(str) => str.trim_end().parse::<u8>().unwrap(),
			Err(_err) => 0, // TODO: Do not return full bat in error case!
		}
	}

	pub async fn get_background_task(
		mut self,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_secs(1800));
		loop {
			self.capacity = self.capacity();

			if self.capacity < self.capacity_threshold {
				if self.capacity_threshold - 10 > 0 {
					self.capacity_threshold -= 10;
				} else {
					self.capacity_threshold = 0;
				}
				nextcloud_sender
					.send(NextcloudEvent::Chat(
						NextcloudChat::Default,
						gettext!(
							"🪫 Battery Capacity is below {}% at {}%",
							self.capacity_threshold,
							self.capacity
						),
					))
					.await?;
			} else if self.capacity == 100 {
				self.capacity_threshold = 50;
				nextcloud_sender
					.send(NextcloudEvent::Chat(
						NextcloudChat::Default,
						gettext!("🔋 Battery Capacity is back to {}%", self.capacity),
					))
					.await?;
			}

			interval.tick().await;
		}
	}
}

impl fmt::Display for Bat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}%", self.capacity())
	}
}
