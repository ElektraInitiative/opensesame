/// This module implements the functions "ambient temperature" and "object temperature" of the [MOD-IR-TEMP](https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware) sensor.
/// Before using this module, you need to configure it with Elektra. The following configurations are necessary: [ir/enable], [ir/device], and [ir/data/interval].
/// A description of these configuration parameters can be found in the file "opensesame.spec".
/// You can also modify the 'THRESHOLD_AMBIENT' and 'THRESHOLD_OBJECT' values. These two thresholds trigger the IrTempStateChange.
/// For instance, if 'THRESHOLD_AMBIENT' < 'ambient_temp', then 'ChangedToAmbientTooHot' is triggered.
use crate::config::Config;
use i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::{Delay, I2cdev};
use mlx9061x::ic::Mlx90614;
use mlx9061x::{Error, Mlx9061x, SlaveAddr};

const THRESHOLD_AMBIENT: f32 = 22.0;
const THRESHOLD_OBJECT: f32 = 44.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IrTempStateChange {
	None,
	ChangedToAmbientToHot,
	ChangedToObjectToHot,
	ChanedToBothToHot,
	ChangedToCancelled,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IrTempState {
	Normal,
	TooHot,
}

pub struct ModIR {
	mlx: Option<Mlx9061x<I2cdev, Mlx90614>>,
	device: String,
	addr: SlaveAddr,
	pub ambient_temp: f32,
	pub object_temp: f32,
	_emissivity: f32,
	active_ambient_state: IrTempState,
	active_object_state: IrTempState,
}

impl ModIR {
	/// This function is used to initialize a default instance.
	pub fn new_default() -> Self {
		Self {
			mlx: None,
			device: String::from("/dev/null"),
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			_emissivity: 1.0,
			active_ambient_state: IrTempState::Normal,
			active_object_state: IrTempState::Normal,
		}
	}

	/// This function initializes the MOD-IR-TEMP and returns an instance of ModIR upon success.
	/// In case of an error, the error code is returned.
	pub fn new(config: &mut Config) -> Result<Self, Error<LinuxI2CError>> {
		let mut s: Self;
		if config.get_bool("ir/enable") {
			s = Self {
				mlx: None,
				device: config.get::<String>("ir/device"),
				addr: SlaveAddr::Default,
				ambient_temp: 0.0,
				object_temp: 0.0,
				_emissivity: 1.0,
				active_ambient_state: IrTempState::Normal,
				active_object_state: IrTempState::Normal,
			};
			if s.device != "/dev/null" {
				match s.init() {
					Ok(_) => {
						return Ok(s);
					}
					Err(error) => {
						return Err(error);
					}
				}
			}
		} else {
			s = ModIR::new_default();
		}
		Ok(s)
	}

	/// This function initiates the I2C connection to the MOD-IR-TEMP sensor.
	/// It returns nothing on success and returns an error message on failure.
	fn init(&mut self) -> Result<(), Error<LinuxI2CError>> {
		match Mlx9061x::new_mlx90614(I2cdev::new(&self.device).unwrap(), self.addr, 5) {
			Ok(mlx_sensor) => {
				self.mlx = Some(mlx_sensor);
				()
			}
			Err(error) => {
				return Err(error);
			}
		}
		Ok(())
	}

	/// This function checks if a state change has occurred and returns the corresponding state change.
	/// It returns 'None' if nothing has changed since the last duration.
	/// It returns 'ChangedTooBothToHot' if both thresholds have been exceeded.
	/// It returns 'ChangedTooAmbientToHot' if only the ambient threshold has been exceeded.
	/// It returns 'ChangedTooObjectToHot' if only the object threshold has been exceeded.
	/// It returns 'ChangedTooCancelled' if 'ChangedToBothTooHot' or 'ChangedToAmbientTooHot' or 'ChangedToObjectTooHot' was set, and both ambient and object temperatures are below their respective threshold values.
	fn set_handle_output(
		&mut self,
		ambient_state: IrTempState,
		object_state: IrTempState,
	) -> IrTempStateChange {
		if self.active_ambient_state == ambient_state && self.active_object_state == object_state {
			return IrTempStateChange::None;
		} else if ambient_state == IrTempState::TooHot && object_state == IrTempState::TooHot {
			self.active_ambient_state = ambient_state;
			self.active_object_state = object_state;
			return IrTempStateChange::ChanedToBothToHot;
		} else if self.active_ambient_state != ambient_state && ambient_state == IrTempState::TooHot
		{
			self.active_ambient_state = ambient_state;
			return IrTempStateChange::ChangedToAmbientToHot;
		} else if self.active_object_state != object_state && object_state == IrTempState::TooHot {
			self.active_object_state = object_state;
			return IrTempStateChange::ChangedToObjectToHot;
		} else if ambient_state == IrTempState::Normal && object_state == IrTempState::Normal {
			self.active_ambient_state = ambient_state;
			self.active_object_state = object_state;
			return IrTempStateChange::ChangedToCancelled;
		}
		return IrTempStateChange::None;
	}

	/// This function reads the ambient temperature and object temperature from the MOD-IR-TEMP sensor.
	/// It then sets the ambient and object states to 'Normal' or 'TooHot' based on whether they are below or above the thresholds.
	/// The function returns the ChangeState to the calling function on success, and returns an error message on failure.
	/// The update frequency can be controlled with the configuration of [ir/data/interval], but also depends on the calling function.
	/// For example, if this function is called every minute and the ir/data/interval is set to 5, the sensor data will be read every 5 minutes.
	pub fn handle(&mut self) -> Result<IrTempStateChange, Error<LinuxI2CError>> {
		match &mut self.mlx {
			Some(mlx_sensor) => {
				let mut ambient_state = IrTempState::Normal;
				let mut object_state = IrTempState::Normal;
				match mlx_sensor.ambient_temperature() {
					Ok(amb_temp) => {
						self.ambient_temp = amb_temp;
						if amb_temp > THRESHOLD_AMBIENT {
							ambient_state = IrTempState::TooHot;
						}
					}
					Err(error) => {
						return Err(error);
					}
				}

				match mlx_sensor.object1_temperature() {
					Ok(obj_temp) => {
						self.object_temp = obj_temp;
						if obj_temp > THRESHOLD_OBJECT {
							object_state = IrTempState::TooHot;
						}
					}
					Err(error) => {
						return Err(error);
					}
				}
				return Ok(self.set_handle_output(ambient_state, object_state));
			}
			None => (),
		}
		Ok(IrTempStateChange::None)
	}

	/// This function sets the emissivity for measuring the object temperature.
	/// Default the emissivity for the MOD-IR-TEMP is set to `1`.
	/// However, the emissivity only needs to be adjusted if we are using a specific object for measurement, as indicated [here](https://en.wikipedia.org/wiki/Emissivity).
	/// The 'emissivity' parameter can be chosen between 0.0 and 1.0.
	pub fn _change_emissivity(&mut self, emissivity: f32) -> Result<bool, Error<LinuxI2CError>> {
		if emissivity >= 0.0 && emissivity <= 1.0 {
			match &mut self.mlx {
				Some(mlx_sensor) => match mlx_sensor.set_emissivity(emissivity, &mut Delay {}) {
					Ok(_) => {
						self._emissivity = emissivity;
						return Ok(true);
					}
					Err(error) => {
						return Err(error);
					}
				},
				None => {
					return Ok(false);
				}
			}
		}
		return Ok(false);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_set_handle_output() {
		let mut s = ModIR {
			mlx: None,
			device: "/dev/null".to_string(),
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			_emissivity: 1.0,
			active_ambient_state: IrTempState::Normal,
			active_object_state: IrTempState::Normal,
		};

		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::Normal)
				== IrTempStateChange::None
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::Normal)
				== IrTempStateChange::ChangedToAmbientToHot
		);
		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::Normal)
				== IrTempStateChange::ChangedToCancelled
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::TooHot)
				== IrTempStateChange::ChanedToBothToHot
		);
		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::TooHot)
				== IrTempStateChange::None
		);
		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::Normal)
				== IrTempStateChange::ChangedToCancelled
		);
		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::TooHot)
				== IrTempStateChange::ChangedToObjectToHot
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::TooHot)
				== IrTempStateChange::ChanedToBothToHot
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::Normal)
				== IrTempStateChange::None
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::TooHot)
				== IrTempStateChange::None
		);
		assert!(
			s.set_handle_output(IrTempState::Normal, IrTempState::Normal)
				== IrTempStateChange::ChangedToCancelled
		);
		assert!(
			s.set_handle_output(IrTempState::TooHot, IrTempState::Normal)
				== IrTempStateChange::ChangedToAmbientToHot
		);
	}
}
