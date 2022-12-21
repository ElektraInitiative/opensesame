use std::collections::HashMap;

use crate::config::Config;

pub struct Validator {
	users: HashMap<Vec<u8>, String>,
	timeout: u64,
}

#[derive(PartialEq, Debug)]
pub enum Validation {
	None,
	Timeout,
	SequenceTooLong,
	Validated(String),
}

impl Validator {
	pub fn new(config: &mut Config) -> Self {
		Self {
			users: config.get_hash_map_vec_u8("validator"),
			timeout: 0,
		}
	}

	pub fn validate(&mut self, sequence: &mut Vec<u8>) -> Validation {
		if !sequence.is_empty() {
			self.timeout += 1;
		}
		if sequence.len() > 10 {
			sequence.clear();
			self.timeout = 0;
			return Validation::SequenceTooLong;
		}
		if self.timeout > 1000 {
			sequence.clear();
			self.timeout = 0;
			return Validation::Timeout;
		}
		if self.users.contains_key(sequence) {
			let ret = self.users.get(sequence).unwrap().to_string();
			sequence.clear();
			self.timeout = 0;
			return Validation::Validated(ret);
		}
		return Validation::None;
	}
}

#[cfg(test)]
mod tests {
	// Note this useful idiom: importing names from outer (for mod tests) scope.
	use super::*;
	use std::env;

	const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";

	#[test]
	fn test_validate() {
		let mut config: Config = Config::new(CONFIG_PARENT);

		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));

		config.cut("validator");
		config.add("validator/1234", "[14, 15, 13, 15, 11, 15, 7, 15]");

		let mut validator = Validator::new(&mut config);

		// validator.users.insert(vec![14, 15, 13, 15, 11, 15, 7, 15], "1234".to_string());
		assert_eq!(validator.timeout, 0);
		assert_eq!(validator.validate(&mut vec![]), Validation::None);
		assert_eq!(validator.timeout, 0);
		assert_eq!(
			validator.validate(&mut vec![14, 15, 13, 15, 13, 15, 11, 15, 7]),
			Validation::None
		);
		assert_eq!(validator.timeout, 1);
		assert_eq!(
			validator.validate(&mut vec![14, 15, 13, 15, 11, 15, 7, 15]),
			Validation::Validated("1234".to_string())
		);
	}
}
