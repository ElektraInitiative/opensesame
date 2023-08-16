use crate::config::Config;

use std::fmt;

use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;

use linux_embedded_hal::{Delay, I2cdev};

use bme280::i2c::BME280;

pub struct Environment {
	pub co2: u16,
	pub voc: u16,
	pub temperature: f32,
	pub pressure: f32,
	pub humidity: f32,
	pub status: u8,
	pub error: u8,
	pub air_quality: AirQualityChange,

	pub boot_version: u16,
	pub app_version: u16,

	data: Vec<u8>,
	read_counter: u16,
	board5a: Option<LinuxI2CDevice>,
	bme280: Option<BME280<I2cdev, Delay>>,
	first_time: bool,
	data_interval: u16,
	pub baseline: u16,
	pub name: String,
}

const LOW_CO2_OK_QUALITY: u16 = 3000;
const HIGH_CO2_OK_QUALITY: u16 = 3500;

const LOW_CO2_BAD_QUALITY: u16 = 4000;
const HIGH_CO2_BAD_QUALITY: u16 = 4500;

const VOC_LOW_QUALITY: u16 = 9000;
const VOC_BELL_QUALITY: u16 = 11000;
const VOC_ALARM_QUALITY: u16 = 13000;

// board 5A

const BOARD5A: u16 = 0x5A;

const SW_RESET: u8 = 0xFF;

const FW_BOOT_VERSION: u8 = 0x23;
const FW_APP_VERSION: u8 = 0x24;

const MEAS_MODE: u8 = 0x01;
const MEAS_MODE_DATA: u8 = 0b011 << 4; // Low power pulse heating mode IAQ measurement every 60 seconds

const BASELINE: u8 = 0x11; // for read and write

const RESET_INTERVAL: u16 = 5; // until sensor is reachable again
const ENV_DATA: u8 = 0x05;

const ERROR_ID: u8 = 0xe0;

const STATUS: u8 = 0x00;
const APP_START: u8 = 0xF4;

const ALG_RESULT_DATA: u8 = 0x02;
const ALG_RESULT_LENGTH: u8 = 5;

// const WRITE_REG_INVALID: u8 = 0x01 << 0; // The CCS811 received an I2C write request addressed to this station but with invalid register address ID
// const READ_REG_INVALID: u8 = 0x01 << 1; // The CCS811 received an I2C read request to a mailbox ID that is invalid
// const MEASMODE_INVALID: u8 = 0x01 << 2; // The CCS811 received an I2C request to write an unsupported mode to MEAS_MODE
// const MAX_RESISTANCE: u8 = 0x01 << 3; // The sensor resistance measurement has reached or exceeded the maximum range
// const HEATER_FAULT: u8 = 0x01 << 4; // The Heater current in the CCS811 is not in range
// const HEATER_SUPPLY: u8 = 0x01 << 5; // The Heater voltage is not being applied correctly

// board 77

// const BOARD77: u16 = 0x77;

#[derive(PartialEq, Debug)]
pub enum AirQualityChange {
	Error,

	Ok,
	Moderate,
	Bad,

	FireChat,
	FireBell,
	FireAlarm,
}

fn set_env_data_ccs811(board5a: &mut LinuxI2CDevice, temperature: f32, humidity: f32) {
	let (temp_conv, hum_conv) = Environment::convert_env_data(temperature, humidity);

	board5a
		.smbus_write_i2c_block_data(
			ENV_DATA,
			&[
				((hum_conv >> 8) & 0xFF) as u8,
				(hum_conv & 0xFF) as u8,
				((temp_conv >> 8) & 0xFF) as u8,
				(temp_conv & 0xFF) as u8,
			],
		)
		.unwrap();
}

impl Environment {
	pub fn new(config: &mut Config) -> Self {
		let dev_name = config.get::<String>("environment/device");
		if dev_name == "/dev/null" {
			Self {
				co2: 0,
				voc: 0,
				temperature: 0f32,
				pressure: 0f32,
				humidity: 0f32,
				status: 0,
				error: 0,
				air_quality: AirQualityChange::Ok,
				data: Vec::new(),
				read_counter: 0,
				app_version: 0,
				boot_version: 0,
				board5a: None,
				bme280: None,
				first_time: true,
				data_interval: 0,
				baseline: 0,
				name: config.get::<String>("environment/name"),
			}
		} else {
			let i2c_bus = I2cdev::new(dev_name).unwrap();
			let mut s = Self {
				co2: 0,
				voc: 0,
				temperature: 0f32,
				pressure: 0f32,
				humidity: 0f32,
				status: 0,
				error: 0,
				air_quality: AirQualityChange::Ok,
				data: Vec::new(),
				read_counter: 0,
				app_version: 0,
				boot_version: 0,
				board5a: Some(
					LinuxI2CDevice::new(config.get::<String>("environment/device"), BOARD5A)
						.unwrap(),
				),
				bme280: Some(BME280::new_secondary(i2c_bus, Delay)),
				first_time: true,
				data_interval: config.get::<u16>("environment/data/interval"),
				baseline: 0,
				name: config.get::<String>("environment/name"),
			};
			//if sending Reset failes it disables ccs811
			match s
				.board5a
				.as_mut()
				.unwrap()
				.smbus_write_i2c_block_data(SW_RESET, &[0x11, 0xE5, 0x72, 0x8A])
			{
				Ok(_) => (),
				Err(_) => {
					s.board5a = None;
				}
			}
			s.bme280.as_mut().unwrap().init().unwrap();
			s
		}
	}

	fn calculate_air_quality(&mut self) -> bool {
		self.co2 = self.data[0].into();
		self.co2 <<= 8;
		let co2_high_byte: u16 = self.data[1].into();
		self.co2 |= co2_high_byte;

		self.voc = self.data[2].into();
		self.voc <<= 8;
		let voc_high_byte: u16 = self.data[3].into();
		self.voc |= voc_high_byte;

		let mut is_changed = true;

		// rising alarm levels
		if self.air_quality != AirQualityChange::FireAlarm && self.voc >= VOC_ALARM_QUALITY {
			self.air_quality = AirQualityChange::FireAlarm;
		} else if self.air_quality != AirQualityChange::FireBell && self.voc >= VOC_BELL_QUALITY {
			self.air_quality = AirQualityChange::FireBell;
		} else if self.air_quality != AirQualityChange::FireChat && self.voc >= VOC_LOW_QUALITY {
			self.air_quality = AirQualityChange::FireChat;

		// becomes Ok
		} else if self.air_quality != AirQualityChange::Ok && self.co2 < LOW_CO2_OK_QUALITY {
			self.air_quality = AirQualityChange::Ok;

		// rising
		} else if self.air_quality == AirQualityChange::Ok && self.co2 > HIGH_CO2_OK_QUALITY {
			self.air_quality = AirQualityChange::Moderate;
		} else if self.air_quality == AirQualityChange::Moderate && self.co2 > HIGH_CO2_BAD_QUALITY
		{
			self.air_quality = AirQualityChange::Bad;

		// descending
		} else if self.air_quality == AirQualityChange::Bad && self.co2 < LOW_CO2_BAD_QUALITY {
			self.air_quality = AirQualityChange::Moderate;
		} else {
			is_changed = false;
		}
		return is_changed;
	}

	/// go back to remembered state
	pub fn restore_baseline(&mut self, state: &mut Config) {
		match self.board5a.as_mut() {
			None => (),
			Some(board5a) => {
				if let Some(baseline) = state.get_option::<u16>("environment/baseline") {
					board5a.smbus_write_word_data(BASELINE, baseline).unwrap();
				}
			}
		}
	}

	/// remember for later
	pub fn remember_baseline(&mut self, state: &mut Config) {
		state.set("environment/baseline", &self.baseline.to_string());
	}

	fn print_values(&self) -> String {
		return format!("Temperature: {} °C, CO₂: {} ppm, VOC: {} ppb, Humidity: {} %, Pressure {} pascals, Baseline: {}", self.temperature, self.co2, self.voc, self.humidity, self.pressure, self.baseline);
	}

	fn convert_env_data(temperature: f32, humidity: f32) -> (u16, u16) {
		/* Humidity is stored as an unsigned 16 bits in 1/512%RH. The
		default value is 50% = 0x64, 0x00. As an example 48.5%
		humidity would be 0x61, 0x00.*/

		/* Temperature is stored as an unsigned 16 bits integer in 1/512
		degrees; there is an offset: 0 maps to -25°C. The default value is
		25°C = 0x64, 0x00. As an example 23.5% temperature would be
		0x61, 0x00.
		The internal algorithm uses these values (or default values if
		not set by the application) to compensate for changes in
		relative humidity and ambient temperature.*/

		return (
			((temperature + 25.0f32) * 512.0f32 + 0.5f32) as u16,
			(humidity * 512.0f32 + 0.5f32) as u16,
		);
	}

	/// to be periodically called every 10 ms
	pub fn handle(&mut self) -> bool {
		match self.board5a.as_mut() {
			None => false,
			Some(board5a) => {
				self.read_counter += 1;

				// check if we get new data
				if !self.first_time && self.read_counter == self.data_interval {
					self.read_counter = 0;

					let measurement = self.bme280.as_mut().unwrap().measure().unwrap();
					set_env_data_ccs811(board5a, measurement.temperature, measurement.humidity);
					self.temperature = measurement.temperature;
					self.humidity = measurement.humidity;
					self.pressure = measurement.pressure;

					let data = board5a
						.smbus_read_i2c_block_data(ALG_RESULT_DATA, ALG_RESULT_LENGTH)
						.unwrap();

					if data.len() >= 4 {
						self.status = data[4];
					}

					if data.len() < 4 || self.status & 0b11110001 != 0b10010000 {
						self.error = board5a.smbus_read_byte_data(ERROR_ID).unwrap();
						self.air_quality = AirQualityChange::Error;
						return true;
					}

					self.baseline = board5a.smbus_read_word_data(BASELINE).unwrap();

					if data == self.data {
						// nothing changed, no error
						return false;
					}

					self.data = data;
					return self.calculate_air_quality();
				}

				if self.read_counter == RESET_INTERVAL && self.first_time {
					board5a.smbus_write_byte(APP_START).unwrap();
					board5a
						.smbus_write_byte_data(MEAS_MODE, MEAS_MODE_DATA)
						.unwrap();

					self.boot_version = board5a.smbus_read_word_data(FW_BOOT_VERSION).unwrap();
					self.app_version = board5a.smbus_read_word_data(FW_APP_VERSION).unwrap();

					self.status = board5a.smbus_read_byte_data(STATUS).unwrap();
					self.error = board5a.smbus_read_byte_data(ERROR_ID).unwrap();
					self.first_time = false;
					self.read_counter = 0;
					return false;
				}

				return false;
			}
		}
	}
}

impl fmt::Display for Environment {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}. {}", self.name, self.print_values())
	}
}

#[cfg(test)]
mod tests {
	// Note this useful idiom: importing names from outer (for mod tests) scope.
	use super::*;

	#[test]
	fn test_validate() {
		assert_eq!(
			(0, 0x6400),
			Environment::convert_env_data(-25.0f32, 50.0f32)
		);
		assert_eq!(
			(0, 0x6100),
			Environment::convert_env_data(-25.0f32, 48.5f32)
		);
		assert_eq!(
			(0x6400, 0x6100),
			Environment::convert_env_data(25.0f32, 48.5f32)
		);
		assert_eq!(
			(0x6100, 0x6100),
			Environment::convert_env_data(23.5f32, 48.5f32)
		);
	}
}
