use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

use crate::config::Config;

const TASTER_EINGANG_OBEN_LINE: u32 = 234; // - Taster Eingang Oben             -> Pin40 GPIO234 EINT10
const TASTER_EINGANG_UNTEN_LINE: u32 = 235; // - Taster Eingang Unten            -> Pin38 GPIO235 EINT11
const TASTER_TOR_OBEN_LINE: u32 = 236; // - Taster Tor Oben                 -> Pin36 GPIO236 EINT12
const TASTER_TOR_UNTEN_LINE: u32 = 237; // - Taster Tor Unten                -> Pin32 GPIO237 EINT13
const SCHALTER_TOR_ENDPOSITION_LINE: u32 = 238; // - Schalter Garagentor Endposition -> Pin26 GPIO238 EINT14

#[derive(PartialEq, Debug)]
pub enum GarageChange {
	None,

	PressedTasterEingangOben,
	PressedTasterEingangUnten,
	PressedTasterTorOben,
	PressedTasterTorUnten,

	ReachedTorEndposition,
	LeftTorEndposition,

	AutoClose,
}

struct LineHandles {
	taster_eingang_oben_line: LineHandle,
	taster_eingang_unten_line: LineHandle,
	taster_tor_oben_line: LineHandle,
	taster_tor_unten_line: LineHandle,

	schalter_tor_endposition_line: LineHandle,
}

pub struct Garage {
	line_handles: Option<LineHandles>,

	taster_eingang_oben: bool,
	taster_eingang_unten: bool,
	taster_tor_oben: bool,
	taster_tor_unten: bool,
	schalter_tor_endposition: bool,

	auto_close_timeout: u16,
	auto_close: bool,
}

impl Garage {
	pub fn new(config: &mut Config) -> Self {
		Self {
			line_handles: if config.get_bool("garage/enable") {
				let mut chip = Chip::new("/dev/gpiochip0").unwrap();
				Some(LineHandles {
					taster_eingang_oben_line: chip
						.get_line(TASTER_EINGANG_OBEN_LINE)
						.unwrap()
						.request(LineRequestFlags::INPUT, 0, "taster_eingang_oben")
						.unwrap(),
					taster_eingang_unten_line: chip
						.get_line(TASTER_EINGANG_UNTEN_LINE)
						.unwrap()
						.request(LineRequestFlags::INPUT, 0, "taster_eingang_unten")
						.unwrap(),
					taster_tor_oben_line: chip
						.get_line(TASTER_TOR_OBEN_LINE)
						.unwrap()
						.request(LineRequestFlags::INPUT, 0, "taster_tor_oben")
						.unwrap(),
					taster_tor_unten_line: chip
						.get_line(TASTER_TOR_UNTEN_LINE)
						.unwrap()
						.request(LineRequestFlags::INPUT, 0, "taster_tor_unten")
						.unwrap(),
					schalter_tor_endposition_line: chip
						.get_line(SCHALTER_TOR_ENDPOSITION_LINE)
						.unwrap()
						.request(LineRequestFlags::INPUT, 0, "schalter_tor_endposition")
						.unwrap(),
				})
			} else {
				None
			},
			taster_eingang_oben: false,
			taster_eingang_unten: false,
			taster_tor_oben: false,
			taster_tor_unten: false,
			schalter_tor_endposition: false,

			auto_close_timeout: 0,
			auto_close: false,
		}
	}

	fn handle_line(now: u8, prev: &mut bool) -> bool {
		let mut ret = false;
		if now == 0 && !*prev {
			*prev = true;
			ret = true;
		}
		if now == 1 && *prev {
			*prev = false;
		}
		return ret;
	}

	fn handle_auto_close(&mut self) {
		if self.schalter_tor_endposition && self.auto_close_timeout == 0 {
			self.auto_close_timeout = 200
		} else if self.auto_close_timeout > 1 {
			// someone pressed within 2 sec, auto close in 2min
			self.auto_close_timeout = 12000;
			self.auto_close = true;
		}
	}

	pub fn handle(&mut self) -> GarageChange {
		match &self.line_handles {
			Some(line_handles) => {
				let s = line_handles
					.schalter_tor_endposition_line
					.get_value()
					.unwrap();
				if s == 0 && !self.schalter_tor_endposition {
					self.schalter_tor_endposition = true;
					return GarageChange::ReachedTorEndposition;
				} else if s == 1 && self.schalter_tor_endposition {
					self.schalter_tor_endposition = false;
					return GarageChange::LeftTorEndposition;
				}

				if Garage::handle_line(
					line_handles.taster_eingang_oben_line.get_value().unwrap(),
					&mut self.taster_eingang_oben,
				) {
					return GarageChange::PressedTasterEingangOben;
				}
				if Garage::handle_line(
					line_handles.taster_eingang_unten_line.get_value().unwrap(),
					&mut self.taster_eingang_unten,
				) {
					self.handle_auto_close();
					return GarageChange::PressedTasterEingangUnten;
				}
				if Garage::handle_line(
					line_handles.taster_tor_oben_line.get_value().unwrap(),
					&mut self.taster_tor_oben,
				) {
					return GarageChange::PressedTasterTorOben;
				}
				if Garage::handle_line(
					line_handles.taster_tor_unten_line.get_value().unwrap(),
					&mut self.taster_tor_unten,
				) {
					self.handle_auto_close();
					return GarageChange::PressedTasterTorUnten;
				}
			}
			None => (),
		}

		if self.auto_close_timeout == 1 && self.auto_close {
			self.auto_close = false;
			if !self.schalter_tor_endposition {
				return GarageChange::AutoClose;
			} else {
				// already closed, don't try to close again
				return GarageChange::None;
			}
		} else if self.auto_close_timeout > 0 {
			self.auto_close_timeout -= 1;
		}

		return GarageChange::None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{env, thread, time};

	const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";

	#[ignore] // remove and run with: cargo test print_events -- --nocapture
	#[test]
	fn print_events() {
		let mut config: Config = Config::new(CONFIG_PARENT);

		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));

		let mut garage = Garage::new(&mut config);

		loop {
			match garage.handle() {
				GarageChange::None => (),
				GarageChange::PressedTasterEingangOben => println!("Pressed Taster Eingang Oben"),
				GarageChange::PressedTasterEingangUnten => println!("Pressed Taster Eingang Unten"),
				GarageChange::PressedTasterTorOben => println!("Pressed Taster Tor Oben"),
				GarageChange::PressedTasterTorUnten => println!("Pressed Taster Tor Unten"),

				GarageChange::ReachedTorEndposition => println!("Reached Tor Endposition"),
				GarageChange::LeftTorEndposition => println!("Left Tor Endposition"),

				GarageChange::AutoClose => println!("Autoclose"),
			}
			thread::sleep(time::Duration::from_millis(10));
		}
	}
}
