use crate::frame::{read_string, write_string, ToBytes};
use crate::peer_message::messages::ConnectionType::{DistributedNetwork, FileTransfer, PeerToPeer};
use crate::peer_message::messages::PeerMessage::PeerInit;
use crate::peer_message::{Header, MessageCode, HEADER_LEN};
use bytes::Buf;
use std::io::Cursor;
use std::str::Bytes as StdBytes;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

pub enum PeerMessage {
    PierceFirewall(u32),
    PeerInit {
        username: String,
        connection_type: ConnectionType,
        token: u32,
    },
}

#[async_trait]
impl ToBytes for PeerMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            PeerMessage::PierceFirewall(token) => {
                buffer.write_u32_le(HEADER_LEN).await?;
                buffer
                    .write_u32_le(MessageCode::PierceFireWall as u32)
                    .await?;
                buffer.write_u32_le(*token).await?;
            }
            PeerMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                let username_len = username.bytes().len() as u32;
                let connection_type_len = connection_type.bytes().len() as u32;
                buffer
                    .write_u32_le(HEADER_LEN + username_len + connection_type_len + 4)
                    .await?;
                buffer.write_u32_le(MessageCode::PeerInit as u32).await?;
                write_string(&username, buffer).await?;
                write_string(connection_type.as_ref(), buffer).await?;
                buffer.write_u32_le(*token).await?;
            }
        }

        Ok(())
    }
}

impl PeerMessage {
    fn parse(src: &mut Cursor<&[u8]>, header: Header) -> std::io::Result<Self> {
        let message = match header.code {
            MessageCode::PierceFireWall => PeerMessage::PierceFirewall(src.get_u32_le()),
            MessageCode::PeerInit => PeerInit {
                username: read_string(src)?,
                connection_type: ConnectionType::from(read_string(src)?),
                token: src.get_u32_le(),
            },
            MessageCode::Unknown => panic!("Unkown message kind, code={}", header.code as u32),
        };

        Ok(message)
    }
}

pub enum ConnectionType {
    PeerToPeer,
    FileTransfer,
    DistributedNetwork,
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
    fn bytes(&self) -> StdBytes {
        match self {
            PeerToPeer => "P".bytes(),
            FileTransfer => "F".bytes(),
            DistributedNetwork => "D".bytes(),
        }
    }
}
