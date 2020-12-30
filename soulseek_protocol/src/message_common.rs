use crate::frame::{read_string, ParseBytes};
use crate::message_common::ConnectionType::{DistributedNetwork, FileTransfer, PeerToPeer};
use std::io::Cursor;
use std::str::Bytes;

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionType {
    PeerToPeer,
    FileTransfer,
    DistributedNetwork,
}

impl ParseBytes for ConnectionType {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let raw_c_type = read_string(src)?;
        Ok(ConnectionType::from(raw_c_type))
    }
}

impl From<String> for ConnectionType {
    fn from(code: String) -> Self {
        match code {
            p if p == "P" => PeerToPeer,
            f if f == "F" => FileTransfer,
            d if d == "D" => DistributedNetwork,
            other => panic!("Unexpected connection type received : {}", other),
        }
    }
}

impl AsRef<str> for ConnectionType {
    fn as_ref(&self) -> &str {
        match self {
            PeerToPeer => "P",
            FileTransfer => "F",
            DistributedNetwork => "D",
        }
    }
}

impl ConnectionType {
    pub(crate) fn bytes(&self) -> Bytes {
        match self {
            PeerToPeer => "P".bytes(),
            FileTransfer => "F".bytes(),
            DistributedNetwork => "D".bytes(),
        }
    }
}
