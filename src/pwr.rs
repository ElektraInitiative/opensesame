// PWR_SWITCH

use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

use crate::config::Config;

const GPIO_PWR_LINE: u32 = 202;

pub struct Pwr {
	pwr_line: Option<LineHandle>,
	state: bool,
}

impl Pwr {
	pub fn new(config: &mut Config) -> Self {
		Self {
			pwr_line: if config.get_bool("pwr/enable") {
				let mut chip = Chip::new("/dev/gpiochip0").unwrap();
				let line = chip
					.get_line(GPIO_PWR_LINE)
					.unwrap()
					.request(LineRequestFlags::OUTPUT, 0, "gpio_pwr_line")
					.unwrap();
				line.set_value(1).unwrap();
				Some(line)
			} else {
				None
			},
			state: true,
		}
	}

	pub fn enabled(&mut self) -> bool {
		match &self.pwr_line {
			Some(_pwr_line) => return true,
			None => return false,
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
}

impl Drop for Pwr {
	fn drop(&mut self) {
		let _ = self.switch(false);
	}
}
