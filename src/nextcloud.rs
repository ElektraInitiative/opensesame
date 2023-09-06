use crate::{config::Config, types::OpensesameError, CommandToButtons};
use std::{collections::HashMap, future::Pending};

use futures::{join, never::Never, try_join};
use reqwest::{
	header::{HeaderMap, ACCEPT, CONTENT_TYPE},
	Client,
};
use tokio::{
	sync::mpsc::{Receiver, Sender},
	time::{self, interval},
};

pub enum NextcloudEvent {
	Ping(String),
	Licht(String),
	Chat(String),
	SendStatus,
	SetStatusOnline(String),
	SetStatusEnv(String),
	SetStatusDoor(String),
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
		self,
		nextcloud_receiver: Receiver<NextcloudEvent>,
		nextcloud_sender: Sender<NextcloudEvent>,
		command_sender: Sender<CommandToButtons>,
	) -> Result<Never, OpensesameError> {
		try_join!(
			self.clone().message_sender_loop(nextcloud_receiver),
			self.command_loop(nextcloud_sender, command_sender)
		)?;
		Err(OpensesameError::new(String::from(
			"Exit get_background_task loop!",
		)))
	}

	async fn message_sender_loop(
		mut self,
		mut nextcloud_receiver: Receiver<NextcloudEvent>,
	) -> Result<Never, OpensesameError> {
		self.send_message(String::from("Nextcloud stated...")).await;
		while let Some(event) = nextcloud_receiver.recv().await {
			match event {
				NextcloudEvent::Chat(message) => self.send_message(message).await,
				NextcloudEvent::Ping(message) => self.ping(message).await,
				NextcloudEvent::Licht(message) => self.licht(message).await,
				NextcloudEvent::SendStatus => self.set_status_in_chat().await,
				NextcloudEvent::SetStatusOnline(message) => self.set_info_online(message).await,
				NextcloudEvent::SetStatusEnv(message) => self.set_info_environment(message).await,
				NextcloudEvent::SetStatusDoor(message) => self.set_info_door(message).await,
			}
		}
		Err(OpensesameError::new(String::from(
			"Exit Nextcloud messagesender loop!",
		)))
	}

	async fn command_loop(
		self,
		nextcloud_sender: Sender<NextcloudEvent>,
		command_sender: Sender<CommandToButtons>,
	) -> Result<Never, OpensesameError> {
		let a = self
			.send_message_once(
				"Command chat started <explain command system>",
				&self.chat_commands,
			)
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
							if message.starts_with("\\") {
								let command = message.strip_prefix("\\").unwrap().trim();
								match command {
									"status" => {
										nextcloud_sender.send(NextcloudEvent::SendStatus).await?
									}
									_ => (),
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
