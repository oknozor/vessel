use std::io::Cursor;
use std::net::Ipv4Addr;

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_ipv4, read_string, write_string, ParseBytes, ToBytes, STR_LENGTH_PREFIX};
use crate::message_common::ConnectionType;
use crate::server::MessageCode;

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    pub username: String,
    pub ip: Ipv4Addr,
    pub port: u32,
}

impl ParseBytes for Peer {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let ip = read_ipv4(src);
        let port = src.get_u32_le();

        Ok(Peer { username, ip, port })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerAddress {
    pub username: String,
    pub ip: Ipv4Addr,
    pub port: u32,
    pub(crate) obfuscation: bool,
    pub(crate) obfuscated_port: u32,
}

impl ParseBytes for PeerAddress {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let ip = read_ipv4(src);
        let port = src.get_u32_le();
        let obfuscation = src.get_u8() == 1;
        let obfuscated_port = src.get_u32_le();

        Ok(PeerAddress {
            username,
            ip,
            port,
            obfuscation,
            obfuscated_port,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerConnectionRequest {
    pub username: String,
    pub connection_type: ConnectionType,
    pub ip: Ipv4Addr,
    pub port: u32,
    pub token: u32,
    pub privileged: bool,
}

impl PeerConnectionRequest {
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl ParseBytes for PeerConnectionRequest {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let connection_type = ConnectionType::parse(src)?;
        let ip = read_ipv4(src);
        let port = src.get_u32_le();
        let token = src.get_u32_le();
        let privileged = src.get_u8() == 1;

        Ok(PeerConnectionRequest {
            username,
            connection_type,
            ip,
            port,
            token,
            privileged,
        })
    }
}

pub type Parents = Vec<Peer>;

impl ParseBytes for Vec<Peer> {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let number_of_parent = src.get_u32_le();

        let mut parents = Vec::with_capacity(number_of_parent as usize);
        for _ in 0..number_of_parent {
            parents.push(Peer::parse(src)?);
        }

        Ok(parents)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConnectionToPeer {
    pub token: u32,
    pub username: String,
    pub connection_type: ConnectionType,
}

impl RequestConnectionToPeer {
    pub fn new(username: String, connection_type: ConnectionType) -> Self {
        let token: u32 = rand::random();

        Self {
            token,
            username,
            connection_type,
        }
    }
}

#[async_trait]
impl ToBytes for RequestConnectionToPeer {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        // Header
        let username_len = STR_LENGTH_PREFIX + self.username.bytes().len() as u32;
        let connection_type_len = STR_LENGTH_PREFIX + self.connection_type.bytes().len() as u32;

        let len = 4 + username_len + connection_type_len + 4;

        buffer.write_u32_le(len).await?;
        buffer
            .write_u32_le(MessageCode::ConnectToPeer as u32)
            .await?;

        buffer.write_u32_le(self.token).await?;
        write_string(&self.username, buffer).await?;
        write_string(self.connection_type.as_ref(), buffer).await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerConnectionTicket {
    pub token: u32,
    pub username: String,
}

impl ParseBytes for PeerConnectionTicket {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let ticket = src.get_u32_le();

        // do we have username sometime or is nicotine doc inaccurate ?
        let username = if src.has_remaining() {
            read_string(src)?
        } else {
            String::new()
        };

        Ok(Self {
            username,
            token: ticket,
        })
    }
}

#[async_trait]
impl ToBytes for PeerConnectionTicket {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let len = 4 + STR_LENGTH_PREFIX + self.username.bytes().len() as u32 + 4;

        buffer.write_u32_le(len).await?;
        buffer
            .write_u32_le(MessageCode::CantConnectToPeer as u32)
            .await?;
        write_string(&self.username, buffer).await?;
        buffer.write_u32_le(self.token).await?;

        Ok(())
    }
}
