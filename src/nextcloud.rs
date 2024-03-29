use crate::{audio::AudioEvent, buttons::CommandToButtons, config::Config, types::ModuleError};
use futures::{never::Never, try_join};
use gettextrs::gettext;
use reqwest::{
	header::{HeaderMap, ACCEPT, CONTENT_TYPE},
	Client,
};
use std::collections::HashMap;
use tokio::{
	sync::mpsc::{Receiver, Sender},
	time::{self, interval},
};

pub enum NextcloudChat {
	Default,
	Ping,
	Licht,
}

pub enum NextcloudStatus {
	Online,
	Env,
	Door,
}

pub enum NextcloudEvent {
	Chat(NextcloudChat, String),
	SendStatus,
	Status(NextcloudStatus, String),
}

#[derive(Clone)]
pub struct Nextcloud {
	base_url: String,
	chat: String,
	chat_ping: String,
	chat_licht: String,
	chat_commands: String,
	user: String,
	pass: String,
	info_door: String,
	info_environment: String,
	info_online: String,
	client: Client,
	headers: HeaderMap,
	startup_time: String,
}

impl Nextcloud {
	pub fn new(config: &mut Config) -> Self {
		let mut headers = HeaderMap::new();
		headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
		headers.insert(ACCEPT, "application/json".parse().unwrap());
		headers.insert("OCS-APIRequest", "true".parse().unwrap());
		let client = reqwest::Client::new();
		Self {
			base_url: config.get::<String>("nextcloud/url"),
			chat: config.get::<String>("nextcloud/chat"),
			chat_ping: config.get::<String>("nextcloud/chat/ping"),
			chat_licht: config.get::<String>("nextcloud/chat/licht"),
			chat_commands: config.get::<String>("nextcloud/chat/commands"),
			user: config.get::<String>("nextcloud/user"),
			pass: config.get::<String>("nextcloud/pass"),
			info_door: String::new(),
			info_environment: String::new(),
			info_online: String::new(),
			client,
			headers,
			startup_time: String::new(),
		}
	}

	// sends once, Err if it does not work on network or nextcloud level
	async fn send_message_once(
		&self,
		message: &str,
		chat: &str,
	) -> Result<reqwest::Response, reqwest::Error> {
		let mut payload = HashMap::new();
		payload.insert("token", &chat);
		payload.insert("message", &message);
		let response = self
			.client
			.post(format!(
				"{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
				&self.base_url, chat
			))
			.basic_auth(&self.user, Some(&self.pass))
			.headers(self.headers.clone())
			.json(&payload)
			.send()
			.await?;
		response.error_for_status()
	}

	async fn licht(&self, message: String) {
		let result = self.send_message_once(&message, &self.chat_licht).await;

		match result {
			Ok(..) => (),
			Err(error) => {
				eprintln!("Couldn't post to licht {} because {}", message, error);
			}
		};
	}

	async fn ping(&self, message: String) {
		let result = self.send_message_once(&message, &self.chat_ping).await;

		match result {
			Ok(..) => (),
			Err(error) => {
				eprintln!("Couldn't ping {} because {}", message, error);
			}
		};
	}

	// logs and sends message, retries once, if it fails twice it logs the error
	async fn send_message(&self, message: String) {
		let result = self.send_message_once(&message, &self.chat).await;

		match result {
			Ok(..) => (),
			Err(old_error) => {
				let result = self
					.send_message_once(&format!("{} (sent again)", message), &self.chat)
					.await;
				match result {
					Ok(..) => (),
					Err(error) => {
						eprintln!(
							"Couldn't send {} first because {} and then because {}",
							message, old_error, error
						);
					}
				}
			}
		}
	}

	async fn set_info_online(&mut self, info: String) {
		self.info_online = info;
		self.send_status().await;
	}

	async fn set_info_door(&mut self, info: String) {
		self.info_door = info;
		self.send_status().await;
	}

	async fn set_info_environment(&mut self, info: String) {
		self.info_environment = info;
		self.send_status().await;
	}

	async fn set_status_in_chat(&self) {
		self.send_message_once(
			&format!(
				"Status: {} {} {}",
				self.info_online, self.info_door, self.info_environment
			),
			&self.chat_commands,
		)
		.await
		.unwrap();
	}

	async fn send_status(&self) {
		let status = format!(
			"{} {} {}",
			self.info_online, self.info_door, self.info_environment
		);
		let mut payload = HashMap::new();
		payload.insert("message", &status);
		let result = self
			.client
			.put(format!(
				"{}/ocs/v2.php/apps/user_status/api/v1/user_status/message/custom",
				&self.base_url
			))
			.basic_auth(&self.user, Some(&self.pass))
			.headers(self.headers.clone())
			.json(&payload)
			.send()
			.await;
		match result {
			Ok(..) => (),
			Err(error) => {
				eprintln!("Couldn't set status message {} because {}", status, error);
			}
		};
	}

	async fn get_last_messages(
		&self,
		last_known_message_id: &str,
	) -> Result<reqwest::Response, reqwest::Error> {
		let endpoint = format!(
			"{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
			self.base_url, self.chat_commands,
		);
		let query_params = [
			("lookIntoFuture", "1"),
			("limit", "100"),
			("lastKnownMessageId", last_known_message_id),
		];

		let response = self
			.client
			.get(&endpoint)
			.query(&query_params)
			.basic_auth(&self.user, Some(&self.pass))
			.headers(self.headers.clone())
			.send()
			.await?;

		response.error_for_status()
	}

	pub async fn get_background_task(
		mut self,
		nextcloud_receiver: Receiver<NextcloudEvent>,
		nextcloud_sender: Sender<NextcloudEvent>,
		command_sender: Sender<CommandToButtons>,
		audio_sender: Sender<AudioEvent>,
		startup_time: String,
	) -> Result<Never, ModuleError> {
		self.startup_time = startup_time;
		try_join!(
			self.clone().message_sender_loop(nextcloud_receiver),
			self.command_loop(nextcloud_sender, command_sender, audio_sender)
		)?;
		Err(ModuleError::new(String::from(
			"Exit get_background_task loop!",
		)))
	}

	async fn message_sender_loop(
		mut self,
		mut nextcloud_receiver: Receiver<NextcloudEvent>,
	) -> Result<Never, ModuleError> {
		self.ping(gettext!(
			"👋 Opensesame {} init {}",
			env!("CARGO_PKG_VERSION"),
			self.startup_time
		))
		.await;
		while let Some(event) = nextcloud_receiver.recv().await {
			match event {
				NextcloudEvent::Chat(chat, message) => match chat {
					NextcloudChat::Default => self.send_message(message).await,
					NextcloudChat::Ping => self.ping(message).await,
					NextcloudChat::Licht => self.licht(message).await,
				},
				NextcloudEvent::SendStatus => self.set_status_in_chat().await,
				NextcloudEvent::Status(status, message) => match status {
					NextcloudStatus::Online => self.set_info_online(message).await,
					NextcloudStatus::Env => self.set_info_environment(message).await,
					NextcloudStatus::Door => self.set_info_door(message).await,
				},
			}
		}
		Err(ModuleError::new(String::from(
			"Exit Nextcloud messagesender loop!",
		)))
	}

	async fn command_loop(
		self,
		nextcloud_sender: Sender<NextcloudEvent>,
		command_sender: Sender<CommandToButtons>,
		audio_sender: Sender<AudioEvent>,
	) -> Result<Never, ModuleError> {
		let a = self
			.send_message_once("Started listening to commands here", &self.chat_commands)
			.await
			.unwrap();
		let mut last_known_message_id =
			a.json::<serde_json::Value>().await.unwrap()["ocs"]["data"]["id"].to_string();
		loop {
			let response = self.get_last_messages(&last_known_message_id).await;
			match response {
				Ok(response) => {
					let status = response.status().as_u16();
					if status == 200 {
						let json = response.json::<serde_json::Value>().await.unwrap();

						let messages = json["ocs"]["data"].as_array();
						for message in messages
							.unwrap()
							.iter()
							.map(|m| m["message"].as_str().unwrap())
						{
							if message.starts_with('\\') {
								let command_and_args = message
									.strip_prefix('\\')
									.unwrap()
									.split_whitespace()
									.collect::<Vec<&str>>();
								let command = command_and_args[0];
								let args = &command_and_args[1..];
								match command {
									"status" => {
										nextcloud_sender.send(NextcloudEvent::SendStatus).await?
									}
									"setpin" => {
										// TODO: How do we access config?
									}
									"switchlights" => {
										if args.len() != 2 {
											nextcloud_sender
												.send(NextcloudEvent::Chat(
													NextcloudChat::Default,
													String::from(
														"Usage: switchlights <bool> <bool>",
													),
												))
												.await?;
										}

										let inner_light = args[0].eq_ignore_ascii_case("true");
										let outer_light = args[1].eq_ignore_ascii_case("true");

										command_sender
											.send(CommandToButtons::SwitchLights(
												inner_light,
												outer_light,
												String::from("Switch lights {} {}"),
											))
											.await?;
									}
									"opensesame" => {
										nextcloud_sender
											.send(NextcloudEvent::Chat(
												NextcloudChat::Default,
												String::from("Opening door"),
											))
											.await?;
										command_sender.send(CommandToButtons::OpenDoor).await?;
									}
									"ring_bell" => audio_sender.send(AudioEvent::Bell).await?,
									"fire_alarm" => {
										audio_sender.send(AudioEvent::FireAlarm).await?
									}
									_ => {
										nextcloud_sender
											.send(NextcloudEvent::Chat(
												NextcloudChat::Default,
												format!("Unknown command {}!", command),
											))
											.await?;
									}
								}
							}
						}
						if let Some(last_message) = messages.unwrap().last() {
							last_known_message_id = last_message["id"].to_string();
						}
					} else if status != 304 {
						// 304 - Not modified is expected if there are no new messages
						println!("Status code was not successful but {}", response.status());
					}
				}
				Err(err) => {
					eprintln!("Error: {:?}", err);
				}
			}
			interval(time::Duration::from_secs(1)).tick().await;
		}
	}
}
