use std::str::FromStr;
use std::fmt::{Display, Formatter};
use dioxus::prelude::*;

#[derive(Debug)]
pub enum SSHError {
    IO(String),
    Connection(String),
    DNS(String),
    Authentication(String),
}

impl FromStr for SSHError {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SSHError::Connection(s.to_string()))
    }
}

impl Display for SSHError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SSHError::IO(msg) => write!(f, "I/O Error: {}", msg),
            SSHError::Connection(msg) => write!(f, "Connection Error: {}", msg),
            SSHError::DNS(msg) => write!(f, "DNS Error: {}", msg),
            SSHError::Authentication(msg) => write!(f, "Authentication Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for SSHError {
    fn from(error: std::io::Error) -> Self {
        SSHError::IO(error.to_string())
    }
}

impl From<SSHError> for ServerFnError {
    fn from(err: SSHError) -> ServerFnError {
        ServerFnError::ServerError(err.to_string())
    }
}
