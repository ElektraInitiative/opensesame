use i2cdev::linux::LinuxI2CError;
/// This module implements the functions of the [MOD-IR-TEMP](https://www.olimex.com/Products/Modules/Sensors/MOD-IR-TEMP/open-source-hardware)
use linux_embedded_hal::I2cdev;
use mlx9061x::ic::Mlx90614;
use mlx9061x::{Error, Mlx9061x, SlaveAddr};

const THRESHOLD_AMBIENT: f32 = 23.0;
const THRESHOLD_OBJECT: f32 = 44.0;

#[derive(Clone, Copy, PartialEq)]
pub enum IrTempStateChange {
	None,
	AmbientToHot,
	ObjectToHot,
}

pub struct ModIrTemps {
	mlx: Option<Mlx9061x<I2cdev, Mlx90614>>,
	device: String,
	addr: SlaveAddr,
	ambient_temp: f32,
	object_temp: f32,
	emissivity: f32,
	state: IrTempStateChange,
}

impl ModIrTemps {
	//Using default address
	pub fn new(
		device_name: &'static str,
		address: Option<u8>,
	) -> Result<Self, Error<LinuxI2CError>> {
		let mut s = Self {
			mlx: None,
			device: device_name.to_string(),
			addr: SlaveAddr::Default,
			ambient_temp: 0.0,
			object_temp: 0.0,
			emissivity: 1.0,
			state: IrTempStateChange::None,
		};

		match address {
			Some(addr) => {
				s.addr = SlaveAddr::Alternative(addr);
			}
			None => {
				s.addr = SlaveAddr::Default;
			}
		}

		match s.init() {
			Ok(_) => Ok(s),
			Err(error) => Err(error),
		}
	}

	fn init(&mut self) -> Result<(), Error<LinuxI2CError>> {
		match Mlx9061x::new_mlx90614(I2cdev::new(self.device.as_str()).unwrap(), self.addr, 5) {
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

	pub fn handle(&mut self) -> Result<IrTempStateChange, Error<LinuxI2CError>> {
		match &mut self.mlx {
			Some(mlx_sensor) => {
				match mlx_sensor.ambient_temperature() {
					Ok(amb_temp) => {
						self.ambient_temp = amb_temp;
					}
					Err(error) => {
						return Err(error);
					}
				}

				match mlx_sensor.object1_temperature() {
					Ok(obj_temp) => {
						self.object_temp = obj_temp;
					}
					Err(error) => {
						return Err(error);
					}
				}

				if self.ambient_temp > THRESHOLD_AMBIENT
					&& matches!(self.state, IrTempStateChange::None)
				{
					self.state = IrTempStateChange::AmbientToHot;
				} else if self.object_temp > THRESHOLD_OBJECT
					&& matches!(self.state, IrTempStateChange::None)
				{
					self.state = IrTempStateChange::ObjectToHot;
				} else if (self.object_temp < THRESHOLD_OBJECT
					&& matches!(self.state, IrTempStateChange::ObjectToHot))
					|| (self.ambient_temp < THRESHOLD_AMBIENT
						&& matches!(self.state, IrTempStateChange::AmbientToHot))
				{
					self.state = IrTempStateChange::None;
				}

				return Ok(self.state);
			}
			None => Ok(IrTempStateChange::None),
		}
	}
}
