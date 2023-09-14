use std::sync::Arc;

use futures::never::Never;
use gettextrs::gettext;
use signal::unix::signal;
use tokio::{
	select,
	signal::{self, unix::SignalKind},
	spawn,
	sync::{mpsc::Sender, Mutex},
};

use crate::{
	buttons::CommandToButtons, config::Config, environment::EnvEvent, nextcloud::NextcloudEvent,
	ping::PingEvent, ssh::exec_ssh_command, types::ModuleError,
};

pub struct Signals<'a> {
	ping_sender: Sender<PingEvent>,
	ping_enabled: bool,
	command_sender: Sender<CommandToButtons>,
	buttons_enabled: bool,
	nextcloud_sender: Sender<NextcloudEvent>,
	environment_sender: Sender<EnvEvent>,
	alarm_not_active: bool,
	startup_time: String,
	config_mutex: Arc<Mutex<Config<'a>>>,
	state_mutex: Arc<Mutex<Config<'a>>>,
}

impl<'a> Signals<'a> {
	pub fn new(
		config_mutex: Arc<Mutex<Config<'a>>>,
		state_mutex: Arc<Mutex<Config<'a>>>,
		ping_enabled: bool,
		buttons_enabled: bool,
		startup_time: String,
		ping_sender: Sender<PingEvent>,
		command_sender: Sender<CommandToButtons>,
		nextcloud_sender: Sender<NextcloudEvent>,
		environment_sender: Sender<EnvEvent>,
	) -> Self {
		Self {
			ping_sender,
			command_sender,
			nextcloud_sender,
			environment_sender,
			alarm_not_active: true,
			startup_time,
			ping_enabled,
			buttons_enabled,
			config_mutex,
			state_mutex,
		}
	}

	async fn sigterm(&mut self) -> Result<(), ModuleError> {
		//send message to environment to remember_baseline ans ENV need to use a Mutex of State
		//How to enable env to use message recv, if it waits one minute or more for the next loop iteration?
		//Two different loops? but they then have different states if a restore baseline is called?
		//And only execute if env is enabled, otherwise we get an error
		self.environment_sender
			.send(EnvEvent::RememberBaseline)
			.await?;
		return Ok(());
	}

	async fn sighup(&mut self) -> Result<(), ModuleError> {
		self.nextcloud_sender
			.send(NextcloudEvent::Ping(gettext!(
				"👋reloading config&state for opensesame {} {}",
				env!("CARGO_PKG_VERSION"),
				self.startup_time
			)))
			.await?;

		let mut config = self.config_mutex.lock().await;
		let mut state = self.state_mutex.lock().await;
		config.sync();
		state.sync();

		self.environment_sender
			.send(EnvEvent::RestoreBaseline)
			.await?;

		self.nextcloud_sender
			.send(NextcloudEvent::Ping(gettext!(
				"👋 reloaded config&state in sensor mode for opensesame {} {}",
				env!("CARGO_PKG_VERSION"),
				self.startup_time
			)))
			.await?;

		//Environment gets mutex of state so we first call sync and then we send a Channel Message to Env and ENV calls restore_baseline
		if let Some(alarm) = state.get_option::<String>("alarm/fire") {
			if self.alarm_not_active {
				self.nextcloud_sender
					.send(NextcloudEvent::Chat(gettext!(
						"🚨 Fire Alarm! Fire Alarm! Fire ALARM! ⏰. {}",
						alarm
					)))
					.await?;
				self.command_sender
					.send(CommandToButtons::RingBellAlarm(10))
					.await?;
				if config.get_bool("garage/enable") {
					/*play_audio_file(
						config.get::<String>("audio/alarm"),
						"--repeat".to_string(),
					);*/
					spawn(exec_ssh_command(format!(
						"kdb set user:/state/libelektra/opensesame/#0/current/alarm/fire \"{}\"",
						alarm
					)));
				};
				self.alarm_not_active = false;
			}
		} else {
			// config option removed, go out of alarm mode
			self.alarm_not_active = true;
		}
		Ok(())
	}

	async fn sigalarm(&mut self) -> Result<(), ModuleError> {
		if self.buttons_enabled {
			self.command_sender
				.send(CommandToButtons::RingBellAlarm(20))
				.await?;
		}
		//Wird mit play audio modul implementiert
		//play_audio_file(config.get::<String>("audio/alarm"), "--repeat".to_string());
		self.nextcloud_sender
			.send(NextcloudEvent::Chat(gettext("🚨 Received alarm")))
			.await?;
		Ok(())
	}

	async fn sigusr1(&mut self) -> Result<(), ModuleError> {
		self.ping_sender.send(PingEvent::SendPing).await?;
		Ok(())
	}

	async fn sigusr2(&mut self) -> Result<(), ModuleError> {
		if self.buttons_enabled {
			self.command_sender
				.send(CommandToButtons::RingBell(20, 0))
				.await?;
		}
		//Wird mit play audio modul implementiert
		//	play_audio_file(config.get::<String>("audio/bell"), "--quiet".to_string());
		self.nextcloud_sender
			.send(NextcloudEvent::Chat(gettext("🔔 Received bell")))
			.await?;
		Ok(())
	}

	pub async fn get_background_task(mut self) -> Result<Never, ModuleError> {
		let mut sig_usr1 = signal(SignalKind::user_defined1())?;
		let mut sig_usr2 = signal(signal::unix::SignalKind::user_defined2())?;
		let mut sig_alarm = signal(signal::unix::SignalKind::alarm())?;
		let mut sig_hanghup = signal(signal::unix::SignalKind::hangup())?;
		let mut sig_term = signal(signal::unix::SignalKind::terminate())?;

		loop {
			select! {
				_ = sig_usr1.recv() => {
					if self.ping_enabled {
						self.sigusr1().await?;
					}
					println!("Received SIGUSR1");
				}
				_ = sig_usr2.recv() => {
					self.sigusr2().await?;
					println!("Received SIGUSR2");
				}
				_ = sig_alarm.recv() => {
					self.sigalarm().await?;
					println!("Received SIGALRM");
				}
				_ = sig_hanghup.recv() => {
					self.sighup().await?;
					println!("Received SIGHUP");
				}
				_ = sig_term.recv() => {
					self.sigterm().await?;
					println!("Received SIGTERM");
				}
			}
		}
	}
}
