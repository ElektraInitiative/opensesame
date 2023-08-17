/// Before using this module you need to configure Elektra with the following elements
/// [weatherstation/enable], [weatherstation/opensensemap/id] and [weatherstation/opensensemap/token]
/// For example:
/// kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/enable 1
/// kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/opensensemap/id "<opensensemap-box-id>"
/// kdb set user:/sw/libelektra/opensesame/#0/current/weatherstation/opensensemap/token "<access-token>"
extern crate libmodbus;

use crate::config::Config;
use libmodbus::*;
use reqwest::header::HeaderMap;
use reqwest::Client;

///Constants
const DEVICE: &'static str = "/dev/ttyS5";
const BAUDRATE: i32 = 9600;
const PARITY: char = 'N';
const DATA_BITS: i32 = 8;
const STOP_BITS: i32 = 1;
const SLAVE_ID: u8 = 1;

const ERROR_CODE_S32: u32 = 0x7FFFFFFF;
const ERROR_CODE_U32: u32 = 0xFFFFFFFF;

//Adresses of registers
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

//OpenSenseMap
const UPDATE_FREQUENCY: u32 = 0; // 1min

//Elements of tuple (opensensemap-id, reg-address, factor, datatype(signed or unsigned))
const OPENSENSE_CLIMA_DATA: [(&'static str, u16, f32, char); 36] = [
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
fn conv_vec_to_value_s(vec: Vec<u16>) -> Result<i32, ()> {
	let usign_val: u32 = (vec[0] as u32) << 16 | (vec[1] as u32);
	if usign_val == ERROR_CODE_S32 {
		Err(())
	} else {
		Ok(usign_val as i32)
	}
}

fn conv_vec_to_value_u(vec: Vec<u16>) -> Result<u32, ()> {
	let usign_val = (vec[0] as u32) << 16 | (vec[1] as u32);
	if usign_val == ERROR_CODE_U32 {
		Err(())
	} else {
		Ok(usign_val)
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum TempWarningStateChange {
	None,
	ChangeToCloseWindow,
	ChangeToWarningTempNoWind,
	ChangeToWarningTemp,
	ChangeToRemoveWarning,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TempWarning {
	None,
	RemoveWarning,
	CloseWindow,
	WarningTempNoWind,
	WarningTemp,
}

pub struct ClimaSensorUS {
	ctx: Option<Modbus>,
	opensensebox_id: String,
	opensense_access_token: String,
	warning_active: TempWarning,
	opensensemap_counter: u32,
}

impl ClimaSensorUS {
	pub fn new(config: &mut Config) -> Self {
		let mut s = Self {
			ctx: None,
			opensensebox_id: config.get::<String>("weatherstation/opensensemap/id"),
			opensense_access_token: config.get::<String>("weatherstation/opensensemap/token"),
			warning_active: TempWarning::None,
			opensensemap_counter: 0,
		};
		if config.get_bool("weatherstation/enable") {
			match s.init() {
				Ok(_) => (),
				Err(error) => {
					panic!("Error oucuured during init of modbus-connection: {}", error);
				}
			}
		}
		s
	}

	fn init(&mut self) -> Result<(), libmodbus::Error> {
		match Modbus::new_rtu(DEVICE, BAUDRATE, PARITY, DATA_BITS, STOP_BITS) {
			Ok(conn) => {
				self.ctx = Some(conn);
			}
			Err(error) => {
				return Err(error);
			}
		}

		if let Some(conn) = &mut self.ctx {
			if let Err(error) = conn.set_slave(SLAVE_ID) {
				return Err(error);
			} else if let Err(error) = conn.rtu_set_serial_mode(SerialMode::RtuRS232) {
				return Err(error);
			} else if let Err(error) = conn.rtu_set_rts(RequestToSendMode::RtuRtsUp) {
				return Err(error);
			} else if let Err(error) = conn.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp) {
				return Err(error);
			} else if let Err(error) = conn.connect() {
				return Err(error);
			}
		}
		Ok(())
	}

	/// This function should be called periodically to check the sensors' values.
	/// if temp > 30°C and no wind, then a warning should be issued
	/// if temp > 35°C a warning should be issued
	/// if temp < 20° either warning is removed
	/// (Input Register - 0x04) temp-reg address 0x76C1; typ S32; real_result = response_temp/10
	/// (Input Register - 0x04) wind-reg address 0x7533; typ U32; real_result = response_wind/10
	/// The return value is bool on success, true if alarm is active and false is alarm is not active
	/// If no ctx is configured the this function returns always false, so no warning is triggered
	pub fn handle(&mut self) -> Result<TempWarningStateChange, std::io::Error> {
		match &self.ctx {
			Some(conn) => {
				let mut response_temp = vec![0u16; 2];
				let mut response_wind = vec![0u16; 2];

				if let Err(error) = conn.read_input_registers(REG_AIR_TEMP, 2, &mut response_temp) {
					return Err(std::io::Error::new(
						std::io::ErrorKind::Other,
						error.to_string(),
					));
				}
				if let Err(error) =
					conn.read_input_registers(REG_MEAN_WIND_SPEED, 2, &mut response_wind)
				{
					return Err(std::io::Error::new(
						std::io::ErrorKind::Other,
						error.to_string(),
					));
				}

				let temp: f32 = (conv_vec_to_value_s(response_temp).unwrap() as f32) / 10.0;
				let wind: f32 = (conv_vec_to_value_u(response_wind).unwrap() as f32) / 10.0;
				#[cfg(debug_assertions)]
				println!(
					"Weatherstation: temperature {} °C, windspeed {} m/s",
					temp, wind
				);
				//check if new data should be published to opensensemap.org
				if self.opensensemap_counter == UPDATE_FREQUENCY {
					self.opensensemap_counter = 0;
					match self.publish_to_opensensemap() {
						Ok(_) => {}
						Err(error) => return Err(error),
					}
				} else {
					self.opensensemap_counter += 1;
				}

				Ok(self.set_warning_active(temp, wind))
			}
			None => Ok(TempWarningStateChange::None),
		}
	}

	/// This function is used to set the warning_active varibale and compare it with the new value.
	fn set_warning_active(&mut self, temp: f32, wind: f32) -> TempWarningStateChange {
		let new_warning: TempWarning;
		let mut result: TempWarningStateChange = TempWarningStateChange::None;

		if temp > 35.0 {
			new_warning = TempWarning::WarningTemp;
		} else if temp > 30.0
			&& wind < 0.3
			&& !matches!(self.warning_active, TempWarning::WarningTemp)
		{
			new_warning = TempWarning::WarningTempNoWind;
		} else if temp > 23.0
			&& !matches!(
				self.warning_active,
				TempWarning::WarningTemp | TempWarning::WarningTempNoWind
			) {
			new_warning = TempWarning::CloseWindow;
		} else if !matches!(
			self.warning_active,
			TempWarning::None | TempWarning::RemoveWarning
		) && temp < 20.0
		{
			new_warning = TempWarning::RemoveWarning;
		} else {
			new_warning = self.warning_active;
		}

		// compaire old and new value of TempWarning
		if self.warning_active != new_warning {
			result = match new_warning {
				TempWarning::RemoveWarning => {
					self.warning_active = new_warning;
					TempWarningStateChange::ChangeToRemoveWarning
				}
				TempWarning::CloseWindow => {
					self.warning_active = new_warning;
					TempWarningStateChange::ChangeToCloseWindow
				}
				TempWarning::WarningTempNoWind => {
					self.warning_active = new_warning;
					TempWarningStateChange::ChangeToWarningTempNoWind
				}
				TempWarning::WarningTemp => {
					self.warning_active = new_warning;
					TempWarningStateChange::ChangeToWarningTemp
				}
				TempWarning::None => {
					self.warning_active = new_warning;
					TempWarningStateChange::None
				}
			};
		}
		result
	}

	/// This methode creates a json payload out of the array `OPENSENSE_CLIMA_DATA` and the data from the weather station
	pub fn create_json(&mut self) -> Result<String, libmodbus::Error> {
		match &self.ctx {
			Some(conn) => {
				let mut json_payload: String = "[".to_string();

				for tuple_data in OPENSENSE_CLIMA_DATA.iter() {
					let mut response = vec![0u16; 2];
					match conn.read_input_registers(tuple_data.1, 2, &mut response) {
						Ok(_) => {
							let value: f32;
							if tuple_data.3 == 's' {
								match conv_vec_to_value_s(response) {
									Ok(conv_response) => {
										value = conv_response as f32 / tuple_data.2;
									}
									Err(_) => {
										value = ERROR_CODE_S32 as f32;
									}
								}
							} else {
								match conv_vec_to_value_u(response) {
									Ok(conv_response) => {
										value = conv_response as f32 / tuple_data.2;
									}
									Err(_) => {
										value = ERROR_CODE_U32 as f32;
									}
								}
							}
							if value != ERROR_CODE_S32 as f32 && value != ERROR_CODE_U32 as f32 {
								json_payload.push_str(&format!(
									"{}\"sensor\":\"{}\",\"value\":\"{}\"{},",
									'{', tuple_data.0, value, '}'
								));
							}
						}
						Err(_) => {}
					}
				}
				//remove last ','
				json_payload.pop();
				json_payload.push_str(&"]");
				Ok(json_payload)
			}
			None => Err(Error::Rtu {
				msg: ("No modbus connection configured".to_string()),
				source: (std::io::Error::new(std::io::ErrorKind::NotFound, "Not configured")),
			}),
		}
	}

	/// This function pulls data from the weatherstation and forms a json file out of the weather station data and the opensensemap-sensor-id.
	/// The created json file is send tho the opensensemap-api.
	/// All information needed are stored in a const array of tuples. The tuples contain the opensensemap-sensor-id, register-address, factor and datatype.
	/// The return value indicates if the api request was successfully or not.
	/// Information about the reading of registers can be accessed through the json_payload  
	pub fn publish_to_opensensemap(&mut self) -> Result<(), std::io::Error> {
		match self.create_json() {
			Ok(json) => {
				//Send JSON to https://api.opensensemap.org
				let mut headers = HeaderMap::new();
				headers.insert(
					"Authorization",
					self.opensense_access_token.parse().unwrap(),
				);
				headers.insert("Content-Type", "application/json".parse().unwrap());
				let result = Client::new()
					.post(&format!(
						"https://api.opensensemap.org/boxes/{}/data",
						self.opensensebox_id
					))
					.headers(headers)
					.body(json)
					.send();
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
			Err(error) => Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				error.to_string(),
			)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[ignore]
	fn test_handle() {
		let mut config: Config = Config::new("/sw/libelektra/opensesame/#0/current");
		let mut weatherstation = ClimaSensorUS::new(&mut config);

		match weatherstation.handle().unwrap() {
			TempWarningStateChange::ChangeToCloseWindow => println!("ChangeToCloseWindow"),
			TempWarningStateChange::ChangeToWarningTempNoWind => {
				println!("ChangeToWarningTempNoWind")
			}
			TempWarningStateChange::ChangeToWarningTemp => println!("ChangeToWarningTemp"),
			TempWarningStateChange::ChangeToRemoveWarning => println!("ChangeToRemoveWarning"),
			TempWarningStateChange::None => println!("None"),
		}
	}

	#[test]
	fn test_set_warning_active() {
		let mut clima_sens = ClimaSensorUS {
			ctx: None,
			opensensebox_id: "null".to_string(),
			opensense_access_token: "null".to_string(),
			warning_active: TempWarning::None,
			opensensemap_counter: 0,
		};

		assert!(clima_sens.set_warning_active(15.0, 0.1) == TempWarningStateChange::None);
		assert!(matches!(clima_sens.warning_active, TempWarning::None));
		assert!(clima_sens.set_warning_active(15.0, 3.5) == TempWarningStateChange::None);
		assert!(matches!(clima_sens.warning_active, TempWarning::None));

		assert!(
			clima_sens.set_warning_active(25.0, 0.1) == TempWarningStateChange::ChangeToCloseWindow
		);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::CloseWindow
		));
		assert!(clima_sens.set_warning_active(25.0, 3.5) == TempWarningStateChange::None);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::CloseWindow
		));

		assert!(
			clima_sens.set_warning_active(33.0, 0.1)
				== TempWarningStateChange::ChangeToWarningTempNoWind
		);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::WarningTempNoWind
		));
		assert!(clima_sens.set_warning_active(33.0, 3.5) == TempWarningStateChange::None);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::WarningTempNoWind
		));

		assert!(
			clima_sens.set_warning_active(36.0, 0.1) == TempWarningStateChange::ChangeToWarningTemp
		);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::WarningTemp
		));
		assert!(clima_sens.set_warning_active(36.0, 3.5) == TempWarningStateChange::None);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::WarningTemp
		));
		assert!(clima_sens.set_warning_active(25.3, 3.4) == TempWarningStateChange::None);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::WarningTemp
		));
		assert!(
			clima_sens.set_warning_active(15.0, 0.1)
				== TempWarningStateChange::ChangeToRemoveWarning
		);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::RemoveWarning
		));
		assert!(clima_sens.set_warning_active(15.0, 3.5) == TempWarningStateChange::None);
		assert!(matches!(
			clima_sens.warning_active,
			TempWarning::RemoveWarning
		));
	}

	#[test]
	fn test_conv_vec_to_value_s() {
		assert_eq!(conv_vec_to_value_s(vec![0x0000u16, 0x0000u16]), Ok(0));
		assert_eq!(conv_vec_to_value_s(vec![0x0000u16, 0x0001u16]), Ok(1));
		assert_eq!(conv_vec_to_value_s(vec![0xffffu16, 0xffffu16]), Ok(-1));
		assert_eq!(conv_vec_to_value_s(vec![0x0000u16, 0x000au16]), Ok(10));
		assert_eq!(conv_vec_to_value_s(vec![0xffffu16, 0xfff6u16]), Ok(-10));
		assert_eq!(conv_vec_to_value_s(vec![0x0000u16, 0x0020u16]), Ok(32));
		assert_eq!(conv_vec_to_value_s(vec![0xffffu16, 0xffe0u16]), Ok(-32));
		assert_eq!(conv_vec_to_value_s(vec![0x0000u16, 0x1524u16]), Ok(5412));
		assert_eq!(conv_vec_to_value_s(vec![0xffffu16, 0xeadcu16]), Ok(-5412));
		assert_eq!(conv_vec_to_value_s(vec![0x7fffu16, 0xffffu16]), Err(()));
		assert_eq!(
			conv_vec_to_value_s(vec![0x8000u16, 0x0000u16]),
			Ok(-2147483648)
		);
	}

	#[test]
	fn test_conv_vec_to_value_u() {
		assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0000u16]), Ok(0));
		assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0001u16]), Ok(1));
		assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x000au16]), Ok(10));
		assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0020u16]), Ok(32));
		assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x1524u16]), Ok(5412));
		assert_eq!(conv_vec_to_value_u(vec![0xffffu16, 0xffffu16]), Err(()));
	}
}
