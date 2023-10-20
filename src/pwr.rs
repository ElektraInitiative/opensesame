use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

use crate::config::Config;

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
}
