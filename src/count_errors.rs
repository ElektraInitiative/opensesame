use std::{thread, time, env};

use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
use std::time::Instant;

use systemstat::{System, Platform};

mod config;

use config::Config;

const INNER_LOOP: u8 = 20;

const SET_TRIS: u8 = 0x01;    // Set GPIO direction
const SET_PORTS: u8 = 0x02;   // Set GPIO output level
const GET_PORTS: u8 = 0x03;   // Get GPIO input level
const SET_PULLUPS: u8 = 0x04; // Set GPIO pull-ups
const SET_RELAYS_ON: u8 = 0x41;  // Set relay(s) on
const SET_RELAYS_OFF: u8 = 0x42;  // Set relay(s) off


const BUTTON_1: u8 = 0x01;
const BUTTON_2: u8 = 0x01 << 1;
const BUTTON_3: u8 = 0x01 << 2;
const BUTTON_4: u8 = 0x01 << 3; // = GPIO3 with external pull-up

// input from GPIO0 - GPIO3 i.e. all buttons (and also buttons+taster on board21)
const ALL_BUTTONS: u8 = BUTTON_1 | BUTTON_2 | BUTTON_3 | BUTTON_4;


const RELAY_DOOR: u8 = 0x01;
const RELAY_BELL: u8 = 0x01 << 1;

const ALL_RELAYS: u8 = RELAY_DOOR | RELAY_BELL;


fn main() {
	let mut config: Config = Config::new();
	env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
	let address = config.get::<u16>("count/errors/address");
	let modio = config.get_bool("count/errors/modio"); // use MOD-IO instead of MOD-IO2
	let mut board21 = LinuxI2CDevice::new("/dev/i2c-2", address).unwrap();
	println!("Using address {:#02X}, modio: {}", address, modio);

	let get_ports;
	let set_relays_on;
	let set_relays_off;
	if modio {
		get_ports = 0x30; // read AIN1A
		set_relays_on = 0x10;
		set_relays_off = 0x10;
	} else {
		get_ports = GET_PORTS;
		set_relays_on = SET_RELAYS_ON;
		set_relays_off = SET_RELAYS_OFF;

		board21.smbus_write_byte_data(SET_TRIS, ALL_BUTTONS).unwrap();
		board21.smbus_write_byte_data(SET_PULLUPS, ALL_BUTTONS).unwrap();
		board21.smbus_write_byte_data(SET_PORTS, ALL_BUTTONS).unwrap();
		board21.smbus_write_byte_data(SET_RELAYS_OFF, ALL_RELAYS).unwrap();
	}

	let mut errors = 0;
	let mut errors_get = 0;
	let mut errors_relay = 0;
	let mut tests = 0;
	let mut iterations = 0;
	let mut error_locations = Vec::new();

	let sys = System::new();
	let cpu = sys.cpu_load_aggregate().unwrap();
	let now = Instant::now();

	for i in 0..50 {
		iterations += 1;

		for j in 0..INNER_LOOP {
			tests += 1;
			let ret = board21.smbus_read_byte_data(get_ports);
			if ret.is_err() {
				errors += 1;
				errors_get += 1;
				error_locations.push(("get", iterations, tests, i, j));
			}
			thread::sleep(time::Duration::from_millis(10));
		}

		tests += 1;
		let ret = board21.smbus_write_byte_data(set_relays_on, if modio {0xf} else {ALL_RELAYS});
		if ret.is_err() {
			errors += 1;
			errors_relay += 1;
			error_locations.push(("relay", iterations, tests, i, 1));
		}

		for j in 0..INNER_LOOP {
			tests += 1;
			let ret = board21.smbus_read_byte_data(get_ports);
			if ret.is_err() {
				errors += 1;
				errors_get += 1;
				error_locations.push(("get", iterations, tests, i, j));
			}
			thread::sleep(time::Duration::from_millis(10));
		}

		tests += 1;
		let ret = board21.smbus_write_byte_data(set_relays_off, if modio {0} else {ALL_RELAYS});
		if ret.is_err() {
			errors += 1;
			errors_relay += 1;
			error_locations.push(("relay", iterations, tests, i, 2));
		}
	}

	let cpu = cpu.done().unwrap();
	let elapsed = now.elapsed();
	let loadavg = sys.load_average().unwrap();

	println!("From {} iterations and {} tests I got {} errors (get: {} relay: {})", iterations, tests, errors, errors_get, errors_relay);
	println!("Elapsed Time: {:.2?}", elapsed);
	println!("CPU load during execution: {}% user, {}% nice, {}% system, {}% intr, {}% idle ", cpu.user * 100.0, cpu.nice * 100.0, cpu.system * 100.0, cpu.interrupt * 100.0, cpu.idle * 100.0);
	println!("Executed version {} with boot time {}", env!("CARGO_PKG_VERSION"), sys.boot_time().unwrap());
	println!("Load average: {} {} {}, Memory usage: {}, Swap: {}, CPU temp: {}", loadavg.one, loadavg.five, loadavg.fifteen, sys.memory().unwrap().total, sys.swap().unwrap().total, sys.cpu_temp().unwrap());
	#[cfg(debug_assertions)]
	println!("Debugging enabled");
	if !error_locations.is_empty() {
		println!("Following errors occurred:");
		for e in &error_locations {
			println!("{:?}", e);
		}
	}
}

