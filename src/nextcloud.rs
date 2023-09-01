use std::{collections::HashMap, sync::Arc};
use crate::config::Config;

use reqwest::{
	header::{HeaderMap, ACCEPT, CONTENT_TYPE},
	Client,
};

pub struct Nextcloud {
	base_url: String,
	chat: String,
	chat_ping: String,
	chat_licht: String,
	user: String,
	pass: String,
	info_door: String,
	info_environment: String,
	info_online: String,
	client: Client,
	headers: HeaderMap,
}

impl Nextcloud {
	pub fn new(config: &Config) -> Self {
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

	pub async fn licht(&self, message: String) {
		let result = self.send_message_once(&message, &self.chat_licht).await;

		match result {
			Ok(..) => (),
			Err(error) => {
				eprintln!("Couldn't post to licht {} because {}", message, error);
			}
		};
	}

	pub async fn ping(&self, message: String) {
		let result = self.send_message_once(&message, &self.chat_ping).await;

		match result {
			Ok(..) => (),
			Err(error) => {
				eprintln!("Couldn't ping {} because {}", message, error);
			}
		};
	}

	// logs and sends message, retries once, if it fails twice it logs the error
	pub async fn send_message(&self, message: String) {
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

	pub async fn set_info_online(&mut self, info: String) {
		self.info_online = info;
		self.send_status().await;
	}

	pub async fn set_info_door(&mut self, info: String) {
		self.info_door = info;
		self.send_status().await;
	}

	pub async fn set_info_environment(&mut self, info: String) {
		self.info_environment = info;
		self.send_status().await;
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
}
