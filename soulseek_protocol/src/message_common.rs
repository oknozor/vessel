use crate::{
    frame::{read_string, ParseBytes},
    message_common::ConnectionType::{DistributedNetwork, FileTransfer, PeerToPeer},
    peers::PeerRequestPacket,
};
use std::{io::Cursor, str::Bytes};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum ConnectionType {
    PeerToPeer,
    FileTransfer,
    DistributedNetwork,
    HandShake,
}

impl ParseBytes for ConnectionType {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let raw_c_type = read_string(src)?;
        Ok(ConnectionType::from(raw_c_type))
    }
}

impl From<&PeerRequestPacket> for ConnectionType {
    fn from(request: &PeerRequestPacket) -> Self {
        match request {
            PeerRequestPacket::Message(_) => ConnectionType::PeerToPeer,
            PeerRequestPacket::DistributedMessage(_) => ConnectionType::DistributedNetwork,
            PeerRequestPacket::ConnectionMessage(_) => unreachable!(),
        }
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
            _ => unreachable!(),
        }
    }
}

impl ConnectionType {
    pub(crate) fn bytes(&self) -> Bytes {
        match self {
            PeerToPeer => "P".bytes(),
            FileTransfer => "F".bytes(),
            DistributedNetwork => "D".bytes(),
            _ => unreachable!(),
        }
    }
}
