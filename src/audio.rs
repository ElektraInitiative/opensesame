use futures::never::Never;

use tokio::{io, process::Command, spawn, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{ssh::exec_ssh_command, types::ModuleError};

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
	) -> Result<Never, ModuleError> {
		let mut maybe_cancellation_token: Option<CancellationToken> = Option::None;

		while let Some(event) = audio_receiver.recv().await {
			if maybe_cancellation_token.is_some() {
				maybe_cancellation_token.unwrap().cancel();
			};
			maybe_cancellation_token = Option::Some(CancellationToken::new());
			eprintln!("Cancelling previous audio...");
			match event {
				AudioEvent::Bell => {
					eprintln!("Ringing bell...");
					spawn(play_audio_file(
						self.bell_path.clone(),
						"",
						maybe_cancellation_token.clone().unwrap(),
					));
				}
				AudioEvent::FireAlarm => {
					eprintln!("Starting alarm...");
					spawn(play_audio_file(
						self.fire_alarm_path.clone(),
						"--repeat",
						maybe_cancellation_token.clone().unwrap(),
					));
					spawn(async move {
						let ssh_result =
							exec_ssh_command(String::from("killall -SIGUSR2 opensesame")).await;
						if let Err(err) = ssh_result {
							eprintln!(
								"Couldn't send SIGUSR2 to other opensesame instance: {}",
								err
							);
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
