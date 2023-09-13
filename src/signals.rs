use std::sync::Arc;

use futures::never::Never;
use gettextrs::gettext;
use signal::unix::signal;
use tokio::{sync::{mpsc::Sender, Mutex}, signal::{self, unix::SignalKind}, select};

use crate::{ping::PingEvent, types::ModuleError, buttons::CommandToButtons, nextcloud::NextcloudEvent, config::Config};

pub struct Signals{}

impl Signals{
	/*async fn sigterm(state: Arc<Mutex<Config<'_>>>) -> Result<(), ModuleError> {
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
	}*/

	async fn sigalarm(command_sender: &Sender<CommandToButtons>, buttons_enabled: bool, nextcloud_sender: &Sender<NextcloudEvent>) -> Result<(), ModuleError>{
		if buttons_enabled {
			command_sender.send(CommandToButtons::RingBellAlarm(20)).await?;
		}
		//Wird mit play audio modul implementiert
		//play_audio_file(config.get::<String>("audio/alarm"), "--repeat".to_string());
		nextcloud_sender.send(NextcloudEvent::Chat(gettext("🚨 Received alarm"))).await?;
		Ok(())
	}

	//Done
	async fn sigusr1(ping_sender: &Sender<PingEvent>) -> Result<(), ModuleError> {
		ping_sender.send(PingEvent::SendPing).await?;
		Ok(())
	}

	async fn sigusr2(command_sender: &Sender<CommandToButtons>, buttons_enabled: bool, nextcloud_sender: &Sender<NextcloudEvent>) -> Result<(), ModuleError>{
		if buttons_enabled {
			command_sender.send(CommandToButtons::RingBell(20,0)).await?;
		}
		//Wird mit play audio modul implementiert
		//	play_audio_file(config.get::<String>("audio/bell"), "--quiet".to_string());
		nextcloud_sender.send(NextcloudEvent::Chat(gettext("🔔 Received bell"))).await?;
		Ok(())
	}

	pub async fn get_background_task(ping_sender: Sender<PingEvent>, ping_enabled: bool, command_sender: Sender<CommandToButtons>, buttons_enabled: bool, nextcloud_sender: Sender<NextcloudEvent>) -> Result<Never, ModuleError> {
		let mut sig_usr1 = signal(SignalKind::user_defined1())?;
		let mut sig_usr2 = signal(signal::unix::SignalKind::user_defined2())?;
		let mut sig_alarm = signal(signal::unix::SignalKind::alarm())?;
		let mut sig_hanghup = signal(signal::unix::SignalKind::hangup())?;
		let mut sig_term = signal(signal::unix::SignalKind::terminate())?;

	loop {
		select! {
			_ = sig_usr1.recv() => {
				if ping_enabled {
					Signals::sigusr1(&ping_sender).await?;
				}
				println!("Received SIGUSR1");
			}
			_ = sig_usr2.recv() => {
				Signals::sigusr2(&command_sender, buttons_enabled, &nextcloud_sender).await?;
				println!("Received SIGUSR2");
			}
			_ = sig_alarm.recv() => {
				Signals::sigalarm(&command_sender, buttons_enabled, &nextcloud_sender).await?;
				println!("Received SIGALRM");
			}
			_ = sig_hanghup.recv() => {
				println!("Received SIGHUP");
			}
			_ = sig_term.recv() => {
				println!("Received SIGTERM");
			}
		}
	}	
	}
}