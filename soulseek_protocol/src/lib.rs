#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

use std::fmt;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

use config::ConfigError;
use tokio::time::Elapsed;

pub mod peers;

// pub mod listener;
// pub mod peer_connection;

pub mod frame;
pub mod message_common;
/// Contains all the soulseek protocol server message, see [`ServerRequest`] and [`ServerResponse`]
/// for a detailed explanation of each one.
///
///  [`ServerRequest`]: crate::server::messages::request::ServerRequest
///  [`ServerResponse`]: crate::server::messages::request::ServerResponse
pub mod server;

pub mod database;
pub mod settings;

pub type Result<T> = std::result::Result<T, SlskError>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum SlskError {
    /// Not enough data is available to parse a message
    Incomplete,
    /// Timeout waiting for an answer`
    TimeOut(Elapsed),
    /// Peer disconnected
    ConnectionResetByPeer,
    /// Configuration error
    ConfigError(ConfigError),

    /// Invalid message encoding
    Other(crate::Error),
}

impl From<String> for SlskError {
    fn from(src: String) -> SlskError {
        SlskError::Other(src.into())
    }
}

impl From<&str> for SlskError {
    fn from(src: &str) -> SlskError {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for SlskError {
    fn from(_src: FromUtf8Error) -> SlskError {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for SlskError {
    fn from(_src: TryFromIntError) -> SlskError {
        "protocol error; invalid frame format".into()
    }
}

impl From<std::io::Error> for SlskError {
    fn from(src: std::io::Error) -> SlskError {
        src.into()
    }
}

impl std::error::Error for SlskError {}

impl fmt::Display for SlskError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlskError::Incomplete => "stream ended early".fmt(fmt),
            SlskError::Other(err) => err.fmt(fmt),
            SlskError::TimeOut(elapsed) => {
                write!(fmt, "timed out after {} reading soulseek stream", elapsed)
            }
            SlskError::ConnectionResetByPeer => {
                write!(fmt, "Connection reset by peer")
            }
            SlskError::ConfigError(err) => {
                write!(fmt, "Configuration error : {}", err)
            }
        }
    }
}
