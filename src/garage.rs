use futures::never::Never;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use systemstat::Duration;
use tokio::{sync::mpsc::Sender, time::interval};

use crate::{
	buttons::CommandToButtons, config::Config, nextcloud::NextcloudEvent, types::ModuleError,
};

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
		ret
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
					return GarageChange::PressedTasterTorUnten;
				}
			}
			None => (),
		}

		GarageChange::None
	}

	/// This function could be triggered by state changes on GPIO, because the pins are connected with the olimex board
	/// So we dont need to run it all few seconds.
	pub async fn get_background_task(
		mut garage: Garage,
		command_sender: Sender<CommandToButtons>,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_millis(10));
		loop {
			match garage.handle() {
				GarageChange::None => (),
				GarageChange::PressedTasterEingangOben => {
					// muss in buttons implementiert werden, damit button dann an nextcloud weiter gibt!

					/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
						"💡 Pressed at entrance top switch. Switch lights in garage. {}",
						buttons.switch_lights(true, false)
					)));*/
					command_sender
						.send(CommandToButtons::SwitchLights(
							true,
							false,
							"💡 Pressed at entrance top switch. Switch lights in garage"
								.to_string(),
						))
						.await?;
				}
				GarageChange::PressedTasterTorOben => {
					/*nextcloud_sender.send(NextcloudEvent::Licht(gettext!(
						"💡 Pressed top switch at garage door. Switch lights in and out garage. {}",
						buttons.switch_lights(true, true)
					)));*/
					command_sender
						.send(CommandToButtons::SwitchLights(
							true,
							true,
							"💡 Pressed top switch at garage door. Switch lights in and out garage"
								.to_string(),
						))
						.await?;
				}
				GarageChange::PressedTasterEingangUnten | GarageChange::PressedTasterTorUnten => {
					//buttons.open_door();
					command_sender.send(CommandToButtons::OpenDoor).await?;
				}

				GarageChange::ReachedTorEndposition => {
					nextcloud_sender
						.send(NextcloudEvent::SetStatusDoor(String::from("🔒 Open")))
						.await?;
					nextcloud_sender
						.send(NextcloudEvent::Chat(String::from("🔒 Garage door closed.")))
						.await?;
				}
				GarageChange::LeftTorEndposition => {
					nextcloud_sender
						.send(NextcloudEvent::SetStatusDoor(String::from("🔓 Closed")))
						.await?;
					nextcloud_sender
						.send(NextcloudEvent::Chat(String::from("🔓 Garage door open")))
						.await?;
				}
			}
			interval.tick().await;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{env, thread, time};

	const CONFIG_PARENT: &str = "/sw/libelektra/opensesame/#0/current";

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
			}
			thread::sleep(time::Duration::from_millis(10));
		}
	}
}
