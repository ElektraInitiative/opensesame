use crate::nextcloud::NextcloudChat;
use crate::{config::Config, nextcloud::NextcloudEvent, types::ModuleError};
use futures::never::Never;
use gettextrs::gettext;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Sender;

const ALPHA: f64 = 0.6;

#[derive(Debug)]
struct Sensor {
	// static data
	loc: String,
	quality: String,
	pin: String,

	// static data from calibration
	chat: Option<u16>,
	alarm: Option<u16>,

	// reliability data (static) recorded during calibration, or also later
	min: u16,
	avg: u16,
	max: u16,

	// dynamic data
	value: u16,
	expmovavg: f64, // exponential moving average with ALPHA
	triggered: SensorsChange,
}

pub struct Sensors {
	init: bool,
	sensors: [Sensor; 12],
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorsChange {
	None,
	Chat(String),
	Alarm(String),
}

const BELL_JUMP: f64 = 40f64;
const ALARM_JUMP: f64 = 60f64;

impl Config<'_> {
	fn get_sensor_element_option<T: FromStr + Default>(&mut self, nr: u8, name: &str) -> Option<T> {
		self.get_option::<T>(&format!("sensors/#{}/{}", nr, name).to_string())
	}

	fn get_sensor_element<T: FromStr + Default>(&mut self, nr: u8, name: &str) -> T {
		match self.get_sensor_element_option::<T>(nr, name) {
			Some(value) => value,
			None => T::default(),
		}
	}

	fn get_sensor(&mut self, nr: u8) -> Sensor {
		Sensor {
			loc: self.get_sensor_element::<String>(nr, "loc"),
			quality: self.get_sensor_element::<String>(nr, "quality"),
			pin: self.get_sensor_element::<String>(nr, "pin"),

			chat: self.get_sensor_element_option::<u16>(nr, "chat"),
			alarm: self.get_sensor_element_option::<u16>(nr, "alarm"),

			min: self.get_sensor_element::<u16>(nr, "min"),
			avg: self.get_sensor_element::<u16>(nr, "avg"),
			max: self.get_sensor_element::<u16>(nr, "max"),

			value: 0,
			expmovavg: 0f64,
			triggered: SensorsChange::None,
		}
	}
}

impl Sensors {
	pub fn new(config: &mut Config) -> Self {
		Self {
			init: false,
			sensors: [
				config.get_sensor(0),
				config.get_sensor(1),
				config.get_sensor(2),
				config.get_sensor(3),
				config.get_sensor(4),
				config.get_sensor(5),
				config.get_sensor(6),
				config.get_sensor(7),
				config.get_sensor(8),
				config.get_sensor(9),
				config.get_sensor(10),
				config.get_sensor(11),
			],
		}
	}

	fn text(&mut self, i: usize, text: &str) -> String {
		let mut ret: String = format!(
			"Sensor #{} {} Quality: {} Pin: {}",
			i, self.sensors[i].loc, self.sensors[i].quality, self.sensors[i].pin
		);

		if let Some(chat) = self.sensors[i].chat {
			ret += &format!(" Chat: {}", chat);
		}

		if let Some(alarm) = self.sensors[i].alarm {
			ret += &format!(" Alarm: {}", alarm);
		}

		ret += &format!(
			" Min: {} Avg: {} Max: {} Text: {}",
			self.sensors[i].min, self.sensors[i].avg, self.sensors[i].max, text
		);
		ret
	}

	fn threstext(&mut self, i: usize, text: &str, value: u16) -> String {
		format!("Threshold {} Value: {}", self.text(i, text), value)
	}

	fn jumptext(
		&mut self,
		i: usize,
		text: &str,
		value: u16,
		expmovavg: f64,
		prev_expmovavg: f64,
	) -> String {
		format!(
			"Jump {} expmovavg: {} prev_expmovavg: {} Value: {}",
			self.text(i, text),
			expmovavg,
			prev_expmovavg,
			value
		)
	}

	pub fn update(&mut self, line: String) -> SensorsChange {
		let values: Vec<u16> = line
			.split_whitespace()
			.map(|s| s.parse().unwrap())
			.collect();
		assert!(values.len() == 12);

		let mut ret = SensorsChange::None;
		for i in 0..12 {
			if self.sensors[i].loc.is_empty() {
				continue;
			}

			// Alarms by jumps (strongly raising)
			if self.init {
				let expmovavg =
					ALPHA * values[i] as f64 + (1.0f64 - ALPHA) * self.sensors[i].value as f64;
				let prev_expmovavg = self.sensors[i].expmovavg;
				match self.sensors[i].triggered {
					SensorsChange::Alarm(_) => {}
					SensorsChange::Chat(_) => {
						if expmovavg >= prev_expmovavg + BELL_JUMP {
							ret = SensorsChange::Alarm(self.jumptext(
								i,
								"2xChat => Alarm",
								values[i],
								expmovavg,
								prev_expmovavg,
							));
							self.sensors[i].triggered = ret.clone();
							return ret; // we got an alarm, don't process anything further
						} else if expmovavg < prev_expmovavg {
							// cancel chat
							self.sensors[i].triggered = SensorsChange::None;
						}
					}
					SensorsChange::None => {
						if expmovavg >= prev_expmovavg + BELL_JUMP {
							ret = SensorsChange::Chat(self.jumptext(
								i,
								"chat jump",
								values[i],
								expmovavg,
								prev_expmovavg,
							));
							self.sensors[i].triggered = ret.clone();
						} else if expmovavg >= prev_expmovavg + ALARM_JUMP {
							ret = SensorsChange::Alarm(self.jumptext(
								i,
								"alarm jump",
								values[i],
								expmovavg,
								prev_expmovavg,
							));
							self.sensors[i].triggered = ret.clone();
							return ret; // we got an alarm, don't process anything further
						}
					}
				}
				self.sensors[i].expmovavg = expmovavg;
			} else {
				self.sensors[i].expmovavg = values[i] as f64;
			}

			self.sensors[i].value = values[i];

			// Alarms by threshold
			if let Some(alarm) = self.sensors[i].alarm {
				if let Some(chat) = self.sensors[i].chat {
					match self.sensors[i].triggered {
						SensorsChange::Alarm(_) => (),
						SensorsChange::Chat(_) => {
							if values[i] >= alarm {
								ret = SensorsChange::Alarm(self.threstext(
									i,
									"from chat -> alarm threshold",
									values[i],
								));
								self.sensors[i].triggered = ret.clone();
								return ret; // we got an alarm, don't process anything further
							} else if values[i] < chat {
								// cancel chat
								self.sensors[i].triggered = SensorsChange::None;
							}
						}
						SensorsChange::None => {
							if values[i] >= alarm {
								ret = SensorsChange::Alarm(self.threstext(
									i,
									"from none -> alarm",
									values[i],
								));
								self.sensors[i].triggered = ret.clone();
								return ret; // we got an alarm, don't process anything further
							} else if values[i] >= chat {
								ret = SensorsChange::Chat(self.threstext(
									i,
									"from none -> chat",
									values[i],
								));
								self.sensors[i].triggered = ret.clone();
							}
						}
					} // end of match
				}
			} // end of if let
		}
		self.init = true;
		ret
	}

	pub async fn get_background_task(
		mut self,
		device_path: String,
		nextcloud_sender: Sender<NextcloudEvent>,
		//state_mutex: Arc<Mutex<Config<'_>>>,
		//pid: u32,
	) -> Result<Never, ModuleError> {
		let device_file = File::open(device_path).await.expect("error here");
		let reader = BufReader::new(device_file);

		let mut lines = reader.lines();
		while let Some(line) = lines.next_line().await? {
			match self.update(line.clone()) {
				SensorsChange::None => (),
				SensorsChange::Alarm(w) => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							gettext!("Fire Alarm {}", w),
						))
						.await?;
					/*let mut state = state_mutex.lock().await;
					state.set("alarm/fire", &w.to_string());
					kill(nix::unistd::Pid::from_raw(pid as i32), Signal::SIGHUP)?;
					spawn(exec_ssh_command(format!(
						"kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"",
						w
					)));*/
				}
				SensorsChange::Chat(w) => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							gettext!("Fire Chat {}", w),
						))
						.await?;
				}
			}
		}
		Err(ModuleError::new(String::from("sensors_loop exited")))
	}
}

#[cfg(test)]
mod tests {
	// Note this useful idiom: importing names from outer (for mod tests) scope.
	use super::*;

	use serial_test::serial;
	use std::env;

	const CONFIG_PARENT: &str = "/sw/libelektra/opensesame/#0/current";

	#[ignore]
	#[test]
	#[serial]
	fn test_should_not_trigger() {
		let mut config: Config = Config::new(CONFIG_PARENT);
		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
		config.cut("sensors");
		config.add("sensors/#0/loc", "artificial raise");
		config.add("sensors/#1/loc", "recorded up/down");
		config.add("sensors/#2/loc", "recorded up/down");
		config.add("sensors/#3/loc", "recorded up/down");
		config.add("sensors/#4/loc", "recorded up/down");
		config.add("sensors/#5/loc", "recorded up/down");
		config.add("sensors/#6/loc", "all zeros");
		config.add("sensors/#7/loc", ""); // would trigger, but is ignored
		config.add("sensors/#8/loc", "recorded up/down");
		config.add("sensors/#9/loc", "all 39");
		config.add("sensors/#10/loc", "all 722");
		config.add("sensors/#11/loc", "all 1555");

		let mut sensors = Sensors::new(&mut config);
		assert_eq!(sensors.update("77       194    127     98     82      81       0       100     79      39      722      1555".to_string()), SensorsChange::None);
		assert_eq!(sensors.update("77       329    179     138    82      127      0       200     79      39      722      1555".to_string()), SensorsChange::Chat("Sensor #1 recorded up/down Quality:  Pin:  Chat: 0 Alarm: 0 Min: 0 Avg: 0 Max: 0 Text: chat jump expmovavg: 275 prev_expmovavg: 194 Value: 329".to_string()));
		assert_eq!(sensors.update("100      268    165     119    82      122      0       300     78      39      722      1555".to_string()), SensorsChange::None);
		assert_eq!(sensors.update("150      216    166     119    81      129      0       400     110     39      722      1555".to_string()), SensorsChange::None);
		assert_eq!(sensors.update("100      206    148     119    132     123      0       500     92      39      722      1555".to_string()), SensorsChange::None);
		assert_eq!(sensors.update("100      200    148     119    97      94       0       600     134     39      722      1555".to_string()), SensorsChange::None);
		assert_eq!(sensors.update("100      200    148     119    97      125      0       700     124     39      722      1555".to_string()), SensorsChange::None);
	}

	#[ignore]
	#[test]
	#[serial]
	fn real_fire_1() {
		let mut config: Config = Config::new(CONFIG_PARENT);
		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
		config.cut("sensors");
		config.add("sensors/#0/loc", "0");
		config.add("sensors/#1/loc", "1");
		config.add("sensors/#2/loc", "2");
		config.add("sensors/#3/loc", "3");
		config.add("sensors/#4/loc", "4");
		config.add("sensors/#5/loc", "5");
		config.add("sensors/#6/loc", "6");
		config.add("sensors/#7/loc", "7");
		config.add("sensors/#8/loc", "8");
		config.add("sensors/#9/loc", "9");
		config.add("sensors/#10/loc", "10");
		config.add("sensors/#11/loc", "11");

		let mut sensors = Sensors::new(&mut config);
		assert_eq!(
			sensors.update("152	237	279	275	177	166	90	440	59	370	423	9".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("153	237	279	258	177	166	106	441	81	370	429	22".to_string()),
			SensorsChange::None
		);
		assert_eq!(sensors.update("293	305	440	419	296	274	265	565	215	513	548	80".to_string()), SensorsChange::Chat("Jump Sensor #11 11 Quality:  Pin:  Chat: 0 Alarm: 0 Min: 0 Avg: 0 Max: 0 Text: chat jump expmovavg: 56.8 prev_expmovavg: 16.8 Value: 80".to_string()));
		assert_eq!(sensors.update("340	349	505	426	356	369	364	628	344	576	594	145".to_string()), SensorsChange::Alarm("Jump Sensor #0 0 Quality:  Pin:  Chat: 0 Alarm: 0 Min: 0 Avg: 0 Max: 0 Text: 2xChat => Alarm expmovavg: 321.2 prev_expmovavg: 237 Value: 340".to_string()));
		assert_eq!(
			sensors.update("340	366	495	463	339	389	372	654	369	598	597	155".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("371	388	514	465	348	395	392	676	410	625	618	180".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("393	399	505	423	357	403	395	692	426	642	629	187".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("359	395	491	414	352	391	357	693	370	639	595	144".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("318	374	453	380	310	352	313	637	296	588	570	108".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("304	365	439	368	298	328	283	618	244	563	562	82".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("290	361	421	376	285	312	259	596	220	533	552	76".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("270	353	393	347	264	278	218	571	169	487	536	53".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("249	347	362	338	240	241	175	554	118	454	518	30".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("236	342	343	341	228	227	160	545	103	442	507	25".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("224	339	330	294	217	215	146	537	89	433	496	20".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("216	335	318	304	209	207	138	533	83	429	489	18".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("208	330	309	324	203	201	129	527	76	424	482	15".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("201	326	301	275	197	195	122	524	73	423	477	14".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("196	330	303	298	192	190	115	517	69	416	469	12".to_string()),
			SensorsChange::None
		);
	}

	#[ignore]
	#[test]
	#[serial]
	fn real_fire_2() {
		let mut config: Config = Config::new(CONFIG_PARENT);
		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
		config.cut("sensors");
		config.add("sensors/#0/loc", "0");
		config.add("sensors/#1/loc", "1");
		config.add("sensors/#2/loc", "2");
		config.add("sensors/#3/loc", "3");
		config.add("sensors/#4/loc", "4");
		config.add("sensors/#5/loc", "5");
		config.add("sensors/#6/loc", "6");
		config.add("sensors/#7/loc", "7");
		config.add("sensors/#8/loc", "8");
		config.add("sensors/#9/loc", "9");
		config.add("sensors/#10/loc", "10");
		config.add("sensors/#11/loc", "11");

		let mut sensors = Sensors::new(&mut config);
		assert_eq!(
			sensors.update("148	219	260	266	257	242	71	441	44	344	486	5".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("152	222	264	264	261	245	100	449	78	349	501	22".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("220	277	345	318	309	309	164	515	137	406	566	56".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("281	332	429	380	351	354	209	564	182	466	600	85".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("320	369	482	410	376	382	249	591	228	505	630	105".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("338	391	491	424	391	416	251	598	201	512	634	95".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("345	397	492	418	398	415	252	607	203	522	641	94".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("362	424	510	433	415	433	277	616	221	535	653	106".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("363	426	511	433	420	437	282	621	224	542	658	107".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("363	427	510	437	422	433	277	626	220	546	660	107".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("372	440	516	448	431	449	305	631	236	553	666	117".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("368	436	512	438	430	436	303	633	230	556	664	113".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("369	434	511	433	435	445	312	636	238	559	665	116".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("367	428	503	428	433	442	310	638	232	560	663	112".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("362	414	497	430	438	443	306	638	225	560	658	110".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("361	404	498	421	435	440	305	640	226	561	657	110".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("364	408	502	429	438	441	314	644	235	568	663	117".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("363	404	502	435	439	441	311	646	235	573	665	125".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("376	426	514	448	447	456	360	655	302	586	689	175".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("387	455	518	448	449	453	351	658	284	590	692	160".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("387	466	516	442	459	464	357	657	294	591	693	162".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("351	397	478	416	435	424	299	651	219	580	666	106".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("306	338	431	376	405	374	222	638	138	540	635	55".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("238	279	355	332	357	310	138	572	76	454	589	22".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("227	267	338	316	351	301	125	565	65	443	584	17".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("173	206	271	281	287	260	80	504	55	383	514	10".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("172	206	271	275	286	263	80	503	54	382	513	10".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("172	206	272	269	285	261	80	502	54	381	512	10".to_string()),
			SensorsChange::None
		);
		assert_eq!(
			sensors.update("172	209	275	277	285	259	80	501	54	380	511	10".to_string()),
			SensorsChange::None
		);
	}

	#[ignore]
	#[test]
	#[serial]
	fn read_csv() {
		let mut config: Config = Config::new(CONFIG_PARENT);
		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));
		config.cut("sensors");
		config.add("sensors/#0/loc", "0");
		config.add("sensors/#1/loc", "1");
		config.add("sensors/#2/loc", "2");
		config.add("sensors/#3/loc", "3");
		config.add("sensors/#4/loc", "4");
		config.add("sensors/#5/loc", "5");
		config.add("sensors/#6/loc", "6");
		config.add("sensors/#7/loc", "7");
		config.add("sensors/#8/loc", "8");
		config.add("sensors/#9/loc", "9");
		config.add("sensors/#10/loc", "10");
		config.add("sensors/#11/loc", "11");

		let file = std::fs::File::open("data/test.csv").unwrap();
		use std::io::{prelude::*, BufReader};
		let reader = BufReader::new(file);

		let mut sensors = Sensors::new(&mut config);

		for l in reader.lines() {
			let line = l.unwrap();
			println!("{}", line);
			assert_eq!(sensors.update(line), SensorsChange::None);
		}
	}
}
