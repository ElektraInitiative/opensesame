use futures::never::Never;
use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use i2cdev::linux::LinuxI2CError;
use systemstat::Duration;
use tokio::{sync::mpsc::Sender, time::interval};

use crate::{
	buttons::CommandToButtons,
	config::Config,
	nextcloud::{NextcloudChat, NextcloudEvent},
	types::ModuleError,
};

const MOD_IO_I2C_ADDR: u16 = 0x58;

const READ_COMMAND_FOR_IO_OPTO_PINS: u8 = 0x20;

#[derive(PartialEq, Debug)]
pub enum HaustuerChange {
	None,

	LightOutdoor,    // IN1
	LightFarOutdoor, // IN2
	BellFarOutdoor,  // IN3
	LightIndoor,     // IN4

	Err(String),
}

pub struct Haustuer {
	board: LinuxI2CDevice,

	light_outdoor: bool,
	light_far_outdoor: bool,
	bell_far_outdoor: bool,
	light_indoor: bool,
}

impl Haustuer {
	pub fn new(_config: &mut Config) -> Self {
		Self {
			board: LinuxI2CDevice::new("/dev/i2c-2", MOD_IO_I2C_ADDR).unwrap(),

			light_outdoor: false,
			light_far_outdoor: false,
			bell_far_outdoor: false,
			light_indoor: false,
		}
	}

	fn _set_relay(&mut self, which: u8) -> Result<(), LinuxI2CError> {
		self.board.smbus_write_byte_data(0x10, which)?;
		Ok(())
	}

	pub fn handle(&mut self) -> HaustuerChange {
		let epins = self
			.board
			.smbus_read_byte_data(READ_COMMAND_FOR_IO_OPTO_PINS);
		if let Err(error) = epins {
			return HaustuerChange::Err(format!("Board 58 with error {}", error));
		}

		let pins = epins.unwrap();
		// self.set_relay(pins).unwrap();

		if pins & 1 == 1 {
			if !self.light_outdoor {
				self.light_outdoor = true;
				return HaustuerChange::LightOutdoor;
			}
		} else {
			self.light_outdoor = false;
		}

		if pins & 1 == 1 {
			if !self.light_outdoor {
				self.light_outdoor = true;
				return HaustuerChange::LightOutdoor;
			}
		} else {
			self.light_outdoor = false;
		}

		if pins & 2 == 2 {
			if !self.light_far_outdoor {
				self.light_far_outdoor = true;
				return HaustuerChange::LightFarOutdoor;
			}
		} else {
			self.light_far_outdoor = false;
		}

		if pins & 4 == 4 {
			if !self.bell_far_outdoor {
				self.bell_far_outdoor = true;
				return HaustuerChange::BellFarOutdoor;
			}
		} else {
			self.bell_far_outdoor = false;
		}

		if pins & 8 == 8 {
			if !self.light_indoor {
				self.light_indoor = true;
				return HaustuerChange::LightIndoor;
			}
		} else {
			self.light_indoor = false;
		}

		HaustuerChange::None
	}

	pub async fn get_background_task(
		mut haustuer: Haustuer,
		_command_sender: Sender<CommandToButtons>,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_millis(1000));
		loop {
			match haustuer.handle() {
				HaustuerChange::None => (),
				HaustuerChange::LightOutdoor => {
					println!("LightOutdoor Pressed");
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							String::from("🔒 LightOutdoor pressed."),
						))
						.await?;
				}
				HaustuerChange::LightFarOutdoor => {
					println!("LightFarOutdoor Pressed");
				}
				HaustuerChange::BellFarOutdoor => {
					println!("BellFarOutdoor Pressed");
				}
				HaustuerChange::LightIndoor => {
					println!("LightIndoor Pressed");
				}
				HaustuerChange::Err(err) => {
					println!("Error {}", err);
				}
			}
			interval.tick().await;
		}
	}
}
