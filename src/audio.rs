use futures::never::Never;
use tokio::{process::Command, spawn, sync::mpsc::Receiver, task::JoinHandle};

use crate::types::ModuleError;

// play audio file with argument. If you do not have an argument, simply pass --quiet again
async fn play_audio_file(file: String, arg: &str) -> Result<(), ModuleError> {
	if file != "/dev/null" {
		Command::new("ogg123")
			.arg("--quiet")
			.arg(file)
			.status()
			.await
			.unwrap();
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

	async fn get_background_task(
		self,
		mut audio_receiver: Receiver<AudioEvent>,
	) -> Result<Never, ModuleError> {
		let mut join_handler: Option<JoinHandle<Result<(), ModuleError>>> = Option::None;
		while let Some(event) = audio_receiver.recv().await {
			if let Some(handler) = join_handler {
				handler.abort();
			}
			join_handler = match event {
				AudioEvent::Bell => Some(spawn(play_audio_file(self.bell_path.clone(), ""))),
				AudioEvent::FireAlarm => Some(spawn(play_audio_file(
					self.fire_alarm_path.clone(),
					"--repeat",
				))),
			}
		}
		Err(ModuleError::new(String::from(
			"audio background task exited",
		)))
	}
}
