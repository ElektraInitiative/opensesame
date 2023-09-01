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
	pub fn new(config: &Config) -> Self {
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
		Validation::None
	}
}

#[cfg(test)]
mod tests {
	// Note this useful idiom: importing names from outer (for mod tests) scope.
	use super::*;
	use std::{env, vec};

	const CONFIG_PARENT: &'static str = "/sw/libelektra/opensesame/#0/current";

	fn setup_test_env(sequence: &str) -> Config {
		let mut config: Config = Config::new(CONFIG_PARENT);

		env::set_var("RUST_BACKTRACE", config.get::<String>("debug/backtrace"));

		config.cut("validator");
		config.add("validator/test", &sequence);
		config
	}

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

	#[test]
	fn test_validate_timeout() {
		//whitespace after comma; otherwise an error occurs
		let mut config: Config = setup_test_env("[7, 7, 13, 13]");
		let mut validator = Validator::new(&mut config);

		for x in 1..1001 {
			if x < 1001 {
				assert_eq!(
					validator.validate(&mut vec![13, 13, 7, 7]),
					Validation::None
				);
				assert_eq!(validator.timeout, x);
			} else {
				assert_eq!(
					validator.validate(&mut vec![13, 13, 7, 7]),
					Validation::Timeout
				);
			}
		}
	}

	#[test]
	fn test_validate_empty() {
		let mut config: Config = setup_test_env("[7, 3, 4, 2, 4]");
		let mut validator = Validator::new(&mut config);

		for _x in 1..2000 {
			assert_eq!(validator.validate(&mut vec![]), Validation::None);
			assert_eq!(validator.timeout, 0);
		}
	}

	#[test]
	fn test_validate_increment_seq() {
		let mut config: Config = setup_test_env("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
		let mut validator = Validator::new(&mut config);

		let mut seq: Vec<u8> = vec![];

		for x in 1..11 {
			seq.push(x);
			if x < 10 {
				assert_eq!(validator.validate(&mut seq), Validation::None);
				assert_eq!(validator.timeout, x as u64);
			} else {
				assert_eq!(
					validator.validate(&mut seq),
					Validation::Validated("test".to_string())
				);
				assert_eq!(validator.timeout, 0);
			}
		}
	}

	#[test]
	fn test_validate_seq_to_long() {
		let mut config: Config = setup_test_env("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]");
		let mut validator = Validator::new(&mut config);

		let mut seq: Vec<u8> = vec![];

		for x in 1..13 {
			seq.push(x);
			if seq.len() <= 10 {
				assert_eq!(validator.validate(&mut seq), Validation::None);
				assert_eq!(validator.timeout, seq.len() as u64);
			} else {
				assert_eq!(validator.validate(&mut seq), Validation::SequenceTooLong);
				assert_eq!(validator.timeout, 0);
				assert_eq!(seq.len(), 0);
			}
		}
	}
}
