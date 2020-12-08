#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use std::fmt;
use std::string::FromUtf8Error;

use bytes::Buf;
use std::io::{Cursor, Read};
use std::net::Ipv4Addr;
use std::num::TryFromIntError;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::Elapsed;

pub mod connection;
mod peer_message;
pub mod server_message;

mod distributed_message;
mod frame;

pub type Result<T> = std::result::Result<T, SlskError>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum SlskError {
    /// Not enough data is available to parse a message
    Incomplete,
    /// Timeout waiting for an answer
    TimeOut(Elapsed),
    /// Invalid message encoding
    Other(crate::Error),
}

pub async fn write_string(src: &str, buffer: &mut BufWriter<TcpStream>) -> tokio::io::Result<()> {
    let bytes = src.as_bytes();
    buffer.write_u32_le(bytes.len() as u32).await?;
    buffer.write(bytes).await?;
    Ok(())
}

pub fn read_string(src: &mut Cursor<&[u8]>) -> std::io::Result<String> {
    let string_len = src.get_u32_le();
    if string_len > 0 {
        let mut string = vec![0u8; string_len as usize];
        src.read_exact(&mut string)?;
        Ok(String::from_utf8_lossy(&string).to_string())
    } else {
        Ok("".to_string())
    }
}

pub fn read_ipv4(src: &mut Cursor<&[u8]>) -> std::io::Result<Ipv4Addr> {
    let ip = src.get_u32_le();
    Ok(Ipv4Addr::from(ip))
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
        }
    }
}

pub const STR_LENGTH_PREFIX: u32 = 4;
