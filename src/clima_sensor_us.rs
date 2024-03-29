extern crate libmodbus;

use crate::config::Config;
use crate::nextcloud::{NextcloudChat, NextcloudEvent};
use crate::types::ModuleError;
use futures::never::Never;
use gettextrs::gettext;
use libmodbus::{Modbus, ModbusClient, ModbusRTU, RequestToSendMode, SerialMode};
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Serialize;
use std::io;
use systemstat::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

///Constants
const DEVICE: &str = "/dev/ttyS5";
const BAUDRATE: i32 = 9600;
const PARITY: char = 'N';
const DATA_BITS: i32 = 8;
const STOP_BITS: i32 = 1;
const SLAVE_ID: u8 = 1;

const ERROR_CODE_S32: u32 = 0x7FFFFFFF;
const ERROR_CODE_U32: u32 = 0xFFFFFFFF;

//Addresses of registers
const REG_MEAN_WIND_SPEED: u16 = 0x88B9;
const REG_MEAN_WIND_DIREC: u16 = 0x88BB;
const REG_AIR_TEMP: u16 = 0x88BD;
const REG_FRAME_TEMP: u16 = 0x88BF;
const REG_ACOUSTIC_TEMP: u16 = 0x88C1;
const REG_AIR_TEMP_UNCORRECTED: u16 = 0x88C3;
const REG_REL_HUMIDITY: u16 = 0x88C5;
const REG_DEW_POINT_TEMP: u16 = 0x88C7;
const REG_ABS_AIR_PRESSURE: u16 = 0x88C9;
const REG_REL_AIR_PRESSURE: u16 = 0x88CB;
const REG_BRIGHTNESS_N: u16 = 0x88CD;
const REG_BRIGHTNESS_E: u16 = 0x88CF;
const REG_BRIGHTNESS_S: u16 = 0x88D1;
const REG_BRIGHTNESS_W: u16 = 0x88D3;
const REG_DIREC_BRIGHTNESS: u16 = 0x88D5;
const REG_BRIGHTNESS_MAX: u16 = 0x88D7;
const REG_PRECIPITATION_EVENT: u16 = 0x88D9;
const REG_PRECIPITATION_INTENSITY: u16 = 0x88DB;
const REG_PRECIPITATION_AMOUNT: u16 = 0x88DD;
const REG_PRECIPITATION_TYPE: u16 = 0x88DF;
const REG_SUN_ELEVATION: u16 = 0x88E9;
const REG_SUN_AZIMUTH: u16 = 0x88EB;
const REG_HEIGHT_ABOVE_SEA: u16 = 0x88ED;
const REG_SENSOR_SUPPLY: u16 = 0x88F1;
const REG_MAX_WIND_SPEED: u16 = 0x8901;
const REG_WIND_DIREC: u16 = 0x8903;
const REG_ABS_HUMIDITY: u16 = 0x8905;
const REG_REL_HUMIDITY_UNCORRECTED: u16 = 0x8907;
const REG_MAGNETIC_COMPASS_DIFF_ANGLE: u16 = 0x8909;
const REG_BRIGHTNESS_VEC_SUM: u16 = 0x890B;
const REG_WINDCHILL_TEMP: u16 = 0x890D;
const REG_HEAT_INDEX_TEMP: u16 = 0x890F;
const REG_ABS_PRECIPITATION_AMOUNT: u16 = 0x8911;
const REG_GLOBAL_RADIATION: u16 = 0x8913;
const REG_PITCH_MAGNETIC_COMPASS_NS: u16 = 0x8915;
const REG_ROLL_MAGNETIC_COMPASS_EW: u16 = 0x8917;

//Elements of tuple (opensensemap-id, reg-address, factor, datatype(signed or unsigned))
const OPENSENSE_CLIMA_DATA: [(&str, u16, f32, char); 36] = [
	("64cb602193c69500072a5813", REG_MEAN_WIND_SPEED, 10.0, 'u'),
	("64cb7c21d588b90007d69a5f", REG_MEAN_WIND_DIREC, 10.0, 'u'),
	("64cb7c21d588b90007d69a60", REG_AIR_TEMP, 10.0, 's'),
	("64cb7c21d588b90007d69a61", REG_FRAME_TEMP, 10.0, 's'),
	("64cb7c21d588b90007d69a62", REG_ACOUSTIC_TEMP, 10.0, 's'),
	(
		"64cb7c21d588b90007d69a63",
		REG_AIR_TEMP_UNCORRECTED,
		10.0,
		's',
	),
	("64cb7c21d588b90007d69a64", REG_REL_HUMIDITY, 10.0, 'u'),
	("64cb7c21d588b90007d69a65", REG_DEW_POINT_TEMP, 10.0, 's'),
	("64cb7c21d588b90007d69a66", REG_ABS_AIR_PRESSURE, 100.0, 'u'),
	("64cb7c21d588b90007d69a67", REG_REL_AIR_PRESSURE, 100.0, 'u'),
	("64cb7c21d588b90007d69a68", REG_BRIGHTNESS_N, 10.0, 'u'),
	("64cb7c21d588b90007d69a69", REG_BRIGHTNESS_E, 10.0, 'u'),
	("64cb7c21d588b90007d69a6a", REG_BRIGHTNESS_S, 10.0, 'u'),
	("64cb7c21d588b90007d69a6b", REG_BRIGHTNESS_W, 10.0, 'u'),
	("64cb7cfdd588b90007d702d6", REG_DIREC_BRIGHTNESS, 1.0, 'u'),
	("64cb7cfdd588b90007d702d7", REG_BRIGHTNESS_MAX, 10.0, 'u'),
	(
		"64cb7cfdd588b90007d702d8",
		REG_PRECIPITATION_EVENT,
		1.0,
		'u',
	),
	(
		"64cb7cfdd588b90007d702d9",
		REG_PRECIPITATION_INTENSITY,
		1000.0,
		'u',
	),
	(
		"64cb7cfdd588b90007d702da",
		REG_PRECIPITATION_AMOUNT,
		1000.0,
		'u',
	),
	("64cb7cfdd588b90007d702db", REG_PRECIPITATION_TYPE, 1.0, 'u'),
	("64cb7d79d588b90007d7402e", REG_SUN_ELEVATION, 10.0, 's'),
	("64cb7d79d588b90007d7402f", REG_SUN_AZIMUTH, 10.0, 's'),
	("64cb7d79d588b90007d74030", REG_HEIGHT_ABOVE_SEA, 1.0, 's'),
	("64cb7d79d588b90007d74031", REG_SENSOR_SUPPLY, 10.0, 'u'),
	("64cb7dfbd588b90007d782fc", REG_MAX_WIND_SPEED, 10.0, 'u'),
	("64cb7dfbd588b90007d782fd", REG_WIND_DIREC, 10.0, 'u'),
	("64cb7dfbd588b90007d782fe", REG_ABS_HUMIDITY, 100.0, 'u'),
	(
		"64cb7dfbd588b90007d782ff",
		REG_REL_HUMIDITY_UNCORRECTED,
		10.0,
		'u',
	),
	(
		"64cb7eb2d588b90007d7dd96",
		REG_MAGNETIC_COMPASS_DIFF_ANGLE,
		10.0,
		'u',
	),
	("64cb7eb2d588b90007d7dd97", REG_BRIGHTNESS_VEC_SUM, 1.0, 'u'),
	("64cb7eb2d588b90007d7dd98", REG_WINDCHILL_TEMP, 10.0, 's'),
	("64cb7eb2d588b90007d7dd99", REG_HEAT_INDEX_TEMP, 10.0, 's'),
	(
		"64cb7eb2d588b90007d7dd9a",
		REG_ABS_PRECIPITATION_AMOUNT,
		1000.0,
		'u',
	),
	("64cb7eb2d588b90007d7dd9b", REG_GLOBAL_RADIATION, 10.0, 's'),
	(
		"64cb7eb2d588b90007d7dd9c",
		REG_PITCH_MAGNETIC_COMPASS_NS,
		10.0,
		's',
	),
	(
		"64cb7eb2d588b90007d7dd9d",
		REG_ROLL_MAGNETIC_COMPASS_EW,
		10.0,
		's',
	),
];

/// These functions create a single number out of a vector.
/// The first entry in the vector are the most significant bytes and the second entry are the least significant bytes.
/// For the unsigned function the input `vec` should be already in two complement, so that the function works right.
fn conv_vec_to_value_s(vec: (u16, u16)) -> Result<i32, ()> {
	let usign_val: u32 = (vec.0 as u32) << 16 | (vec.1 as u32);
	if usign_val == ERROR_CODE_S32 {
		Err(())
	} else {
		Ok(usign_val as i32)
	}
}

fn conv_vec_to_value_u(vec: (u16, u16)) -> Result<u32, ()> {
	let usign_val = (vec.0 as u32) << 16 | (vec.1 as u32);
	if usign_val == ERROR_CODE_U32 {
		Err(())
	} else {
		Ok(usign_val)
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Warning {
	CloseWindow,
	HighTemp,
	LowTemp,
	StrongWind,
	ErrorTemp,
	ErrorWind,
	ErrorBoth,
	None,
}

pub struct ClimaSensorUS {
	ctx: Modbus,
	opensensebox_id: String,
	warning_active: Warning,
	client: Client,
	headers: HeaderMap,
}

#[derive(Serialize)]
struct SensorValue {
	sensor: &'static str,
	value: f32,
}

unsafe impl Send for ClimaSensorUS {}

impl ClimaSensorUS {
	// Temperature
	pub const LOW_CANCEL_TEMP: f32 = 5.0;
	pub const HIGH_CANCEL_TEMP: f32 = 20.0;
	pub const CLOSE_WINDOW_TEMP: f32 = 22.0;
	pub const NO_WIND_TEMP: f32 = 30.0;
	pub const NO_WIND_SPEED: f32 = 0.3;
	pub const HIGH_WARNING_TEMP: f32 = 35.0;
	pub const LOW_WARNING_TEMP: f32 = 0.0;
	pub const STRONG_WIND_SPEED: f32 = 10.8;
	pub const OK_WIND_SPEED: f32 = 8.0;

	pub fn new(config: &mut Config) -> Result<Self, libmodbus::Error> {
		let opensensebox_id = config.get::<String>("weatherstation/opensensemap/id");
		let opensense_access_token = config.get::<String>("weatherstation/opensensemap/token");
		let warning_active = Warning::None;

		let client = Client::new();

		let mut headers = HeaderMap::new();
		headers.insert("Authorization", opensense_access_token.parse().unwrap());
		headers.insert("Content-Type", "application/json".parse().unwrap());

		let mut modbus = Modbus::new_rtu(DEVICE, BAUDRATE, PARITY, DATA_BITS, STOP_BITS)?;

		modbus.set_slave(SLAVE_ID)?;
		modbus.rtu_set_serial_mode(SerialMode::RtuRS232)?;
		modbus.rtu_set_rts(RequestToSendMode::RtuRtsUp)?;
		modbus.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp)?;
		modbus.connect()?;

		Ok(Self {
			ctx: modbus,
			opensensebox_id,
			warning_active,
			client,
			headers,
		})
	}

	/// This function should be called periodically to check the sensors' values.
	///
	/// (Input Register - 0x04) temp-reg address 0x76C1; typ S32; real_result = response_temp/10
	/// (Input Register - 0x04) wind-reg address 0x7533; typ U32; real_result = response_wind/10
	/// The return value is bool on success, true if alarm is active and false is alarm is not active
	/// If no ctx is configured the this function returns always false, so no warning is triggered
	async fn handle(&mut self) -> Result<Option<String>, libmodbus::Error> {
		let mut response_temp = vec![0u16; 2];
		let mut response_wind = vec![0u16; 2];
		let temp: f32;
		let wind: f32;

		self.ctx
			.read_input_registers(REG_AIR_TEMP, 2, &mut response_temp)?;
		self.ctx
			.read_input_registers(REG_MEAN_WIND_SPEED, 2, &mut response_wind)?;

		match conv_vec_to_value_s((response_temp[0], response_temp[1])) {
			Ok(conv_response) => {
				temp = conv_response as f32 / 10.0;
			}
			Err(_) => {
				temp = ERROR_CODE_S32 as f32;
			}
		}

		match conv_vec_to_value_u((response_wind[0], response_wind[1])) {
			Ok(conv_response) => {
				wind = conv_response as f32 / 10.0;
			}
			Err(_) => {
				wind = ERROR_CODE_S32 as f32;
			}
		}

		//check if new data should be published to opensensemap.org
		match self.publish_to_opensensemap().await {
			Ok(_) => {}
			// TODO
			Err(error) => return Err(libmodbus::Error::IoError(error)),
		}

		Ok(ClimaSensorUS::set_warning_active(
			&mut self.warning_active,
			temp,
			wind,
		))
	}

	/// This function is used to set the warning_active variable and compare it with the new value.
	fn set_warning_active(warning_active: &mut Warning, temp: f32, wind: f32) -> Option<String> {
		let new_warning;

		if temp == ERROR_CODE_S32 as f32 && wind == ERROR_CODE_U32 as f32 {
			new_warning = Warning::ErrorBoth;
		} else if temp == ERROR_CODE_S32 as f32 {
			new_warning = Warning::ErrorTemp;
		} else if wind == ERROR_CODE_U32 as f32 {
			new_warning = Warning::ErrorWind;
		} else {
			if temp > ClimaSensorUS::LOW_CANCEL_TEMP
				&& temp < ClimaSensorUS::HIGH_CANCEL_TEMP
				&& wind < ClimaSensorUS::OK_WIND_SPEED
			{
				new_warning = Warning::None;
			} else if wind > ClimaSensorUS::STRONG_WIND_SPEED {
				new_warning = Warning::StrongWind;
			} else if temp > ClimaSensorUS::HIGH_WARNING_TEMP {
				new_warning = Warning::HighTemp;
			} else if temp < ClimaSensorUS::LOW_WARNING_TEMP {
				new_warning = Warning::LowTemp;
			} else if temp > ClimaSensorUS::NO_WIND_TEMP
				&& wind < ClimaSensorUS::NO_WIND_SPEED
				&& !matches!(warning_active, Warning::HighTemp)
			{
				new_warning = Warning::HighTemp;
			} else if temp >= ClimaSensorUS::CLOSE_WINDOW_TEMP
				&& !matches!(warning_active, Warning::LowTemp | Warning::HighTemp)
			{
				new_warning = Warning::CloseWindow;
			} else {
				new_warning = *warning_active;
			}
		}

		// compare old and new value of Warning
		if *warning_active != new_warning {
			*warning_active = new_warning;
			Some(match new_warning {
				Warning::CloseWindow => gettext!(
					"🌡️ High Temperature {} °C, close the window (Wind {} m/s)",
					temp,
					wind
				),
				Warning::HighTemp => {
					gettext!(
						"🌡️ Heat Alert {} °C, turn on PV cooling (Wind {} m/s)",
						temp,
						wind
					)
				}
				Warning::LowTemp => {
					gettext!(
						"🌡 Freezing Temperature {} °C, yield is in danger (Wind {} m/s)",
						temp,
						wind
					)
				}
				Warning::StrongWind => {
					gettext!("༄ Strong Wind {} m/s (Temperature: {} °C)", wind, temp)
				}
				Warning::ErrorTemp => {
					gettext!("⚠️ Error in temperature measurement (Wind {} m/s)", wind)
				}
				Warning::ErrorWind => {
					gettext!("⚠️ Error in wind measurement (Temperature: {} °C)", temp)
				}
				Warning::ErrorBoth => {
					gettext("⚠️⚠️ Error in temperature measurement and wind measurement")
				}
				Warning::None => {
					gettext!("🌡 ༄ Temperature {} °C and Wind {} m/s are moderate again, no warning present",
							temp, wind)
				}
			})
		} else {
			Option::None
		}
	}

	/// This method creates a json payload out of the array `OPENSENSE_CLIMA_DATA` and the data from the weather station
	async fn collect_sensor_values(&mut self) -> Vec<SensorValue> {
		let mut sensor_values = vec![];

		for tuple_data in OPENSENSE_CLIMA_DATA.iter() {
			let mut response = vec![0u16; 2];

			// Leave out sensor with error
			if self
				.ctx
				.read_input_registers(tuple_data.1, 2, &mut response)
				.is_ok()
			{
				let value: f32;
				if tuple_data.3 == 's' {
					match conv_vec_to_value_s((response[0], response[1])) {
						Ok(conv_response) => {
							value = conv_response as f32 / tuple_data.2;
						}
						Err(_) => {
							value = ERROR_CODE_S32 as f32;
						}
					}
				} else {
					match conv_vec_to_value_u((response[0], response[1])) {
						Ok(conv_response) => {
							value = conv_response as f32 / tuple_data.2;
						}
						Err(_) => {
							value = ERROR_CODE_U32 as f32;
						}
					}
				}
				if value != ERROR_CODE_S32 as f32 && value != ERROR_CODE_U32 as f32 {
					sensor_values.push(SensorValue {
						sensor: tuple_data.0,
						value,
					});
				}
			}
		}
		sensor_values
	}

	/// This function pulls data from the weatherstation and forms a json file out of the weather station data and the opensensemap-sensor-id.
	/// The created json file is send tho the opensensemap-api.
	/// All information needed are stored in a const array of tuples. The tuples contain the opensensemap-sensor-id, register-address, factor and datatype.
	/// The return value indicates if the api request was successfully or not.
	/// Information about the reading of registers can be accessed through the json_payload  
	async fn publish_to_opensensemap(&mut self) -> Result<(), io::Error> {
		let json = serde_json::to_string(&self.collect_sensor_values().await).unwrap();

		//Send JSON to https://api.opensensemap.org
		let result = self
			.client
			.post(&format!(
				"https://api.opensensemap.org/boxes/{}/data",
				self.opensensebox_id
			))
			.headers(self.headers.clone())
			.body(json)
			.send()
			.await;
		match result {
			Ok(response) => match response.error_for_status() {
				Ok(_response) => Ok(()),
				Err(error) => Err(std::io::Error::new(
					std::io::ErrorKind::Other,
					error.to_string(),
				)),
			},
			Err(error) => Err(std::io::Error::new(
				std::io::ErrorKind::ConnectionRefused,
				error.to_string(),
			)),
		}
	}

	pub async fn get_background_task(
		mut self,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut interval = interval(Duration::from_secs(60));
		loop {
			match self.handle().await {
				Ok(Some(message)) => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(NextcloudChat::Default, message))
						.await?;
				}
				Ok(None) => (),
				Err(error) => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Ping,
							gettext!("⚠️ Error from weather station: {}", error),
						))
						.await?;
				}
			};
			interval.tick().await;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_set_warning_active() {
		let mut warning_active = Warning::None;

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 0.1).is_none());
		assert_eq!(warning_active, Warning::None);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 3.5).is_none());
		assert_eq!(warning_active, Warning::None);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 25.0, 0.1).is_some());
		assert_eq!(warning_active, Warning::CloseWindow);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 25.0, 3.5).is_none());
		assert_eq!(warning_active, Warning::CloseWindow);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 33.0, 0.1).is_some());
		assert_eq!(warning_active, Warning::HighTemp);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 33.0, 3.5).is_none());
		assert_eq!(warning_active, Warning::HighTemp);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 36.0, 0.1).is_none());
		assert_eq!(warning_active, Warning::HighTemp);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 36.0, 3.5).is_none());
		assert_eq!(warning_active, Warning::HighTemp);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 25.3, 3.4).is_none());
		assert_eq!(warning_active, Warning::HighTemp);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 0.1).is_some());
		assert_eq!(warning_active, Warning::None);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 3.5).is_none());
		assert_eq!(warning_active, Warning::None);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 20.5).is_some());
		assert_eq!(warning_active, Warning::StrongWind);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 13.5).is_none());
		assert_eq!(warning_active, Warning::StrongWind);

		assert!(ClimaSensorUS::set_warning_active(&mut warning_active, 15.0, 5.5).is_some());
		assert_eq!(warning_active, Warning::None);
	}

	#[test]
	fn test_conv_vec_to_value_s() {
		assert_eq!(conv_vec_to_value_s((0x0000u16, 0x0000u16)), Ok(0));
		assert_eq!(conv_vec_to_value_s((0x0000u16, 0x0001u16)), Ok(1));
		assert_eq!(conv_vec_to_value_s((0xffffu16, 0xffffu16)), Ok(-1));
		assert_eq!(conv_vec_to_value_s((0x0000u16, 0x000au16)), Ok(10));
		assert_eq!(conv_vec_to_value_s((0xffffu16, 0xfff6u16)), Ok(-10));
		assert_eq!(conv_vec_to_value_s((0x0000u16, 0x0020u16)), Ok(32));
		assert_eq!(conv_vec_to_value_s((0xffffu16, 0xffe0u16)), Ok(-32));
		assert_eq!(conv_vec_to_value_s((0x0000u16, 0x1524u16)), Ok(5412));
		assert_eq!(conv_vec_to_value_s((0xffffu16, 0xeadcu16)), Ok(-5412));
		assert_eq!(conv_vec_to_value_s((0x7fffu16, 0xffffu16)), Err(()));
		assert_eq!(conv_vec_to_value_s((0x8000u16, 0x0000u16)), Ok(-2147483648));
	}

	#[test]
	fn test_conv_vec_to_value_u() {
		assert_eq!(conv_vec_to_value_u((0x0000u16, 0x0000u16)), Ok(0));
		assert_eq!(conv_vec_to_value_u((0x0000u16, 0x0001u16)), Ok(1));
		assert_eq!(conv_vec_to_value_u((0x0000u16, 0x000au16)), Ok(10));
		assert_eq!(conv_vec_to_value_u((0x0000u16, 0x0020u16)), Ok(32));
		assert_eq!(conv_vec_to_value_u((0x0000u16, 0x1524u16)), Ok(5412));
		assert_eq!(conv_vec_to_value_u((0xffffu16, 0xffffu16)), Err(()));
	}
}
