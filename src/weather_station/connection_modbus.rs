extern crate libmodbus_rs;

use libmodbus_rs::*;
use std::{thread, time}; 



fn main() {
    let PIN_SCK = 273;
    let PIN_SS = 272;

    // Modbus-Verbindung initialisieren
    let mut ctx = Modbus::new_rtu("/dev/ttyS5", 9600, 'N', 8, 1).unwrap();
    ctx.set_slave(0).expect("Setting Slave-ID failed!");
    assert!(ctx.rtu_set_serial_mode(SerialMode::RtuRS485).is_ok());
    assert!(ctx.rtu_set_rts(RequestToSendMode::RtuRtsUp).is_ok());
    assert!(ctx.rtu_set_costom_rts(RequestToSendMode::RtuRtsUp).is_ok());

    // Modbus-Verbindung öffnen
    ctx.connect().expect("Verbindung mit ctx Fehlerhaft!");

    //Change to Half-Duplex
    print!("Change to Admin\n");
    let _ = ctx.write_register(0x9c49, 0x1267);

    print!("Change to Half-Duplex\n");
    let _ = ctx.write_register(0x9c4b, 0x0);

    print!("Save by switching back to READ_ONLY\n");
    let _= ctx.write_register(0x9c49, 0x0);

    //try to read register
    let mut dest = vec![0u16; 100];

    let result = ctx.read_input_registers(0x7533, 1, &mut dest).expect("read_input_register something went wrong!");

    println!("Result of read_input_reg {}", result);

    for value in dest.iter(){
        println!("Received data: {}", value);
    }

    // Modbus-Verbindung schließen
    ctx.close();
    ctx.free();
}
