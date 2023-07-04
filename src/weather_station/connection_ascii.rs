use serialport::{DataBits, Parity, StopBits, Error};
use std::io::{Read, Write};
use std::time::Duration;

use std::env;
use gpio::GpioOut;

fn main() -> Result<(), Error> {

    const PIN_SCL: u16 = 273;
    const PIN_SS: u16 = 272;

    let args: Vec<String> = env::args().collect();
    let device = &args[1];
    let baudrate: u32 = args[2].parse().unwrap();

    let mut gpio_scl = gpio::sysfs::SysFsGpioOutput::open(PIN_SCL).unwrap();
    let mut gpio_ss = gpio::sysfs::SysFsGpioOutput::open(PIN_SS).unwrap();

    //open the serial interface
    let mut port = serialport::new(device,baudrate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_secs(1))
        .open()?;
        
    // Send data over the serial interface
    gpio_scl.set_value(true).expect("cant set gpio_scl to true");
    gpio_ss.set_value(true).expect("cant set gpio_ss to true");

    let change_to_admin = b"00KY4711";
    port.write_all(change_to_admin)?;

    //sleep dazwischen

    let set_half_duplex = b"00DM0";
    port.write_all(set_half_duplex)?;

    
    // Receive data from the serial interface
    gpio_scl.set_value(false).expect("cant set gpio_scl to false");
    gpio_ss.set_value(false).expect("cant set gpio_ss to false");

    let mut buffer = Vec::new();
    port.read_to_end(&mut buffer)?;
    
    // Process the received data 
    let received_data = String::from_utf8_lossy(&buffer);
    println!("Received: {}", received_data);
    
    Ok(())
}
