extern crate libmodbus;
use libmodbus::*;

///Constants
const DEVICE: &'static str = "/dev/ttyS5";
const BAUDRATE: i32 = 9600;
const PARITY: char = 'N';
const DATA_BITS: i32 = 8;
const STOP_BITS: i32 = 1;
const SLAVE_ID: u8 = 1;

const ERROR_CODE_S32: u32 = 0x7FFFFFFF;
const ERROR_CODE_U32: u32 = 0xFFFFFFFF;

//Special reg-addresses
const REG_MEAN_WIND_SPEED: u16 = 0x7533;
const REG_AIR_TEMP: u16 = 0x76C1;

/// These functions create a single number out of a vector.
/// The first entry in the vector are the most significant bytes and the second entry are the least significant bytes.
/// For the unsigned function the input `vec` should be already in two complement, so that the function works right.
fn conv_vec_to_value_s(vec: Vec<u16>) -> Result<i32,()> { 
    let usign_val: u32 = (vec[0] as u32) << 16 | (vec[1] as u32);
    if usign_val == ERROR_CODE_S32 {
        Err(())
    }else{
        Ok(usign_val as i32)
    }
}

fn conv_vec_to_value_u(vec: Vec<u16>) -> Result<u32,()> {
    let usign_val = (vec[0] as u32) << 16 | (vec[1] as u32);
    if usign_val == ERROR_CODE_U32 {
        Err(())
    }else{
        Ok(usign_val)
    }
}

#[derive(Clone,Copy,PartialEq)]
pub enum TempWarningStateChange {
    None,
    ChangeToCloseWindow,
    ChangeToWarningTempNoWind,
    ChangeToWarningTemp,
    ChangeToRemoveWarning,
}

#[derive(Clone,Copy,PartialEq)]
pub enum TempWarning {
    None,
    RemoveWarning,
    CloseWindow,
    WarningTempNoWind,
    WarningTemp,
}

pub struct ClimaSensorUS{
    ctx: Option<Modbus>,
    warning_active: TempWarning,
}

impl ClimaSensorUS{
    
    pub fn new(enabled: bool) -> Self{
        let mut s = Self {
            ctx: None,
            warning_active: TempWarning::None,
        };
        if enabled {
            s.init();
        }
        s
    }

    fn init(&mut self){
        self.ctx = Some(Modbus::new_rtu(DEVICE, BAUDRATE, PARITY, DATA_BITS, STOP_BITS).expect("Error accured while creating new RTU Object"));
        
        if let Some(conn) = &mut self.ctx {
            conn.set_slave(SLAVE_ID).unwrap_or_else(|_| panic!("Error accured while setting slave-id to '{}'", SLAVE_ID));
            conn.rtu_set_serial_mode(SerialMode::RtuRS232).expect("Error accured while setting serial mode to RS485");
            conn.rtu_set_rts(RequestToSendMode::RtuRtsUp).expect("Error accured while setting RTS ti RTS-UP");
            conn.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp).expect("Error accured while setting custom RTS-function");

            conn.connect().expect("Error accured while connecting to Clima-Sensor");
        }
        
    }    

    /// This function should be called periodically to check the sensors' values.
    /// if temp > 30°C and no wind, then a warning should be issued
    /// if temp > 35°C a warning should be issued
    /// if temp < 20° either warning is removed
    /// (Input Register - 0x04) temp-reg address 0x76C1; typ S32; real_result = response_temp/10
    /// (Input Register - 0x04) wind-reg address 0x7533; typ U32; real_result = response_wind/10
    /// 
    /// The return value is bool on success, true if alarm is active and false is alarm is not active
    /// If no ctx is configured the this function returns always false, so no warning is triggered
    pub fn handle(&mut self) -> Result<TempWarningStateChange,Error> {
        match &self.ctx  {
            Some(conn) => {
                let mut response_temp = vec![0u16; 2];
                let mut response_wind = vec![0u16; 2];

                conn.read_input_registers(REG_AIR_TEMP, 2, &mut response_temp)?;
                conn.read_input_registers(REG_MEAN_WIND_SPEED, 2, &mut response_wind)?;

                let temp: f32 = (conv_vec_to_value_s(response_temp).unwrap() as f32)/10.0;
                let wind: f32 = (conv_vec_to_value_u(response_wind).unwrap() as f32)/10.0;
                
                #[cfg(debug_assertions)]
                println!("Weatherstation: temperature {} °C, windspeed {} m/s", temp, wind);

                Ok(self.set_warning_active(temp,wind))
            },
            None => Ok(TempWarningStateChange::None)
        }
    }

    /// This function is used to set the warning_active varibale and compare it with the new value.
    fn set_warning_active(&mut self,temp: f32, wind: f32) -> TempWarningStateChange{
        let new_warning: TempWarning;
        let mut result: TempWarningStateChange = TempWarningStateChange::None;

        if temp > 35.0 {
            new_warning = TempWarning::WarningTemp;
        }else if temp > 30.0 && wind < 0.3 {
            new_warning = TempWarning::WarningTempNoWind;
        }else if temp > 23.0 {
            new_warning = TempWarning::CloseWindow;
        }else if !matches!(self.warning_active, TempWarning::None) && !matches!(self.warning_active, TempWarning::RemoveWarning) && temp < 20.0 {
            new_warning = TempWarning::RemoveWarning;
        }else{
            new_warning = TempWarning::None;
        }

        // compaire old and new value of TempWarning
        if self.warning_active != new_warning {
            result = match new_warning {
                TempWarning::RemoveWarning => {
                    self.warning_active = new_warning;
                    TempWarningStateChange::ChangeToRemoveWarning
                },
                TempWarning::CloseWindow => {
                    self.warning_active = new_warning;
                    TempWarningStateChange::ChangeToCloseWindow
                },
                TempWarning::WarningTempNoWind => {
                    self.warning_active = new_warning;
                    TempWarningStateChange::ChangeToWarningTempNoWind
                },
                TempWarning::WarningTemp => {
                    self.warning_active = new_warning;
                    TempWarningStateChange::ChangeToWarningTemp
                },
                TempWarning::None => {
                    self.warning_active = new_warning;
                    TempWarningStateChange::None
                }

            };
        }
        result 
    }

}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    #[ignore]
    fn test_handle(){
        let mut weatherstation = ClimaSensorUS::new(true);

        match weatherstation.handle().unwrap(){
            TempWarningStateChange::ChangeToCloseWindow => println!("ChangeToCloseWindow"),
            TempWarningStateChange::ChangeToWarningTempNoWind => println!("ChangeToWarningTempNoWind"),
            TempWarningStateChange::ChangeToWarningTemp => println!("ChangeToWarningTemp"),
            TempWarningStateChange::ChangeToRemoveWarning => println!("ChangeToRemoveWarning"),
            TempWarningStateChange::None => println!("None"),
        }
    }

    #[test]
    fn test_set_warning_active(){
        let mut clima_sens = ClimaSensorUS {
                ctx: None,
                warning_active: TempWarning::None,
        };

        assert!(clima_sens.set_warning_active(15.0,0.1) == TempWarningStateChange::None);
        assert!(matches!(clima_sens.warning_active,TempWarning::None));
        assert!(clima_sens.set_warning_active(15.0,3.5) == TempWarningStateChange::None);
        assert!(matches!(clima_sens.warning_active,TempWarning::None));

        assert!(clima_sens.set_warning_active(25.0,0.1) == TempWarningStateChange::ChangeToCloseWindow);
        assert!(matches!(clima_sens.warning_active, TempWarning::CloseWindow));
        assert!(clima_sens.set_warning_active(25.0,3.5) == TempWarningStateChange::None);
        assert!(matches!(clima_sens.warning_active, TempWarning::CloseWindow));

        assert!(clima_sens.set_warning_active(33.0,0.1) == TempWarningStateChange::ChangeToWarningTempNoWind);
        assert!(matches!(clima_sens.warning_active,TempWarning::WarningTempNoWind));
        assert!(clima_sens.set_warning_active(33.0,3.5) == TempWarningStateChange::ChangeToCloseWindow);
        assert!(matches!(clima_sens.warning_active, TempWarning::CloseWindow));

        assert!(clima_sens.set_warning_active(36.0,0.1) == TempWarningStateChange::ChangeToWarningTemp);
        assert!(matches!(clima_sens.warning_active, TempWarning::WarningTemp));
        assert!(clima_sens.set_warning_active(36.0,3.5) == TempWarningStateChange::None);
        assert!(matches!(clima_sens.warning_active, TempWarning::WarningTemp));

        assert!(clima_sens.set_warning_active(15.0,0.1) == TempWarningStateChange::ChangeToRemoveWarning);
        assert!(matches!(clima_sens.warning_active, TempWarning::RemoveWarning));
        assert!(clima_sens.set_warning_active(15.0,3.5) == TempWarningStateChange::None);
        assert!(matches!(clima_sens.warning_active, TempWarning::None));
    }

    #[test]
    fn test_conv_vec_to_value_s(){
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
        assert_eq!(conv_vec_to_value_s(vec![0x8000u16, 0x0000u16]), Ok(-2147483648));
    }

    #[test]
    fn test_conv_vec_to_value_u(){
        assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0000u16]), Ok(0));
        assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0001u16]), Ok(1));
        assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x000au16]), Ok(10));
        assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x0020u16]), Ok(32));
        assert_eq!(conv_vec_to_value_u(vec![0x0000u16, 0x1524u16]), Ok(5412));
        assert_eq!(conv_vec_to_value_u(vec![0xffffu16, 0xffffu16]), Err(()));
    }
}