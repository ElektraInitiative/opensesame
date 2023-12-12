use futures::never::Never;
use systemstat::Duration;
use tokio::{sync::mpsc::Sender, time::interval};
use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use i2cdev::linux::LinuxI2CError;

use crate::{
	buttons::CommandToButtons,
	config::Config,
	nextcloud::NextcloudEvent,
// 	nextcloud::{NextcloudChat, NextcloudEvent, NextcloudStatus},
	types::ModuleError,
};

const MOD_IO_I2C_ADDR: u16 = 0x58;

const READ_COMMAND_FOR_IO_OPTO_PINS: u8 = 0x20;


#[derive(PartialEq, Debug)]
pub enum HaustuerChange {
	None,

	Pressed(u8),

	Err (String),
}

pub struct Haustuer {
	board: LinuxI2CDevice,
}

impl Haustuer {
	pub fn new(_config: &mut Config) -> Self {
		Self {
			board: LinuxI2CDevice::new("/dev/i2c-2", MOD_IO_I2C_ADDR).unwrap()
		}
	}

	fn set_relay(&mut self, which: u8) -> Result<(), LinuxI2CError> {
		self.board.smbus_write_byte_data(0x10, which)?;
		Ok(())
	}

	pub fn handle(&mut self) -> HaustuerChange {
		let epins = self.board.smbus_read_byte_data(READ_COMMAND_FOR_IO_OPTO_PINS);
		if let Err(error) = epins {
			return HaustuerChange::Err(format!("Board 58 with error {}", error));
		}

		let pins = epins.unwrap();
		self.set_relay (pins).unwrap();

		if pins != 0 {
			return HaustuerChange::Pressed(pins);
		}

		HaustuerChange::None
	}

	pub async fn get_background_task(
		mut haustuer: Haustuer,
		_command_sender: Sender<CommandToButtons>,
		_nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_millis(10));
		loop {
			match haustuer.handle() {
				HaustuerChange::None => (),
				HaustuerChange::Pressed(epins) => {
					println!("Pressed {}", epins);
				}
				HaustuerChange::Err(err) => {
					println!("Pressed {}", err);
				}
			}
			interval.tick().await;
		}
	}
}
