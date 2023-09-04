use std::{error::Error, fmt};
use tokio::sync::mpsc::error::SendError;


#[derive(Debug, Clone)]
pub struct OpensesameError {
	reason: String
}

impl Error for OpensesameError {}

impl fmt::Display for OpensesameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OpensesameError: reason: {}", self.reason)
    }
}

impl<T> From<SendError<T>> for OpensesameError {
    fn from(error: SendError<T>) -> Self {
        OpensesameError { reason: format!("SendError: {}", error) }
    }
}