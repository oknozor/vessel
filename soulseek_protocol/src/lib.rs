#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use std::{fmt, io::Cursor, net::SocketAddr, num::TryFromIntError, string::FromUtf8Error};

use bytes::Buf;
use tokio::time::error::Elapsed;

pub mod frame;
pub mod message_common;
pub mod peers;
/// Contains all the soulseek protocol server message, see [`ServerRequest`] and [`ServerResponse`]
/// for a detailed explanation of each one.
///
///  [`ServerRequest`]: crate::server::request::ServerRequest
///  [`ServerResponse`]: crate::server::request::ServerResponse
pub mod server;

pub type Result<T> = std::result::Result<T, SlskError>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum SlskError {
    /// Not enough data is available to parse a message
    Incomplete,
    /// Timeout waiting for an answer`
    TimeOut(Elapsed),
    BackoffReached(u64),
    /// Peer disconnected
    ConnectionResetByPeer,
    NoPermitAvailable,
    NoTicket,
    PeerConnectionLost,
    UnknownMessage,
    InvalidSocketAddress(SocketAddr),

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
        SlskError::Other(Box::new(src))
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
            SlskError::BackoffReached(value) => {
                write!(fmt, "Backoff period reached : {}", value)
            }
            SlskError::NoPermitAvailable => {
                write!(fmt, "No available connection")
            }
            SlskError::UnknownMessage => {
                write!(fmt, "Got unkown message")
            }
            SlskError::NoTicket => {
                write!(fmt, "Ticket for download not found")
            }
            SlskError::PeerConnectionLost => {
                write!(fmt, "Previous connection with peer was lost")
            }
            SlskError::InvalidSocketAddress(addr) => {
                write!(fmt, "Invalid socket address : {}", addr)
            }
        }
    }
}

pub trait ProtocolMessage<Output = Self> {
    type Header: ProtocolHeader;

    fn check(src: &mut Cursor<&[u8]>) -> crate::Result<Self::Header> {
        // Check if the buffer contains enough bytes to parse the message header
        if src.remaining() < Self::Header::LEN {
            return Err(SlskError::Incomplete);
        }
        // Check if the buffer contains the full message already
        let header = Self::Header::read::<Self::Header>(src)?;

        if src.remaining() < header.message_len() as usize {
            Err(SlskError::Incomplete)
        } else {
            Ok(header)
        }
    }

    fn parse(src: &mut Cursor<&[u8]>, header: &Self::Header) -> std::io::Result<Output>;
}

pub trait ProtocolHeader {
    const LEN: usize;
    type Code: MessageCode;

    fn message_len(&self) -> usize;

    fn read<T>(src: &mut Cursor<&[u8]>) -> std::io::Result<T>
    where
        T: ProtocolHeader,
    {
        let message_length = src.get_u32_le() as usize;
        let code = T::Code::read(src);

        // We can subtract message code from the length since we already know it
        let message_len = message_length - Self::Code::LEN;

        Ok(T::new(message_len, code))
    }

    fn new(message_len: usize, code: Self::Code) -> Self;
}

pub trait MessageCode<Output = Self> {
    const LEN: usize;
    fn read(src: &mut Cursor<&[u8]>) -> Output;
}
