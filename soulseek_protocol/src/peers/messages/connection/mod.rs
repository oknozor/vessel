use std::io::Cursor;

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_string, write_string, ToBytes};
use crate::message_common::ConnectionType;
use crate::SlskError;

use self::InitMessageCode::*;

pub(crate) const CONNECTION_MSG_HEADER_LEN: u32 = 5;

#[derive(Debug)]
pub struct ConnectionMessageHeader {
    pub(crate) code: InitMessageCode,
    pub(crate) message_len: usize,
}

impl ConnectionMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u8();
        let code = InitMessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 1;

        Ok(Self { message_len, code })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum InitMessageCode {
    PierceFireWall = 0,
    PeerInit = 1,
    Unknown,
}

impl From<u8> for InitMessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => PierceFireWall,
            1 => PeerInit,
            _ => Unknown,
        }
    }
}

#[derive(Debug)]
pub enum PeerConnectionMessage {
    PierceFirewall(u32),
    PeerInit {
        username: String,
        connection_type: ConnectionType,
        token: u32,
    },
}

#[async_trait]
impl ToBytes for PeerConnectionMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            PeerConnectionMessage::PierceFirewall(token) => {
                buffer.write_u32_le(CONNECTION_MSG_HEADER_LEN).await?;
                buffer
                    .write_u8(InitMessageCode::PierceFireWall as u8)
                    .await?;
                buffer.write_u32_le(*token).await?;
            }
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                let username_len = username.bytes().len() as u32 + 4;
                let connection_type_len = connection_type.bytes().len() as u32 + 4;

                buffer
                    .write_u32_le(1 + username_len + connection_type_len + 4)
                    .await?;
                buffer.write_u8(InitMessageCode::PeerInit as u8).await?;
                write_string(&username, buffer).await?;
                write_string(connection_type.as_ref(), buffer).await?;

                buffer.write_u32_le(*token).await?;
            }
        }

        Ok(())
    }
}

impl PeerConnectionMessage {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<ConnectionMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < CONNECTION_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = ConnectionMessageHeader::read(src)?;

        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            Ok(header)
        }
    }

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &ConnectionMessageHeader,
    ) -> crate::Result<Self> {
        match header.code {
            InitMessageCode::PierceFireWall => {
                Ok(PeerConnectionMessage::PierceFirewall(src.get_u32_le()))
            }
            InitMessageCode::PeerInit => Ok(PeerConnectionMessage::PeerInit {
                username: read_string(src)?,
                connection_type: ConnectionType::from(read_string(src)?),
                token: src.get_u32_le(),
            }),
            InitMessageCode::Unknown => {
                error!("Unkown message kind, code");
                Err(SlskError::UnkownMessage)
            }
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerConnectionMessage::PierceFirewall(_) => "PierceFirewall",
            PeerConnectionMessage::PeerInit { .. } => "PeerInit",
        }
    }
}
