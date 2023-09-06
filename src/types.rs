use i2cdev::linux::LinuxI2CError;
use std::{error::Error, fmt};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Clone)]
pub struct OpensesameError {
	reason: String,
}

impl OpensesameError {
	pub fn new(reason: String) -> Self {
		Self { reason }
	}
}

impl Error for OpensesameError {}

impl fmt::Display for OpensesameError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "OpensesameErrorr: reason: {}", self.reason)
	}
}

// TODO: These are hacks for now:

impl<T> From<SendError<T>> for OpensesameError {
	fn from(error: SendError<T>) -> Self {
		OpensesameError {
			reason: format!("SendError: {}", error),
		}
	}
}

impl From<std::io::Error> for OpensesameError {
	fn from(error: std::io::Error) -> Self {
		OpensesameError {
			reason: format!("std::io::Error: {} {}", error.kind(), error),
		}
	}
}

impl From<LinuxI2CError> for OpensesameError {
	fn from(error: LinuxI2CError) -> Self {
		OpensesameError {
			reason: format!("LinuxI2CError: {}", error),
		}
	}
}
