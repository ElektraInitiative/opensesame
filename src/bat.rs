// workaround until https://github.com/svartalf/rust-battery/issues/96 is solved

const CAPACITY_FILE : &'static str = "/sys/class/power_supply/axp20x-battery/capacity";

use std::fs;
use std::fmt;

pub struct Bat {
}

impl Bat {
	pub fn new() -> Self {
		Self {
		}
	}

	pub fn capacity(& self) -> u8 {
		match fs::read_to_string(CAPACITY_FILE) {
			Ok(str) => return str.trim_end().parse::<u8>().unwrap(),
			Err(_err) => return 100,
		}
	}
}

impl fmt::Display for Bat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}%", self.capacity())
	}
}
