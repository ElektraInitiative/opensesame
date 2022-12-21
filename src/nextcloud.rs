use reqwest::header;

use crate::config::Config;

pub struct Nextcloud {
	url: String,
	chat: String,
	chat_ping: String,
	chat_licht: String,
	user: String,
	pass: Option<String>,
	info_door: String,
	info_environment: String,
	info_online: String,
}

impl Nextcloud {
	pub fn new(config: &mut Config) -> Self {
		Self {
			url: config.get::<String>("nextcloud/url"),
			chat: config.get::<String>("nextcloud/chat"),
			chat_ping: config.get::<String>("nextcloud/chat/ping"),
			chat_licht: config.get::<String>("nextcloud/chat/licht"),
			user: config.get::<String>("nextcloud/user"),
			pass: Some(config.get::<String>("nextcloud/pass")),
			info_door: "".to_string(),
			info_environment: "".to_string(),
			info_online: "".to_string(),
		}
	}

	// sends once, Err if it does not work on network or nextcloud level
	fn send_message_once(
		&self,
		message: String,
		chat: String,
	) -> Result<reqwest::Response, reqwest::Error> {
		let mut headers = header::HeaderMap::new();
		headers.insert("Content-Type", "application/json".parse().unwrap());
		headers.insert("Accept", "application/json".parse().unwrap());
		headers.insert("OCS-APIRequest", "true".parse().unwrap());

		let result = reqwest::Client::new()
			.post(&format!(
				"{}/ocs/v2.php/apps/spreed/api/v1/chat/{}",
				&self.url, chat
			))
			.basic_auth(&self.user, self.pass.as_ref())
			.headers(headers)
			.body(format!(
				"{{\"token\": \"{}\", \"message\": \"{}\"}}",
				chat, message
			))
			.send();
		match result {
			Ok(response) => match response.error_for_status() {
				Ok(response) => Ok(response),
				Err(error) => Err(error),
			},
			Err(error) => Err(error),
		}
	}

	pub fn licht(&self, message: String) {
		let result = self.send_message_once(message.to_string(), self.chat_licht.to_string());

		match result {
			Ok(_response) => (),
			Err(error) => {
				eprintln!("Couldn't post to licht {} because {}", message, error);
			}
		}
	}

	pub fn ping(&self, message: String) {
		let result = self.send_message_once(message.to_string(), self.chat_ping.to_string());

		match result {
			Ok(_response) => (),
			Err(error) => {
				eprintln!("Couldn't ping {} because {}", message, error);
			}
		}
	}

	// logs and sends message, retries once, if it fails twice it logs the error
	pub fn send_message(&self, message: String) {
		let result = self.send_message_once(message.to_string(), self.chat.to_string());

		match result {
			Ok(_response) => (),
			Err(old_error) => {
				let result = self.send_message_once(
					message.to_string() + " (sent again)",
					self.chat.to_string(),
				);
				match result {
					Ok(_response) => (),
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

	pub fn set_info_online(&mut self, info: String) {
		self.info_online = info;
		self.update_status();
	}

	pub fn set_info_door(&mut self, info: String) {
		self.info_door = info;
		self.update_status();
	}

	pub fn set_info_environment(&mut self, info: String) {
		self.info_environment = info;
		self.update_status();
	}

	fn update_status(&mut self) {
		self.set_status(format!(
			"{} {} {}",
			self.info_online, self.info_door, self.info_environment
		));
	}

	fn set_status(&self, message: String) {
		let mut headers = header::HeaderMap::new();
		headers.insert("Content-Type", "application/json".parse().unwrap());
		headers.insert("Accept", "application/json".parse().unwrap());
		headers.insert("OCS-APIRequest", "true".parse().unwrap());

		let result = reqwest::Client::new()
			.put(&format!(
				"{}/ocs/v2.php/apps/user_status/api/v1/user_status/message/custom",
				&self.url
			))
			.basic_auth(&self.user, self.pass.as_ref())
			.headers(headers)
			.body(format!("{{\"message\": \"{}\"}}", message))
			.send();
		match result {
			Ok(_response) => (),
			Err(error) => {
				eprintln!("Couldn't set status message {} because {}", message, error);
			}
		};
	}
}
