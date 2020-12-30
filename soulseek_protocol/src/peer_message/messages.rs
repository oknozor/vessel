use crate::frame::{read_string, write_string, ToBytes};
use crate::message_common::ConnectionType;
use crate::peer_message::messages::PeerMessage::PeerInit;
use crate::peer_message::{Header, MessageCode, HEADER_LEN};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug)]
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
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<Header, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = Header::read(src)?;
        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub(crate) fn parse(src: &mut Cursor<&[u8]>, header: &Header) -> std::io::Result<Self> {
        let message = match header.code {
            MessageCode::PierceFireWall => PeerMessage::PierceFirewall(src.get_u32_le()),
            MessageCode::PeerInit => PeerInit {
                username: read_string(src)?,
                connection_type: ConnectionType::from(read_string(src)?),
                token: src.get_u32_le(),
            },
            MessageCode::Unknown => panic!("Unkown message kind, code"),
        };

        Ok(message)
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerMessage::PierceFirewall(_) => "PierceFirewall",
            PeerMessage::PeerInit { .. } => "PeerInit",
        }
    }
}
