/*This program tests if the connection with the weather station is possible and for that reason the register 40015 is read.*/
 
extern crate libmodbus;

use libmodbus::*;
use std::{thread, time}; 

fn main() {
    let PIN_SCK = 273;
    let PIN_SS = 272;

    // Modbus-Verbindung initialisieren
    let mut ctx = Modbus::new_rtu("/dev/ttyS5", 9600, 'N', 8, 1).unwrap();
    ctx.set_slave(1).expect("Setting Slave-ID failed!");
    assert!(ctx.rtu_set_serial_mode(SerialMode::RtuRS485).is_ok());
    assert!(ctx.rtu_set_rts(RequestToSendMode::RtuRtsUp).is_ok());
    assert!(ctx.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp).is_ok())

    // Modbus-Verbindung öffnen
    ctx.connect().expect("Verbindung mit ctx Fehlerhaft!");

    //try to read register
    let mut dest = vec![0u16; 5];

    let result = ctx.read_registers(40015, 2, &mut dest).expect("read_input_register something went wrong!");

    println!("Result of read_input_reg {}", result);

    for value in dest.iter(){
        println!("Received data: {}", value);
    }

    // Modbus-Verbindung schließen
    ctx.close();
    ctx.free();
}
