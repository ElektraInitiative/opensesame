use futures::never::Never;

use gettextrs::gettext;
use tokio::{
	io,
	process::Command,
	spawn,
	sync::mpsc::{Receiver, Sender},
};
use tokio_util::sync::CancellationToken;

use crate::{
	nextcloud::{NextcloudChat, NextcloudEvent},
	ssh::exec_ssh_command,
	types::ModuleError,
};

// play audio file with argument. If you do not have an argument, simply pass --quiet again
async fn play_audio_file(
	file: String,
	_arg: &str,
	cancellation_token: CancellationToken,
) -> Result<(), io::Error> {
	if file != "/dev/null" {
		let mut command = Command::new("ogg123").arg("--quiet").arg(file).spawn()?;

		// Wait for the process to finish
		let _ = tokio::select! {
			result = command.wait() => result,
			_ = cancellation_token.cancelled() => {
				// If cancellation is requested, kill the audio playback process
				let _ = command.kill().await;
				return Ok(());
			}
		};
	}
	Ok(())
}

pub enum AudioEvent {
	Bell,
	FireAlarm,
}

pub struct Audio {
	bell_path: String,
	fire_alarm_path: String,
}

impl Audio {
	pub fn new(bell_path: String, fire_alarm_path: String) -> Self {
		Audio {
			bell_path,
			fire_alarm_path,
		}
	}

	pub async fn get_background_task(
		self,
		mut audio_receiver: Receiver<AudioEvent>,
		nextcloud_sender: Sender<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		let mut maybe_cancellation_token: Option<CancellationToken> = Option::None;

		while let Some(event) = audio_receiver.recv().await {
			if maybe_cancellation_token.is_some() {
				maybe_cancellation_token.unwrap().cancel();
			};
			maybe_cancellation_token = Option::Some(CancellationToken::new());
			match event {
				AudioEvent::Bell => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							gettext("🔔 Ringing the Audio Bell"),
						))
						.await?;
					spawn(play_audio_file(
						self.bell_path.clone(),
						"",
						maybe_cancellation_token.clone().unwrap(),
					));
				}
				AudioEvent::FireAlarm => {
					nextcloud_sender
						.send(NextcloudEvent::Chat(
							NextcloudChat::Default,
							gettext("🚨 Audio Fire Alarm!"),
						))
						.await?;
					spawn(play_audio_file(
						self.fire_alarm_path.clone(),
						"--repeat",
						maybe_cancellation_token.clone().unwrap(),
					));
					let nextcloud_sender_clone = nextcloud_sender.clone();
					spawn(async move {
						let ssh_result =
							exec_ssh_command(String::from("killall -SIGUSR2 opensesame")).await;
						if let Err(err) = ssh_result {
							let _ = nextcloud_sender_clone
								.send(NextcloudEvent::Chat(
									NextcloudChat::Ping,
									gettext!(
										"Couldn't send SIGUSR2 to other opensesame instance: {}",
										err
									),
								))
								.await;
						}
					});
				}
			}
		}
		Err(ModuleError::new(String::from(
			"audio background task exited",
		)))
	}
}
