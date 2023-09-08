/// This program writes to a register of the clima-sensor-us weather station
extern crate libmodbus;
use libmodbus::*;
use std::env;

///Constants
const DEVICE: &'static str = "/dev/ttyS5";
const BAUDRATE: i32 = 9600;
const PARITY: char = 'N';
const DATA_BITS: i32 = 8;
const STOP_BITS: i32 = 1;
const SLAVE_ID: u8 = 1;

const KY_REG: u16 = 40009;

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() == 3 {
		let reg: u16 = args[1].parse::<u16>().unwrap();
		let value: Vec<u16> = vec![0, args[2].parse::<u16>().unwrap()];

		// Modbus-Verbindung initialisieren
		let mut ctx = Modbus::new_rtu(DEVICE, BAUDRATE, PARITY, DATA_BITS, STOP_BITS).unwrap();
		ctx.set_slave(SLAVE_ID).expect("Setting Slave-ID failed!");
		assert!(ctx.rtu_set_serial_mode(SerialMode::RtuRS232).is_ok());
		assert!(ctx.rtu_set_rts(RequestToSendMode::RtuRtsUp).is_ok());
		assert!(ctx.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp).is_ok());

		// Modbus-Verbindung öffnen
		ctx.connect().expect("Verbindung mit ctx Fehlerhaft!");

		ctx.write_registers(KY_REG, 2, &vec![0,0x1267]).expect("Fehler beim setzen vom Register");
		ctx.write_registers(reg, 2, &value).expect("Fehler beim setzen vom Register");
		ctx.write_registers(KY_REG, 1, &vec![0,0]).expect("Fehler beim setzen vom Register");

		// Modbus-Verbindung schließen
		ctx.close();
		ctx.free();
	} else {
		println!("Usage: {} <register> <value>", &args[0]);
	}
}