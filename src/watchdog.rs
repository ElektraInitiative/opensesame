use std::fs;
use std::io::Write;

use crate::config::Config;

pub const TIMEOUT: u64 = 16 * 1000; // timeout in ms as seen in dmesg

pub struct Watchdog {
	handle: Option<fs::File>,
	pub wait_for_watchdog_trigger: u64,
}

impl Watchdog {
	pub fn new(config: &mut Config) -> Self {
		Self {
			handle: if config.get_bool("watchdog/enable") {
				Some(fs::File::create("/dev/watchdog").expect("could not open watchdog"))
			} else {
				None
			},
			wait_for_watchdog_trigger: 0,
		}
	}

	pub fn trigger(&mut self) {
		if let Some(handle) = &mut self.handle {
			self.wait_for_watchdog_trigger += 1;
			if self.wait_for_watchdog_trigger > 1000 {
				handle.write(b"a").expect("could not write to watchdog");
				self.wait_for_watchdog_trigger = 0;
			}
		}
	}
}
