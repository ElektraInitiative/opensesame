/// This programm dumps all registers of the clima-sensor-us weather station
extern crate libmodbus;
use libmodbus::*;

///Constants
const DEVICE: &'static str = "/dev/ttyS5";
const BAUDRATE: i32 = 9600;
const PARITY: char = 'N';
const DATA_BITS: i32 = 8;
const STOP_BITS: i32 = 1;
const SLAVE_ID: u8 = 1;

///All 96 input registers
const INPUT_REG: [(u16, &'static str, &'static str, i32, char); 96] = [
	(0x7533, "Mittelwert Windgeschwindigkeit", "m/s", 10, 'u'),
	(0x753B, "Maximalwert Windgeschwindigkeit (Böe) verfügbar wenn AV>=30", "m/s", 10, 'u'),
	(0x75FB, "Mittelwert Windrichtung", "°", 10, 'u'),
	(0x7603, "Windrichtung der Böe verfügbar wenn AV>=30", "°", 10, 'u'),
	(0x76C1, "Lufttemperatur", "°C", 10, 's'),
	(0x76C3, "Gehäuseinnentemperatur", "°C", 10, 's'),
	(0x76C5, "Akustische Temperatur", "°C", 10, 's'),
	(0x76C7, "Lufttemperatur unkorrigiert", "°C", 10, 's'),
	(0x76C9, "Windchill Temperatur Gültig wenn Lufttemperatur <10°C.", "°C", 10, 's'),
	(0x76CB, "Hitze Index Temperatur Gültig wenn Lufttemperatur >26°C und rel.Feuchte >40%", "°C", 10, 's'),
	(0x7789, "Rel. Feuchte", "%r.F.", 10, 'u'),
	(0x778B, "Absolute Feuchte", "g/m^3", 100, 'u'),
	(0x778D, "Taupunkttemperatur", "°C", 10, 's'),
	(0x778F, "Rel. Feuchte unkorrigiert", "%r.F.", 10, 'u'),
	(0x7851, "Absoluter Luftdruck", "hPa", 100, 'u'),
	(0x7853, "relativer Luftdruck bezogen auf NHN", "hPa", 100, 'u'),
	(0x7919, "Globalstrahlung", "W/m^2", 10, 's'),
	(0x79E1, "Helligkeit Nord (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x79E3, "Helligkeit Ost (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x79E5, "Helligkeit Süd (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x79E7, "Helligkeit West (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x79EB, "Richtung der Helligkeit Achtung: Bei diffusen Strahlungsverhältnissen benutzen Sie bitte den Parameter Sonnenstand Azimut", "°", 1, 'u'),
	(0x79ED, "Helligkeit, größter Wert der 4 Einzelsensoren", "kLux", 10, 'u'),
	(0x79EF, "Helligkeit Nord (feste Gerätezuordnung)", "Lux", 1, 'u'),
	(0x79F1, "Helligkeit Ost (feste Gerätezuordnung)", "Lux", 1, 'u'),
	(0x79F3, "Helligkeit Süd (feste Gerätezuordnung)", "Lux", 1, 'u'),
	(0x79F5, "Helligkeit West (feste Gerätezuordnung)", "Lux", 1, 'u'),
	(0x79F7, "Helligkeit, größter Wert der 4 Einzelsensoren", "Lux", 1, 'u'),
	(0x79F9, "Helligkeit, vektorielle Summe", "Lux", 1, 'u'),
	(0x7AA9, "Niederschlagsereignis", "", 1, 'u'),
	(0x7AAB, "Niederschlagsintensität (der letzten Minute auf die Stunde hoch-gerechnet)", "mm/h", 1000, 'u'),
	(0x7AAD, "Niederschlagsmenge wird um 24:00 Uhr zurückgesetzt)", "mm/h", 1000, 'u'),
	(0x7AAF, "Niederschlagsart", "Synop Code", 1, 'u'),
	(0x7AB1, "Niederschlagsmenge absolut (Überlauf bei 1000.000)", "mm", 1000, 'u'),
	(0x8729, "Datum", "", 1, 'u'),
	(0x872B, "Uhrzeit", "", 1, 'u'),
	(0x87F1, "Längengrad", "°", 1000000, 's'),
	(0x87F3, "Breitengrad", "°", 1000000, 's'),
	(0x87F5, "Sonnenstand Elevation", "°", 10, 's'),
	(0x87F7, "Sonnenstand Azimut", "°", 10, 's'),
	(0x87F9, "Höhe über NN", "m", 1, 's'),
	(0x87FB, "Sensorstatus der Windmessung", "", 1, 'u'),
	(0x8815, "Magnetkompass Differenzwinkel Geräte-Nordmarkierung zum magnetischen Nordpol", "°", 10, 'u'),
	(0x8817, "Pitch vom Magnetkompass Winkel zwischen Nord-Süd zur Horizontalen", "°", 10, 's'),
	(0x8819, "Roll vom Magnetkompass Winkel zwischen West-Ost zur Horizontalen", "°", 10, 's'),
	(0x88B3, "Sensorversorgung", "V", 10, 'u'),
	(0x88B5, "Live Counter", "ms", 1, 'u'),
	(0x88B7, "Fehlerstatus des letzten Messwertes", "", 1, 'u'),
	(0x88B9, "Mittelwert Windgeschwindigkeit", "m/s", 10, 'u'),
	(0x88BB, "Mittelwert Windrichtung", "°", 10, 'u'),
	(0x88BD, "Lufttemperatur", "°C", 10, 's'),
	(0x88BF, "Gehäuseinnentemperatur", "°C", 10, 's'),
	(0x88C1, "Akustische Temperatur", "°C", 10, 's'),
	(0x88C3, "Lufttemperatur unkorrigiert", "°C", 10, 's'),
	(0x88C5, "Rel. Feuchte", "%r.F.", 10, 'u'),
	(0x88C7, "Taupunkttemperatur", "°C", 10, 's'),
	(0x88C9, "Absoluter Luftdruck", "hPa", 100, 'u'),
	(0x88CB, "relativer Luftdruck bezogen auf Meereshöhe", "hPa", 100, 'u'),
	(0x88CD, "Helligkeit Nord (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x88CF, "Helligkeit Ost (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x88D1, "Helligkeit Süd (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x88D3, "Helligkeit West (feste Gerätezuordnung)", "kLux", 10, 'u'),
	(0x88D5, "Richtung der Helligkeit Achtung: Bei diffusen Strahlungsverhältnissen benutzen Sie bitte den Parameter Sonnenstand Azimut", "°", 1, 'u'),
	(0x88D7, "Helligkeit, größter Wert der 4 Einzelsensoren", "kLux", 10, 'u'),
	(0x88D9, "Niederschlagsereignis", "", 1,'u'),
	(0x88DB, "Niederschlagsintensität (der letzten Minute auf die Stunde hoch-gerechnet)", "mm/h", 1000,'u'),
	(0x88DD, "Niederschlagsmenge (wird um 24:00 Uhr zurückgesetzt)", "mm/h", 1000,'u'),
	(0x88DF, "Niederschlagsart", "Synop Code", 1,'u'),
	(0x88E1, "Datum", "", 1,'u'),
	(0x88E3, "Uhrzeit", "", 1,'u'),
	(0x88E5, "Längengrad", "°", 1000000,'s'),
	(0x88E7, "Breitengrad", "°", 1000000,'s'),
	(0x88E9, "Sonnenstand Elevation", "°", 10,'s'),
	(0x88EB, "Sonnenstand Azimut", "°", 10,'s'),
	(0x88ED, "Höhe über NN", ",", 1,'s'),
	(0x88EF, "Sensorstatus der Windmessung", "", 1,'u'),
	(0x88F1, "Sensorversorgung", "V", 10,'u'),
	(0x88F3, "Live Counter", "ms", 1,'u'),
	(0x88F5, "Fehlerstatus des letzten Messwerte", "", 1,'u'),
	(0x88F7, "Helligkeit Nord feste Gerätezuordnung)", "Lux", 1,'u'),
	(0x88F9, "Helligkeit Ost (feste Gerätezuordnung)", "Lux", 1,'u'),
	(0x88FB, "Helligkeit Süd (feste Gerätezuordnung)", "Lux", 1,'u'),
	(0x88FD, "Helligkeit West (feste Gerätezuordnung)", "Lux", 1,'u'),
	(0x88FF, "Helligkeit, größter Wert der 4 Einzelsensoren", "Lux", 1,'u'),
	(0x8901, "Maximalwert der Wind-geschwindigkeit (Böe) verfügbar wenn AV>=30", "m/s", 10,'u'),
	(0x8903, "Windrichtung der Böe verfügbar wenn AV>=30", "°", 10,'u'),
	(0x8905, "Absolute Feuchte", "g/m^3", 100,'u'),
	(0x8907, "Rel. Feuchte unkorrigiert", "%r.F.", 10,'u'),
	(0x8909, "Magnetkompass Differenzwinkel Geräte-Nordmarkierung zum magnetischen Nordpol", "°", 10,'u'),
	(0x890B, "Helligkeit, vektorielle Summe", "Lux", 1,'u'),
	(0x890D, "Windchill Temperatur Gültig wenn Lufttemperatur <10°C", "°C", 10,'s'),
	(0x890F, "Hitze Index Temperatur Gültig wenn Lufttemperatur >26°C und rel.Feuchte >40%", "°C", 10,'s'),
	(0x8911, "Niederschlagsmenge absolut (Überlauf bei 1000.000)", "mm", 1000,'u'),
	(0x8913, "Globalstrahlung", "W/m^2", 10,'s'),
	(0x8915, "Pitch vom Magnetkompass Winkel zwischen Nord-Süd zur Horizontalen", "°", 10,'s'),
	(0x8917, "Roll vom Magnetkompass Winkel zwischen West-Ost zur Horizontalen", "°", 10,'s'),
];

///All x hold registers
const HOLD_REG: [(u16, &'static str, &'static str); 18] = [
	(40015, "Befehl AV", "Mittelungsintervall für Windgeschwindigkeit und Windrichtung. 0..6000 (x100ms)"),
	(40031, "Befehl BP", "Parität, s. Befehl „BP“ Thies Format"),
	(40005, "Befehl BR", "Baudrate, s. Befehl „BR“ Thies Format"),
	(40013, "Befehl CI", "Kommandointerpreter, s. Befehl „CI“ThiesFormat"),
	(40011, "Befehl DM", "Duplex-Modus, s. Befehl „DM“ Thies Format"),
	(40023, "Befehl HC", "Heizungsbedingung"),
	(40025, "Befehl HS", "Höheneinstellung"),
	(40027, "Befehl HT", "Heizungssteuerung"),
	(40003, "Befehl ID", "Identifikationsnummer / Slave-Adresse"),
	(40009, "Befehl KY", "Schlüssel / Passwort setzen (Admin = 4711)"),
	(40029, "Befehl MC", "Magnetkompass Korrektur, Gehäuse zu Sensor(0..359°)"),
	(40017, "Befehl NC", "Nordkorrektur der Windrichtung (0..359°) / 1000 = automatische Richtungskorrektur nach Magnetkompass"),
	(40253, "Befehl RS", "Reset: 1 -> Warmstart 2 -> Tagessumme Niederschlag = 0"),
	(40019, "Befehl SH", "Stationshöhe (0...9000m)"),
	(40007, "Befehl SN", "Seriennummer"),
	(45005, "Befehl SV", "Software Version z.B.: 160 = V1.60"),
	(45001, "Befehl TA", "Thies Artikelnummer z.B: 4.9200.00.000 (64Bit)"),
	(40021, "Befehl TZ", "Zeitzone, s. Befehl „TZ“ Thies Format"),
];

fn conv_vec_to_value_s(vec: Vec<u16>) -> i32 {
	let usign_val: u32 = (vec[0] as u32) << 16 | (vec[1] as u32);
	usign_val as i32
}

fn conv_vec_to_value_u(vec: Vec<u16>) -> u32 {
	let usign_val = (vec[0] as u32) << 16 | (vec[1] as u32);
	usign_val
}

fn main() {
	// Modbus-Verbindung initialisieren
	let mut ctx = Modbus::new_rtu(DEVICE, BAUDRATE, PARITY, DATA_BITS, STOP_BITS).unwrap();
	ctx.set_slave(SLAVE_ID).expect("Setting Slave-ID failed!");
	assert!(ctx.rtu_set_serial_mode(SerialMode::RtuRS232).is_ok());
	assert!(ctx.rtu_set_rts(RequestToSendMode::RtuRtsUp).is_ok());
	assert!(ctx.rtu_set_custom_rts(RequestToSendMode::RtuRtsUp).is_ok());

	// Modbus-Verbindung öffnen
	ctx.connect().expect("Verbindung mit ctx Fehlerhaft!");

	// reading input registers
	println!("--- INPUT REGISTERS ---");
	println!("Reg-Address - Parametername : value ");
	for input_reg in INPUT_REG.iter() {
		let mut data = vec![0u16; 2];
		match ctx.read_input_registers(input_reg.0, 2, &mut data) {
			Ok(_) => {
				let conv_data: f32;
				if input_reg.4 == 'u' {
					conv_data = (conv_vec_to_value_u(data) as f32) / input_reg.3 as f32;
				} else {
					conv_data = (conv_vec_to_value_s(data) as f32) / input_reg.3 as f32;
				}
				println!(
					"{} - {} : {} {}",
					input_reg.0, input_reg.1, conv_data, input_reg.2
				);
			}
			Err(_) => {
				println!("{} - {} : couldn't read data", input_reg.0, input_reg.1);
			}
		}
	}

	// reading hold registers
	println!("--- HOLD REGISTERS ---");
	println!("Reg-Address - Parametername : value ");
	for hold_reg in HOLD_REG.iter() {
		let mut data = vec![0u16; 4];
		match ctx.read_registers(hold_reg.0, 4, &mut data) {
			Ok(_) => {
				if hold_reg.0 == 45001 {
					let conv_data: u64 = (data[0] as u64) << 48
						| (data[1] as u64) << 32 | (data[2] as u64) << 16
						| (data[3] as u64);
					println!("{} - {} -{} : {}", hold_reg.0, hold_reg.1, hold_reg.2, conv_data);
				} else {
					let conv_data: u32 = conv_vec_to_value_u(vec![data[0], data[1]]);
					println!("{} - {} - {} : {}", hold_reg.0, hold_reg.1, hold_reg.2, conv_data);
				}
			}
			Err(error) => {
				println!("{} - {} - {} : couldn't read data '{}'", hold_reg.0, hold_reg.1, hold_reg.2, error.to_string());
			}
		}
	}

	// Modbus-Verbindung schließen
	ctx.close();
	ctx.free();
}
