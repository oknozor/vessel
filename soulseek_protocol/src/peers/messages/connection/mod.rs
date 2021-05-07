use std::io::{Cursor, ErrorKind};

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_string, write_string, ToBytes};
use crate::message_common::ConnectionType;
use crate::{MessageCode, ProtocolHeader, ProtocolMessage};

use self::ConnectionMessageCode::*;

#[derive(Debug)]
pub struct ConnectionMessageHeader {
    pub(crate) code: ConnectionMessageCode,
    pub(crate) message_len: usize,
}

impl ProtocolHeader for ConnectionMessageHeader {
    const LEN: usize = 5;
    type Code = ConnectionMessageCode;

    fn message_len(&self) -> usize {
        self.message_len
    }

    fn new(message_len: usize, code: Self::Code) -> Self {
        Self { message_len, code }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum ConnectionMessageCode {
    PierceFireWall = 0,
    PeerInit = 1,
    Unknown,
}

impl MessageCode for ConnectionMessageCode {
    const LEN: usize = 1;

    fn read(src: &mut Cursor<&[u8]>) -> Self {
        let code = src.get_u8();
        Self::from(code)
    }
}

impl From<u8> for ConnectionMessageCode {
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
                buffer.write_u32_le(8).await?;
                buffer
                    .write_u8(ConnectionMessageCode::PierceFireWall as u8)
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
                buffer
                    .write_u8(ConnectionMessageCode::PeerInit as u8)
                    .await?;
                write_string(&username, buffer).await?;
                write_string(connection_type.as_ref(), buffer).await?;

                buffer.write_u32_le(*token).await?;
            }
        }

        Ok(())
    }
}

impl ProtocolMessage for PeerConnectionMessage {
    type Header = ConnectionMessageHeader;

    fn parse(src: &mut Cursor<&[u8]>, header: &ConnectionMessageHeader) -> std::io::Result<Self> {
        match header.code {
            ConnectionMessageCode::PierceFireWall => {
                Ok(PeerConnectionMessage::PierceFirewall(src.get_u32_le()))
            }
            ConnectionMessageCode::PeerInit => {
                let username = read_string(src)?;
                let connection_type = ConnectionType::from(read_string(src)?);
                let token = src.get_u32_le();

                Ok(PeerConnectionMessage::PeerInit {
                    username,
                    connection_type,
                    token,
                })
            }
            ConnectionMessageCode::Unknown => {
                error!("Unkown message kind, code");
                Err(std::io::Error::from(ErrorKind::Other))
            }
        }
    }
}
