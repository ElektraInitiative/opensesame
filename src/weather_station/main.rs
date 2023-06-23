use serialport::{DataBits, Parity, StopBits, Error};
use std::io::{Read, Write};
use std::time::Duration;

use std::env;
use gpio::GpioOut;

fn main() -> Result<(), Error> {

    let args: Vec<String> = env::args().collect();
    let device = &args[1];
    let baudrate: u32 = args[2].parse().unwrap();

    let mut gpio_scl = gpio::sysfs::SysFsGpioOutput::open(273).unwrap();
    let mut gpio_ss = gpio::sysfs::SysFsGpioOutput::open(272).unwrap();

    // Öffne die serielle Schnittstelle
    let mut port = serialport::new(device,baudrate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_secs(1))
        .open()?;
        
    // Sende Daten über die serielle Schnittstelle
    gpio_scl.set_value(true).expect("cant set gpio_scl to true");
    gpio_ss.set_value(true).expect("cant set gpio_ss to true");

    let data = b"00KY1";
    port.write_all(data)?;


    
    // Empfange Daten von der seriellen Schnittstelle
    gpio_scl.set_value(false).expect("cant set gpio_scl to false");
    gpio_ss.set_value(false).expect("cant set gpio_ss to false");

    let mut buffer = Vec::new();
    port.read_to_end(&mut buffer)?;
    
    // Verarbeite die empfangenen Daten
    let received_data = String::from_utf8_lossy(&buffer);
    println!("Received: {}", received_data);
    
    Ok(())
}
