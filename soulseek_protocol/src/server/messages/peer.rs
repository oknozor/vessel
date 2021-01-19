use crate::frame::{read_ipv4, read_string, ParseBytes};
use crate::message_common::ConnectionType;
use bytes::Buf;
use std::io::Cursor;
use std::net::Ipv4Addr;

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerConnection {
    username: String,
    connection_type: ConnectionType,
    ip: Ipv4Addr,
    port: u32,
    token: u32,
    privileged: bool,
}

impl ParseBytes for PeerConnection {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let username = read_string(src)?;
        let connection_type = ConnectionType::parse(src)?;
        let ip = read_ipv4(src)?;
        let port = src.get_u32_le();
        let token = src.get_u32_le();
        let privileged = src.get_u8() == 1;

        Ok(PeerConnection {
            username,
            connection_type,
            ip,
            port,
            token,
            privileged,
        })
    }
}

pub type Parents = Vec<Parent>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Parent {
    username: String,
    ip: Ipv4Addr,
    port: u32,
}

impl ParseBytes for Vec<Parent> {
    type Output = Vec<Parent>;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let number_of_parent = src.get_u32_le();

        let mut parents = Vec::with_capacity(number_of_parent as usize);
        for _ in 0..number_of_parent {
            let username = read_string(src)?;
            let ip = read_ipv4(src)?;
            let port = src.get_u32_le();

            parents.push(Parent { username, ip, port });
        }

        Ok(parents)
    }
}
