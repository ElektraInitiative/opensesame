use futures::never::Never;
use gettextrs::*;
use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use i2cdev::linux::LinuxI2CError;
use systemstat::Duration;
use tokio::{sync::mpsc::Sender, time::interval};

use crate::{
	buttons::CommandToButtons,
	config::Config,
	nextcloud::{NextcloudChat, NextcloudEvent},
	rst::Rst,
	types::ModuleError,
};

const MOD_IO_I2C_ADDR: u16 = 0x58;

const READ_COMMAND_FOR_IO_OPTO_PINS: u8 = 0x20;

const IN1: u8 = 0x1;
const IN3: u8 = 0x4;
const IN4: u8 = 0x8;

#[derive(PartialEq, Debug)]
pub enum HaustuerChange {
	None,

	LightFarOutdoor, // IN1
	// IN2 unused
	BellFarOutdoor, // IN3
	LightIndoor,    // IN4

	Err(String),
}

pub struct Haustuer {
	board: LinuxI2CDevice,

	light_far_outdoor: bool,
	bell_far_outdoor: bool,
	light_indoor: bool,

	how_many_err: u8,
}

impl Haustuer {
	pub fn new(_config: &mut Config) -> Self {
		Self {
			board: LinuxI2CDevice::new("/dev/i2c-2", MOD_IO_I2C_ADDR).unwrap(),

			light_far_outdoor: false,
			bell_far_outdoor: false,
			light_indoor: false,

			how_many_err: 0,
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
			self.how_many_err += 1;
			if self.how_many_err >= 58 {
				panic!("58 errors reached, giving up, last error {}", error)
			}
			return HaustuerChange::Err(gettext!("Board 58 with error {}", error));
		}
		self.how_many_err = 0;

		let pins = epins.unwrap();
		// self.set_relay(pins).unwrap();

		if pins & IN1 == IN1 {
			if !self.light_far_outdoor {
				self.light_far_outdoor = true;
				return HaustuerChange::LightFarOutdoor;
			}
		} else {
			self.light_far_outdoor = false;
		}

		// IN2 unused

		if pins & IN3 == IN3 {
			if !self.bell_far_outdoor {
				self.bell_far_outdoor = true;
				return HaustuerChange::BellFarOutdoor;
			}
		} else {
			self.bell_far_outdoor = false;
		}

		if pins & IN4 == IN4 {
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
		command_sender: Sender<CommandToButtons>,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_millis(30));
		let mut rst = Rst::new();
		loop {
			match haustuer.handle() {
				HaustuerChange::None => (),
				HaustuerChange::LightFarOutdoor => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Licht,
							gettext("💡 Light far outdoor pressed."),
						))
						.await?;
					command_sender
						.send(CommandToButtons::SwitchLights(
							false,
							true,
							gettext("💡 Pressed at entrance switch. Switch lights."),
						))
						.await?;
				}
				HaustuerChange::BellFarOutdoor => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							gettext("🔔 Bell far outdoor pressed."),
						))
						.await?;
					command_sender
						.send(CommandToButtons::RingBell(5, 5))
						.await?;
				}
				HaustuerChange::LightIndoor => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Licht,
							gettext("💡 Indoor light pressed."),
						))
						.await?;
					command_sender
						.send(CommandToButtons::SwitchLights(
							true,
							true,
							gettext("💡 Pressed in entrance. Switch all lights."),
						))
						.await?;
				}
				HaustuerChange::Err(err) => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Ping,
							gettext!("⚠️ Error on {}, will do reset.", err),
						))
						.await?;
					rst.do_reset().await?;
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Ping,
							gettext("👋 RST of MOD-IO done."),
						))
						.await?;
				}
			}
			interval.tick().await;
		}
	}
}
