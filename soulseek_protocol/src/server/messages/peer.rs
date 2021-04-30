use crate::frame::{read_ipv4, read_string, write_string, ParseBytes, ToBytes, STR_LENGTH_PREFIX};
use crate::message_common::ConnectionType;
use crate::server::messages::MessageCode;
use bytes::Buf;
use std::io::Cursor;
use std::net::Ipv4Addr;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

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
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub username: String,
    pub ip: Ipv4Addr,
    pub(crate) port: u32,
}

impl Peer {
    pub fn get_address_with_port(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn get_address(&self) -> String {
        self.ip.to_string()
    }
}

impl ParseBytes for Vec<Peer> {
    type Output = Vec<Peer>;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let number_of_parent = src.get_u32_le();

        let mut parents = Vec::with_capacity(number_of_parent as usize);
        for _ in 0..number_of_parent {
            let username = read_string(src)?;
            let ip = read_ipv4(src);
            let port = src.get_u32_le();

            parents.push(Peer { username, ip, port });
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
    pub ticket: u32,
    pub username: String,
}

impl ParseBytes for PeerConnectionTicket {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let ticket = src.get_u32_le();

        // do we have username sometime or is nicotine doc unaccurate ?
        let username = if src.has_remaining() {
            read_string(src)?
        } else {
            String::new()
        };

        Ok(Self { username, ticket })
    }
}
