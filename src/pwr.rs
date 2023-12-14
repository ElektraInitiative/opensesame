use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

use systemstat::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::nextcloud::NextcloudChat;
use crate::nextcloud::NextcloudEvent;
use gettextrs::gettext;

use crate::config::Config;
use crate::types::ModuleError;
use crate::watchdog;

const GPIO_PWR_LINE: u32 = 202;

pub struct Pwr {
	state: bool,
	pwr_line: Option<LineHandle>,
}

impl Pwr {
	pub fn new(config: &mut Config) -> Self {
		if config.get_bool("pwr/enable") {
			let mut chip = Chip::new("/dev/gpiochip0").unwrap();
			let line = chip
				.get_line(GPIO_PWR_LINE)
				.unwrap()
				.request(LineRequestFlags::OUTPUT, 0, "gpio_pwr_line")
				.unwrap();
			Self {
				state: line.get_value().unwrap() != 0,
				pwr_line: Some(line),
			}
		} else {
			Self {
				state: true,
				pwr_line: None,
			}
		}
	}

	pub fn enabled(&mut self) -> bool {
		match &self.pwr_line {
			Some(_pwr_line) => true,
			None => false,
		}
	}

	pub fn switch(&mut self, state: bool) {
		match &self.pwr_line {
			Some(pwr_line) => {
				if state && !self.state {
					pwr_line.set_value(1).unwrap();
				} else if self.state {
					pwr_line.set_value(0).unwrap();
				}
			}
			None => (),
		}
		self.state = state;
	}

	pub async fn do_reset(
		&mut self,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<(), ModuleError> {
		if self.enabled() {
			self.switch(false);
			nextcloud_sender
				.send(NextcloudEvent::Chat(
					NextcloudChat::Ping,
					gettext("👋 Turned PWR_SWITCH off"),
				))
				.await?;
			sleep(Duration::from_millis(watchdog::SAFE_TIMEOUT)).await;

			self.switch(true);
			nextcloud_sender
				.send(NextcloudEvent::Chat(
					NextcloudChat::Ping,
					gettext("👋 Turned PWR_SWITCH on"),
				))
				.await?;
			sleep(Duration::from_millis(watchdog::SAFE_TIMEOUT)).await;
		}
		Ok(())
	}
}
