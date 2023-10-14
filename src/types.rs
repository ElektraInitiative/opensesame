use i2cdev::linux::LinuxI2CError;
use nix::errno::Errno;
use std::{error::Error, fmt};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Clone)]
pub struct ModuleError {
	reason: String,
}

impl ModuleError {
	pub fn new(reason: String) -> Self {
		Self { reason }
	}
}

impl Error for ModuleError {}

impl fmt::Display for ModuleError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ModuleErrorr: reason: {}", self.reason)
	}
}

// TODO: These are hacks for now:

impl<T> From<SendError<T>> for ModuleError {
	fn from(error: SendError<T>) -> Self {
		ModuleError {
			reason: format!("SendError: {}", error),
		}
	}
}

impl From<std::io::Error> for ModuleError {
	fn from(error: std::io::Error) -> Self {
		ModuleError {
			reason: format!("std::io::Error: {} {}", error.kind(), error),
		}
	}
}

impl From<LinuxI2CError> for ModuleError {
	fn from(error: LinuxI2CError) -> Self {
		ModuleError {
			reason: format!("LinuxI2CError: {}", error),
		}
	}
}

impl From<Errno> for ModuleError {
	fn from(error: Errno) -> Self {
		ModuleError {
			reason: format!("Errno: {}", error),
		}
	}
}
