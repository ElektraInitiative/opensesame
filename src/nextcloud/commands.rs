use std::process::exit;

use crate::{
	audio::{self, AudioEvent},
	buttons::CommandToButtons,
};
use chrono::Local;
use tokio::sync::mpsc::Sender;

use super::nextcloud::NextcloudEvent;

#[derive(Debug)]
pub enum CommandType {
	Action,
	Info,
}

// Commands have this structure:
// [@user](command)(!|?)
#[derive(Debug)]
pub struct Command {
	user_prefix: Option<String>, // @user
	words: Vec<String>,          // the actual command
	command_type: CommandType,   // ! = Action ? = Info
}

impl Command {
	pub fn new(user_input: &str) -> Result<Command, String> {
		eprintln!("user_input: {}", user_input);
		let input_trimmed = user_input.trim();

		let command_type = match input_trimmed.chars().last() {
			Some('!') => CommandType::Action,
			Some('?') => CommandType::Info,
			_ => return Err(String::from("Command must end with either '!' or '?'.")),
		};

		let mut words = input_trimmed[..input_trimmed.len() - 1]
			.split_whitespace()
			.map(|word| word.to_lowercase())
			.collect::<Vec<String>>();

		let user_prefix = match words.get(0) {
			None => None,
			Some(first_word) => {
				if first_word.starts_with("@") {
					Some(first_word.clone())
				} else {
					None
				}
			}
		};

		if user_prefix.is_some() {
			words.drain(0..1);
		}

		Ok(Command {
			user_prefix,
			words,
			command_type,
		})
	}
}

pub async fn run_command(
	command: Command,
	nextcloud_sender: Sender<NextcloudEvent>,
	command_sender: Sender<CommandToButtons>,
	audio_sender: Sender<AudioEvent>,
	user: &str,
) -> String {
	if let Some(user_prefix) = command.user_prefix {
		if user_prefix != user {
			return "".to_string();
		}
	}

	let words = command
		.words
		.iter()
		.map(|word| word.as_str())
		.collect::<Vec<&str>>();

	match command.command_type {
		CommandType::Info => match words[..] {
			[] => include_str!("command_list.txt").to_owned(),
			["open"] | ["offen"] => todo!(),       // show_door_status(),
			["lights"] | ["licht"] => todo!(),     // switch_lights(),
			["battery"] | ["batterie"] => todo!(), // show_battery_status(),
			["weather"] | ["wetter"] => todo!(),   // show_weather(),
			["indoor", "climate"] | ["innenklima"] => todo!(), // report_indoor_climate(),
			["status"] => todo!(),                 // report_nextcloud_status(),
			["sensors"] | ["sensoren"] => todo!(), // report_sensor_data(),
			["time"] | ["uhrzeit"] => Local::now().to_string(),
			["code"] | ["pin"] => todo!(), // list_codes(),
			_ => String::from("Unknown command!"),
		},
		CommandType::Action => match words[..] {
			["open"] | ["öffnen"] => {
				command_sender
					.send(CommandToButtons::OpenDoor)
					.await
					.unwrap();
				"Sending open door command...".to_owned()
			} // open_door(),
			["lights", "in"] | ["licht", "innen"] => todo!(), // lights,
			["lights", "out"] | ["licht", "aussen"] => todo!(), // lights,
			["play", audio_file_path] | ["abspielen", audio_file_path] => {
				audio_sender
					.send(AudioEvent::PlayFile(audio_file_path.to_owned()))
					.await
					.unwrap();
				format!("Started playing audio file {}", audio_file_path).to_owned()
			}
			["bell"] | ["glocke"] => {
				audio_sender.send(AudioEvent::Bell).await.unwrap();
				"Started bell audio...".to_owned()
			}
			["alarm"] => {
				audio_sender.send(AudioEvent::FireAlarm).await.unwrap();
				"Started fire alarm sound...".to_owned()
			}
			["all", "clear"] | ["entwarnen"] => {
				audio_sender.send(AudioEvent::CancelAll).await.unwrap();
				"Canceled audio events...".to_owned()
			}

			["quit"] | ["beenden"] => exit(0),
			["code", "add", name, pin] | ["pin", "hinzufügen", name, pin] => {
				todo!() //add_code(name, pin)
			}
			["code", "del"] | ["pin", "löschen"] => todo!(), //delete_code(),
			["code", "set"] | ["pin", "ändern"] => todo!(),  //set_code(),
			_ => String::from("Unknown command!"),
		},
	}
}
