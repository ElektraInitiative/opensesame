/// Before using this struct you need setup the config with libelektra.
/// 
/// For example:
/// kdb set user:/sw/libelektra/opensesame/#0/current/climasensor/device "/dev/ttyS5"
/// kdb set user:/sw/libelektra/opensesame/#0/current/climasensor/baudrate 9600
/// 
/// Otherwise the default config is used, which disables this struct by using /dev/null as device.

extern crate libmodbus;
use libmodbus::*;
use crate::config::Config;

///Constants 

const ERROR_CODE_S32: i32 = 0x7FFFFFFF;
const ERROR_CODE_U32: i32 = 0xFFFFFFFF;

//Special reg-addresses
const REG_MEAN_WIND_SPEED: u16 = 0x7533;
const REG_AIR_TEMP: u16 = 0x76C1;


pub struct ClimaSensorUS{
    device: &String,
    baudrate: i32,
    parity: char,
    data_bits: i32,
    stop_bits: i32,
    slave_id: u8,
    ctx: Modbus,
    alarm_active: bool,
}

impl ClimaSensorUS{
    
    pub fn new(config: &mut Config) -> Self{
        let device_name = config.get::<string>("climasensor/device");
        if device_name != "/dev/null" {
            let mut s = Self {
                device: device_name,
                baudrate: config.get::<i32>("climasensor/baudrate"),
                parity: config.get::<char>("climasensor/parity"),
                data_bits: config.get::<i32>("climasensor/databits"),
                stop_bits: config.get::<i32>("climasensor/stopbits"),
                slave_id: config.get::<i32>("climasensor/salveid"),
                ctx: null,
                alarm_active: false,
            };
            s.init();
            s
        }else{
            Self {
                device: device_name,
                baudrate: 0,
                parity: 'N',
                data_bits: 0,
                stop_bits: 0,
                slave_id: 0,
                ctx: null,
                alarm_active: false,
            }
        }
    }

    fn init(&mut self){
        self.ctx = Modbus::new_rtu(self.device, self.baudrate, self.parity, self.data_bits, self.stop_bits).expect("Error accured while creating new RTU Object");
        self.ctx.set_slave(self.slave_id).unwrap_or_else(|_| panic!("Error accured while setting slave-id to '{}'", self.slave_id));
        self.ctx.rtu_set_serial_mode(SerialMode::RtuRS485).expect("Error accured while setting serial mode to RS485");
        self.ctx.rtu_set_rts(RequestToSendMode::RtuRtsUp).expect("Error accured while setting RTS ti RTS-UP");
        self.ctx.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp).expect("Error accured while setting custom RTS-function");

        self.ctx.connect().expect("Error accured while connecting to Clima-Sensor");
    }

    /// These functions create a single number out of an vector.
    /// The first entry in the vector are the most significant bytes and the second entry are the least significant bytes

    fn conv_vec_to_value_s(vec: Vec<i16>) -> i32{
        (vec[0]<<16 | vec[1]).into()
    }

    fn conv_vec_to_value_u(vec: Vec<u16>) -> u32{
        (vec[0]<<16 | vec[1]).into()
    }

    

    /// This function should be called periodically to check the temperature and wind speed.
    /// if temp > 30°C and no wind, then a warning should be issued
    /// if temp > 35°C a warning should be issued
    /// if temp < 20° the warning should be removed
    /// (Input Register - 0x04) temp-reg address 0x76C1; typ S32; real_result = response_temp/10
    /// (Input Register - 0x04) wind-reg address 0x7533; typ U32; real_result = response_wind/10
    /// 
    /// The return value is bool on success, true if alarm is active and false is alarm is not active
    /// If no ctx is configured the this function returns always false, so no warning is triggered

    pub fn handle_temperature(&mut self) -> Result<bool,Error> {
        if self.ctx != null {
            let mut response_temp = vec![0i16; 2];
            let mut response_wind = vec![0u16; 2];

            self.ctx.read_input_registers(REG_AIR_TEMP, 2, &mut response_temp)?;
            self.ctx.read_input_registers(REG_MEAN_WIND_SPEED, 2, &mut response_wind)?;

            let temp: f32 = response_temp/10;
            let wind: f32 = response_wind/10;

            if !self.alarm_active && temp > 30 && wind < 0.3 {
                self.alarm_active = true;
            } else if !self.alarm_active && temp > 35 {
                self.alarm_active = true;
            } else if self.alarm_active && temp < 20 {
                self.alarm_active= false;
            }

            Ok(self.alarm_active)
        }else{
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    /// Befor using this test you need to setup you configuration with libelektra
    #[test]
    #[ignore]
    fn test_handle_temperature(){
        
        let weatherstation: ClimaSensorUS = new(Config::new("/sw/libelektra/opensesame/#0/current"));


    }

    #[test]
    fn test_conv_vec_to_value_s(){
        assert!(conv_vec_to_value_s(vec![0,32]), "32");
        assert!(conv_vec_to_value_s(vec![-1,0]),"-65536");
    }

    #[test]
    fn test_conv_vec_to_value_u(){
        let vec: Vec<u16> = [0,32];
        assert!(conv_vec_to_value_u(vec), "32");
    }
}