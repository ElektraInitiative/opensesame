async fn sigterm(){
	//send message to environment to remember_baseline ans ENV need to use a Mutex of State
	//How to enable env to use message recv, if it waits one minute or more for the next loop iteration?
	//Two different loops? but they then have different states if a restore baseline is called?
	//And only execute if env is enabled, otherwise we get an error
	environment.remember_baseline(&mut state);
	return Ok(());
}

async fn sighup(){
	//sensors
	sighup.store(false, Ordering::Relaxed);
	config.sync();
	state.sync();
	environment.restore_baseline(&mut state);
	nc.ping(gettext!(
		"👋 reloaded config&state in sensor mode for opensesame {} {}",
		env!("CARGO_PKG_VERSION"),
		startup_time
	));
	//Buttons
	nc.ping(gettext!(
		"👋reloading config&state for opensesame {} {}",
		env!("CARGO_PKG_VERSION"),
		startup_time
	));
	//Need to use Mutex on config and sync
	sighup.store(false, Ordering::Relaxed);
	config.sync();
	state.sync();
	//Environment gets mutex of state so we first call sync and then we send a Channel Message to Env and ENV calls restore_baseline
	environment.restore_baseline(&mut state);
	if let Some(alarm) = state.get_option::<String>("alarm/fire") {
		if alarm_not_active {
			nc.send_message(gettext!(
				"🚨 Fire Alarm! Fire Alarm! Fire ALARM! ⏰. {}",
				alarm
			));
			buttons.ring_bell_alarm(10);
			if config.get_bool("garage/enable") {
				play_audio_file(
					config.get::<String>("audio/alarm"),
					"--repeat".to_string(),
				);
				thread::Builder::new().name("killall to ring ALARM".to_string()).spawn(move || {
					exec_ssh_command(format!("kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"", alarm));
				}).unwrap();
			};
			alarm_not_active = false;
		}
	} else {
		// config option removed, go out of alarm mode
		alarm_not_active = true;
	}
}

async fn sigalarm(){
	//easy to implement, using Channels to play_audio_file and buttons and NC
	sigalrm.store(false, Ordering::Relaxed);
	buttons.ring_bell_alarm(20);
	play_audio_file(config.get::<String>("audio/alarm"), "--repeat".to_string());
	nc.send_message(gettext("🚨 Received alarm"));
}

async fn sigusr1(){
	//This sets the counter wait_for_ping bigger than wait_for_ping_timeout, so a Ping message is send to NC
	//Send Ping in NC, but how do we get the information of the other modules?
	// 	nc.ping (format!("{} Ping! Version {}, Watchdog {}, {}, Status {}, Error {}, Load {} {} {}, Memory usage {}, 
	// 	Swap {}, CPU temp {}, Startup {} Bat {}", ping_counter, env!("CARGO_PKG_VERSION"), watchdog.wait_for_watchdog_trigger, 
	//	environment.to_string(), environment.status, environment.error, loadavg.one, loadavg.five, loadavg.fifteen, 
	//	sys.memory().unwrap().total, sys.swap().unwrap().total, sys.cpu_temp().unwrap(), startup_time, bat));
	//
	//Need to implement Module Ping which sends ping in a specific interval
	sigusr1.store(false, Ordering::Relaxed);
	wait_for_ping = wait_for_ping_timeout + 1;
}

async fn sigusr2(){
	//easy to implement, using Channels to play_audio_file and buttons and NC
	sigusr2.store(false, Ordering::Relaxed);
	buttons.ring_bell(20, 0);
	nc.send_message(gettext("🔔 Received bell"));
	play_audio_file(config.get::<String>("audio/bell"), "--quiet".to_string());
}