use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;
use futures::never::Never;
use gettextrs::gettext;
use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use i2cdev::linux::LinuxI2CError;
use sunrise::sunrise_sunset;
use systemstat::Duration;
use systemstat::{Platform, System};
use tokio::process::Command;
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;
use tokio::time::sleep;

use crate::audio;
use crate::audio::AudioEvent;
use crate::config::Config;
use crate::nextcloud::NextcloudEvent;
use crate::pwr::Pwr;
use crate::ssh::exec_ssh_command;
use crate::types::ModuleError;
use crate::validator::{Validation, Validator};
use crate::watchdog;

pub struct Buttons {
	pub sequence: Vec<u8>,
	pub pins1: u8,
	pub pins2: u8,
	pub led1: bool,
	pub led2: bool,
	pub led3: bool,
	pub led4: bool,
	pub led_light: bool,
	pub led_bell: bool,

	pub door_timeout: u32,

	pub init_light_timeout: u32,
	pub light_timeout: u32,
	pub light_permanent: bool,

	// timeout is used to count until period
	//       X------->period
	//       |-------|
	//       |       |
	//       |       |
	// ------|       |--------
	//
	// counter is how often this is done
	pub bell_enable: bool,
	pub bell_timeout: u32,
	pub bell_timeout_init: u32,
	pub bell_counter: u32,
	failed_counter: u8,
	wrong_input_timeout: u8,

	board20: LinuxI2CDevice,
	board21: LinuxI2CDevice,
}

#[derive(PartialEq, Debug)]
pub enum StateChange {
	None,
	Err(u8),
	Pressed(u8),
	Released(u8),
	LightsOff,
}

pub enum CommandToButtons {
	OpenDoor,
	TurnOnLight,
	RingBell(u32, u32), // maybe implement it with interval
	RingBellAlarm(u32),
	SwitchLights(bool, bool, String), // This also need to implement the sending of a Message to nextcloud, which is now in Garage
}

const FAILED_COUNTER: u8 = 20; // = 200ms how long to wait after failure before resetting (*10ms)
const BELL_MINIMUM_PERIOD: u32 = 20; // = 200ms shortest period time for bell

const SET_TRIS: u8 = 0x01; // Set GPIO direction
const SET_PORTS: u8 = 0x02; // Set GPIO output level
const GET_PORTS: u8 = 0x03; // Get GPIO input level
const SET_PULLUPS: u8 = 0x04; // Set GPIO pull-ups
const SET_RELAYS_ON: u8 = 0x41; // Set relay(s) on
const SET_RELAYS_OFF: u8 = 0x42; // Set relay(s) off

// board 20

const BOARD20: u16 = 0x20;

const BUTTON_1: u8 = 0x01;
const BUTTON_2: u8 = 0x01 << 1;
const BUTTON_3: u8 = 0x01 << 2;
const BUTTON_4: u8 = 0x01 << 3; // = GPIO3 with external pull-up

// input from GPIO0 - GPIO3 i.e. all buttons (and also buttons+taster on board21)
const ALL_BUTTONS: u8 = BUTTON_1 | BUTTON_2 | BUTTON_3 | BUTTON_4;

const LED_1: u8 = 0x01 << 4;
const LED_2: u8 = 0x01 << 5;
const LED_3: u8 = 0x01 << 6;

const RELAY_DOOR: u8 = 0x01;
const RELAY_LICHT_AUSSEN: u8 = 0x01 << 1;

const ALL_RELAYS: u8 = RELAY_DOOR | RELAY_LICHT_AUSSEN;

const PINS1_INIT: u8 = 15;

// board 21

const BOARD21: u16 = 0x21;

pub const BUTTON_LIGHT: u8 = 0x01;
pub const BUTTON_BELL: u8 = 0x01 << 1;

pub const TASTER_AUSSEN: u8 = BUTTON_LIGHT;
pub const TASTER_INNEN: u8 = 0x01 << 2;
pub const TASTER_GLOCKE: u8 = 0x01 << 3; // = GPIO3 with external pull-up

const LED_4: u8 = 0x01 << 4;
const LED_LIGHT: u8 = 0x01 << 5;
const LED_BELL: u8 = 0x01 << 6;

const RELAY_BELL: u8 = 0x01;
const RELAY_LICHT_INNEN: u8 = 0x01 << 1;

const PINS2_INIT: u8 = 0b01100000;

impl Buttons {
	pub fn new(config: &mut Config) -> Self {
		let mut s = Self {
			sequence: vec![],

			pins1: PINS1_INIT,
			pins2: PINS2_INIT,

			led1: false,
			led2: false,
			led3: false,
			led4: false,
			led_light: false,
			led_bell: false,

			door_timeout: 0,

			init_light_timeout: config.get::<u32>("light/timeout") * 100,
			light_timeout: 0,
			light_permanent: false,

			bell_enable: config.get_bool("bell/enable"),
			bell_timeout: 0,
			bell_timeout_init: 0,
			bell_counter: 0,
			failed_counter: 0,
			wrong_input_timeout: 0,
			board20: LinuxI2CDevice::new("/dev/i2c-2", BOARD20).unwrap(),
			board21: LinuxI2CDevice::new("/dev/i2c-2", BOARD21).unwrap(),
		};
		s.init();
		s
	}

	fn init(&mut self) {
		self.board20
			.smbus_write_byte_data(SET_TRIS, ALL_BUTTONS)
			.expect("I2C Communication to Buttons does not work");
		self.board21
			.smbus_write_byte_data(SET_TRIS, ALL_BUTTONS)
			.unwrap();

		self.board20
			.smbus_write_byte_data(SET_PULLUPS, ALL_BUTTONS)
			.unwrap();
		self.board21
			.smbus_write_byte_data(SET_PULLUPS, ALL_BUTTONS)
			.unwrap();

		self.turn_everything_off().unwrap();
	}

	fn turn_everything_off(&mut self) -> Result<(), LinuxI2CError> {
		// all LEDs:
		self.led1 = false;
		self.led2 = false;
		self.led3 = false;
		self.led4 = false;
		self.led_light = false;
		self.led_bell = false;

		// all timeouts
		self.door_timeout = 0;
		self.light_timeout = 0;
		self.bell_counter = 0;
		self.bell_timeout = 0;

		self.board20
			.smbus_write_byte_data(SET_RELAYS_OFF, ALL_RELAYS)?;
		self.board21
			.smbus_write_byte_data(SET_RELAYS_OFF, ALL_RELAYS)?;

		self.board20.smbus_write_byte_data(SET_PORTS, ALL_BUTTONS)?;
		self.board21.smbus_write_byte_data(SET_PORTS, ALL_BUTTONS)?;
		Ok(())
	}

	fn handle_door(&mut self) {
		if self.door_timeout == 1 {
			self.board20
				.smbus_write_byte_data(SET_RELAYS_OFF, RELAY_DOOR)
				.unwrap();
			self.led_bell = false;
			self.door_timeout = 0;
		} else if self.door_timeout > 0 {
			self.door_timeout -= 1;
		}
	}

	fn handle_light(&mut self) -> bool {
		let mut ret = false;
		let timeout_progress;
		if self.light_permanent {
			timeout_progress = 0;
		} else if self.light_timeout == self.init_light_timeout {
			self.board20
				.smbus_write_byte_data(SET_RELAYS_ON, RELAY_LICHT_AUSSEN)
				.unwrap();
			timeout_progress = 1;
		} else if self.light_timeout == 10 {
			self.board20
				.smbus_write_byte_data(SET_RELAYS_OFF, RELAY_LICHT_AUSSEN)
				.unwrap();
			timeout_progress = 1;
		} else if self.light_timeout == 1 {
			self.board21
				.smbus_write_byte_data(SET_RELAYS_OFF, RELAY_LICHT_INNEN)
				.unwrap();

			self.led_light = false;

			timeout_progress = 1;
			self.light_permanent = false;
			ret = true;
		} else if self.light_timeout > 0 {
			timeout_progress = 1;
		} else {
			assert!(self.light_timeout == 0, "wrong logic");
			timeout_progress = 0;
		}
		self.light_timeout -= timeout_progress;
		ret
	}

	fn handle_bell(&mut self) {
		if !self.bell_enable {
			return;
		}
		if self.bell_counter == 0 {
			return;
		}
		if self.bell_timeout == 0 {
			self.bell_timeout = self.bell_timeout_init;
			if self.bell_counter % 2 == 0 {
				self.board21
					.smbus_write_byte_data(SET_RELAYS_ON, RELAY_BELL)
					.unwrap();
			} else {
				self.board21
					.smbus_write_byte_data(SET_RELAYS_OFF, RELAY_BELL)
					.unwrap();
				self.led_bell = false;
			}
			self.bell_counter -= 1;
		}

		self.bell_timeout -= 1;
	}

	fn handle_wrong_input(&mut self) -> bool {
		if self.wrong_input_timeout == 1 {
			self.led_light = false;
			self.led1 = false;
			self.led2 = false;
			self.led3 = false;
			self.led4 = false;
			self.wrong_input_timeout = 0;
			return false;
		} else if self.wrong_input_timeout > 1 {
			self.wrong_input_timeout -= 1;
			return false;
		}
		true
	}

	/// to be periodically called, e.g. every 10 ms
	pub fn handle(&mut self) -> Result<StateChange, LinuxI2CError> {
		// wait for recover
		if self.failed_counter > 1 {
			self.failed_counter -= 1;
			return Ok(StateChange::None);
		// try to recover
		} else if self.failed_counter == 1 {
			self.pins1 = PINS1_INIT;
			self.pins2 = PINS2_INIT;
			self.init();
			self.failed_counter = 0;
			return Ok(StateChange::None);
		}

		let epins1 = self.board20.smbus_read_byte_data(GET_PORTS);
		if let Err(error) = epins1 {
			self.failed_counter = FAILED_COUNTER;
			self.led1 = true;
			self.led2 = true;
			return Err(error);
		}

		let epins2 = self.board21.smbus_read_byte_data(GET_PORTS);
		if let Err(error) = epins2 {
			self.failed_counter = FAILED_COUNTER;
			self.led1 = true;
			self.led3 = true;
			return Err(error);
		}

		let mut pins1 = epins1.unwrap() & ALL_BUTTONS;
		let mut pins2 = epins2.unwrap() & ALL_BUTTONS;

		// check first if something relevant to sequence changed:
		if pins1 != (self.pins1 & ALL_BUTTONS) {
			self.sequence.push(pins1);
		}

		// now determine the StateChange
		let ret: StateChange;
		if pins2 < (self.pins2 & ALL_BUTTONS) {
			// pressed buttons are logical 0
			ret = StateChange::Pressed(!pins2 & self.pins2 & ALL_BUTTONS);
		} else if pins2 > (self.pins2 & ALL_BUTTONS) {
			ret = StateChange::Released(pins2 & !self.pins2 & ALL_BUTTONS);
		} else if self.handle_light() {
			ret = StateChange::LightsOff;
		} else {
			ret = StateChange::None;
		}

		if self.handle_wrong_input() {
			self.handle_bell();
			self.handle_door();
		}

		// now calculate output
		if pins1 & BUTTON_1 == 0 || self.led1 {
			pins1 |= LED_1;
		}
		if pins1 & BUTTON_2 == 0 || self.led2 {
			pins1 |= LED_2;
		}
		if pins1 & BUTTON_3 == 0 || self.led3 {
			pins1 |= LED_3;
		}
		if pins1 & BUTTON_4 == 0 || self.led4 {
			pins2 |= LED_4;
		} // LED_4 is on second board
		if pins2 & BUTTON_LIGHT == 0 || self.led_light {
			pins2 |= LED_LIGHT;
		}
		if pins2 & BUTTON_BELL == 0 || self.led_bell {
			pins2 |= LED_BELL;
		}

		// println!("pins1 {} {:08b}, self.pins1 {} {:08b}, pins2 {} {:08b}, self.pins2 {} {:08b}", pins1, pins1, self.pins1, self.pins1, pins2, pins2, self.pins2, self.pins2);

		if pins1 != self.pins1 {
			// println!("will write pins1 {:02} {:08b}", pins1, pins1);
			self.board20
				.smbus_write_byte_data(SET_PORTS, pins1 & !ALL_BUTTONS)
				.unwrap();
			self.pins1 = pins1;
		}

		if pins2 != self.pins2 {
			// println!("will write pins2 {:02} {:08b}", pins2, pins2);
			self.board21
				.smbus_write_byte_data(SET_PORTS, pins2 & !ALL_BUTTONS)
				.unwrap();
			self.pins2 = pins2;
		}
		Ok(ret)
	}

	/// opensesame!
	pub fn open_door(&mut self) {
		self.board20
			.smbus_write_byte_data(SET_RELAYS_ON, RELAY_DOOR)
			.unwrap();
		self.led_bell = true;
		self.door_timeout = 150;
	}

	pub fn show_wrong_input(&mut self) {
		self.wrong_input_timeout = 150;
		self.led_light = true;
		self.led1 = true;
		self.led2 = true;
		self.led3 = true;
		self.led4 = true;
	}

	/// start ringing bell with given period, for very long or until ring_bell is called, which terminates the alarm
	pub fn ring_bell_alarm(&mut self, period: u32) {
		self.board21
			.smbus_write_byte_data(SET_RELAYS_ON, RELAY_BELL)
			.unwrap();
		self.led_light = true;
		self.bell_counter = u32::MAX; // never stop
		self.bell_timeout_init = period * BELL_MINIMUM_PERIOD;
		self.bell_timeout = self.bell_timeout_init;
	}

	/// start ringing bell with given period for a short time
	pub fn ring_bell(&mut self, period: u32, counter: u32) {
		if !self.bell_enable {
			return;
		}
		self.board21
			.smbus_write_byte_data(SET_RELAYS_ON, RELAY_BELL)
			.unwrap();
		self.led_bell = true;
		self.bell_counter = counter * 2 + 1;
		self.bell_timeout_init = period * BELL_MINIMUM_PERIOD;
		self.bell_timeout = self.bell_timeout_init;
	}

	/// returns what was done
	/// usually extends light time
	/// on double press event (on true) -> make light permanent on (until next press event)
	pub fn switch_lights(&mut self, inside: bool, outside: bool) -> String {
		assert!(
			inside || outside,
			"logic error, at least one must be switched on!"
		);

		let init_light_timeout = if outside {
			self.init_light_timeout + 10
		} else {
			self.init_light_timeout - 1
		};

		let ret;
		if self.light_permanent {
			self.light_permanent = false;
			self.light_timeout = 30; // turn off soon
			return "Light not permanent anymore".to_string();
		} else if inside && self.light_timeout > init_light_timeout - 200 {
			// make permanent
			self.light_permanent = true;
			ret = "Light now permanently on".to_string();
		} else if self.light_timeout > 1 {
			// extend
			self.light_timeout = init_light_timeout;
			ret = "Time extended.".to_string();
		} else {
			self.light_timeout = init_light_timeout;
			self.led_light = true;
			ret = "Light switched on.".to_string();
		}

		// now actually switch on (might also extend light if it was only outside before)
		if inside {
			self.board21
				.smbus_write_byte_data(SET_RELAYS_ON, RELAY_LICHT_INNEN)
				.unwrap();
		}
		ret
	}

	async fn do_reset(
		nextcloud_sender: Sender<NextcloudEvent>,
		pwr: &mut Pwr,
	) -> Result<(), ModuleError> {
		if pwr.enabled() {
			pwr.switch(false);
			nextcloud_sender
				.send(NextcloudEvent::Ping(gettext("👋 Turned PWR_SWITCH off")))
				.await?;
			sleep(Duration::from_millis(watchdog::SAFE_TIMEOUT)).await;

			pwr.switch(true);
			nextcloud_sender
				.send(NextcloudEvent::Ping(gettext("👋 Turned PWR_SWITCH on")))
				.await?;
			sleep(Duration::from_millis(watchdog::SAFE_TIMEOUT)).await;
		}
		Ok(())
	}

	pub async fn get_background_task(
		mut self,
		mut validator: Validator,
		mut pwr: Pwr,
		time_format: String,
		mut command_receiver: Receiver<CommandToButtons>,
		nextcloud_sender: Sender<NextcloudEvent>,
		audio_sender: Sender<AudioEvent>,
		garage_enabled: bool,
		location_latitude: f64,
		location_longitude: f64,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_millis(10));
		loop {
			if let Ok(command) = command_receiver.try_recv() {
				match command {
					CommandToButtons::OpenDoor => {
						self.open_door();
					}
					CommandToButtons::RingBell(period, counter) => {
						self.ring_bell(period, counter);
					}
					CommandToButtons::SwitchLights(inside, outside, text) => {
						nextcloud_sender
							.send(NextcloudEvent::Licht(gettext!(
								"{}. {}",
								text,
								self.switch_lights(inside, outside)
							)))
							.await?;
					}
					CommandToButtons::TurnOnLight => (),
					CommandToButtons::RingBellAlarm(period) => {
						self.ring_bell_alarm(period);
					}
				}
			}

			match self.handle()? {
				StateChange::Pressed(button) => match button {
					BUTTON_BELL => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							self.ring_bell(2, 5);
							if garage_enabled {
								audio_sender.send(AudioEvent::Bell);
							}
							nextcloud_sender
								.send(NextcloudEvent::Chat(gettext("🔔 Pressed button bell.")))
								.await?;
						} else {
							self.show_wrong_input();
							nextcloud_sender
								.send(NextcloudEvent::Chat(gettext!(
									"🔕 Did not ring bell (button was pressed) because the time 🌜 is {}, {}",
									now.format(&time_format)
								)))
								.await?;
						}
					}
					TASTER_INNEN => {
						nextcloud_sender
							.send(NextcloudEvent::Licht(gettext!(
								"💡 Pressed switch inside. {}.",
								self.switch_lights(true, true)
							)))
							.await?;
					}
					TASTER_AUSSEN => {
						nextcloud_sender
							.send(NextcloudEvent::Licht(gettext!(
								"💡 Pressed switch outside or light button. {}.",
								self.switch_lights(false, true),
							)))
							.await?;
					}
					TASTER_GLOCKE => {
						let now = Local::now();
						if now.hour() >= 7 && now.hour() <= 21 {
							self.ring_bell(5, 5);
							nextcloud_sender
								.send(NextcloudEvent::Chat(gettext("🔔 Pressed switch bell.")))
								.await?;
						} else {
							self.show_wrong_input();
							nextcloud_sender
								.send(NextcloudEvent::Chat(gettext!(
									"🔕 Did not ring bell (taster outside) because the time 🌜 is {}, {}",
									now.format(&time_format)
								)))
								.await?;
						}
					}
					_ => panic!("🔘 Pressed {}", button),
				},
				StateChange::Released(_button) => (),
				StateChange::LightsOff => {
					nextcloud_sender
						.send(NextcloudEvent::Licht(gettext("🕶️ Light was turned off.")))
						.await?;
				}
				StateChange::None => (),
				StateChange::Err(board) => {
					let sys = System::new();
					let loadavg = sys.load_average().unwrap();
					//TODO implementierung von Ping Senden
					nextcloud_sender
						.send(NextcloudEvent::Ping(gettext!("⚠️ Error reading buttons of board {}. Load average: {} {} {}, Memory usage: {}, Swap: {}, CPU temp: {}", board, loadavg.one, loadavg.five, loadavg.fifteen, sys.memory().unwrap().total, sys.swap().unwrap().total, sys.cpu_temp().unwrap())))
						.await?;
					Buttons::do_reset(nextcloud_sender.clone(), &mut pwr).await?;
				}
			}
			// Validation benötigt button, somit threads abhängig!!!; channel zwischen buttons und validator? damit validator nur getriggert ist wenn buttons sich ändert?
			// Validation start
			let sequence = self.sequence.to_vec();
			match validator.validate(&mut self.sequence) {
				Validation::Validated(user) => {
					self.open_door();
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!("🤗 Opened for {}", user)))
						.await?;
					let now = Local::now();
					let (sunrise, sunset) = sunrise_sunset(
						location_latitude,
						location_longitude,
						now.year(),
						now.month(),
						now.day(),
					);
					if now.timestamp() < sunrise || now.timestamp() > sunset {
						nextcloud_sender
							.send(NextcloudEvent::Licht(gettext!(
								"💡 Switch lights in and out. {}",
								self.switch_lights(true, true)
							)))
							.await?;
					} else {
						nextcloud_sender
							.send(NextcloudEvent::Licht(gettext!(
								"🕶️ Don't switch lights as its day. Now: {} Sunrise: {} Sunset: {}",
								now.timestamp(),
								sunrise,
								sunset
							)))
							.await?;
					}
				}
				Validation::Timeout => {
					if sequence != vec![0, 15] {
						self.show_wrong_input();
						self.ring_bell(20, 0);
						nextcloud_sender
							.send(NextcloudEvent::Chat(gettext!(
								"⌛ Timeout with sequence {}",
								format!("{:?}", sequence)
							)))
							.await?;
					}
				}
				Validation::SequenceTooLong => {
					self.show_wrong_input();
					self.ring_bell(20, 0);
					nextcloud_sender
						.send(NextcloudEvent::Chat(gettext!(
							"⌛ Sequence {} too long",
							format!("{:?}", sequence)
						)))
						.await?;
				}
				Validation::None => (),
			}
			interval.tick().await;
		}
	}
}

impl Drop for Buttons {
	fn drop(&mut self) {
		let _ = self.turn_everything_off();
	}
}
