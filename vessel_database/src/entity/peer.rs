use std::net::{Ipv4Addr, SocketAddr};
use crate::entity::Entity;
use soulseek_protocol::server::peer::{PeerConnectionRequest, PeerAddress, Peer};
use crate::entity::IpAddr;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerEntity {
    pub username: String,
    pub ip: Ipv4Addr,
    pub(crate) port: u32,
}

impl Entity for PeerEntity {
    fn get_key(&self) -> Vec<u8> {
        self.username.as_bytes().to_vec()
    }

    const COLLECTION: &'static str = "users";
}

impl PeerEntity {
    pub fn new(username: &str, ip: Ipv4Addr, port: u32) -> Self {
        PeerEntity {
            username: username.to_string(),
            ip,
            port,
        }
    }
}

impl From<PeerConnectionRequest> for PeerEntity {
    fn from(request: PeerConnectionRequest) -> Self {
        PeerEntity {
            username: request.username,
            ip: request.ip,
            port: request.port,
        }
    }
}

impl From<PeerAddress> for PeerEntity {
    fn from(peer: PeerAddress) -> Self {
        PeerEntity {
            username: peer.username,
            ip: peer.ip,
            port: peer.port,
        }
    }
}

impl From<Peer> for PeerEntity {
    fn from(peer: Peer) -> Self {
        PeerEntity {
            username: peer.username,
            ip: peer.ip,
            port: peer.port,
        }
    }
}

impl PeerEntity {
    pub fn get_address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::from(self.ip), self.port as u16)
    }
}