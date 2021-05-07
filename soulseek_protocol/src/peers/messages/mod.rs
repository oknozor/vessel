use p2p::request::PeerRequest;
use p2p::response::PeerResponse;

use crate::peers::messages::connection::PeerConnectionMessage;
use crate::peers::messages::distributed::DistributedMessage;

pub mod connection;
pub mod distributed;
pub mod p2p;

#[derive(Debug)]
pub enum PeerRequestPacket {
    Message(PeerRequest),
    ConnectionMessage(PeerConnectionMessage),
    DistributedMessage(DistributedMessage),
}

#[derive(Debug)]
pub enum PeerResponsePacket {
    Message(PeerResponse),
    ConnectionMessage(PeerConnectionMessage),
    DistributedMessage(DistributedMessage),
}
