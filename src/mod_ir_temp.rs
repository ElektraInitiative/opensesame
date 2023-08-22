/// This module implements the functions of the [MOD-IR-TEMP](https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware)
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

pub struct ModIrTemps {
	mlx: Option<Mlx9061x<I2cdev, Mlx90614>>,
	device: String,
	addr: SlaveAddr,
	pub ambient_temp: f32,
	pub object_temp: f32,
	_emissivity: f32,
	active_ambient_state: IrTempState,
	active_object_state: IrTempState,
	data_interval: u16,
	read_counter: u16,
}

impl ModIrTemps {
	//Generate default instanz of struct.
	pub fn new_default() -> Self {
		Self {
			mlx: None,
			device: "/dev/null".to_string(),
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			_emissivity: 1.0,
			active_ambient_state: IrTempState::Normal,
			active_object_state: IrTempState::Normal,
			data_interval: 0,
			read_counter: 0,
		}
	}

	pub fn new(
		device_name: String,
		address: Option<u8>,
		data_interval: u16,
	) -> Result<Self, Error<LinuxI2CError>> {
		let mut s = Self {
			mlx: None,
			device: device_name,
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			_emissivity: 1.0,
			active_ambient_state: IrTempState::Normal,
			active_object_state: IrTempState::Normal,
			data_interval: data_interval,
			read_counter: 0,
		};
		if s.device != "/dev/null" {
			match address {
				Some(addr) => {
					s.addr = SlaveAddr::Alternative(addr);
				}
				None => {
					s.addr = SlaveAddr::Default;
				}
			}

			match s.init() {
				Ok(_) => {
					return Ok(s);
				}
				Err(error) => {
					return Err(error);
				}
			}
		}
		Ok(s)
	}

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

	pub fn handle(&mut self) -> Result<IrTempStateChange, Error<LinuxI2CError>> {
		match &mut self.mlx {
			Some(mlx_sensor) => {
				self.read_counter += 1;
				if self.read_counter == self.data_interval {
					self.read_counter = 0;
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
			}
			None => (),
		}
		Ok(IrTempStateChange::None)
	}

	//This function sets the emissivity for the measurement of the object temperture
	//The parameter emissivity can be choosen between 0.0 and 1.0
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
		let mut s = ModIrTemps {
			mlx: None,
			device: "/dev/null".to_string(),
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			_emissivity: 1.0,
			active_ambient_state: IrTempState::Normal,
			active_object_state: IrTempState::Normal,
			data_interval: 0,
			read_counter: 0,
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
