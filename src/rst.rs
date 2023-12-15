use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

use systemstat::Duration;
use tokio::time::sleep;

use crate::types::ModuleError;

const GPIO_RST_LINE: u32 = 203;

pub struct Rst {
	state: bool,
	rst_line: LineHandle,
}

impl Rst {
	pub fn new() -> Self {
		let mut chip = Chip::new("/dev/gpiochip0").unwrap();
		let line = chip
			.get_line(GPIO_RST_LINE)
			.unwrap()
			.request(LineRequestFlags::OUTPUT, 0, "gpio_rst_line")
			.unwrap();
		Self {
			state: line.get_value().unwrap() != 0,
			rst_line: line,
		}
	}

	pub fn switch(&mut self, state: bool) {
		if state && !self.state {
			self.rst_line.set_value(1).unwrap();
		} else if self.state {
			self.rst_line.set_value(0).unwrap();
		}
		self.state = state;
	}

	pub async fn do_reset(&mut self) -> Result<(), ModuleError> {
		self.switch(false);
		sleep(Duration::from_millis(10)).await;

		self.switch(true);
		sleep(Duration::from_millis(10)).await;
		Ok(())
	}
}
