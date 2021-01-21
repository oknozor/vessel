use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

use crate::peers::connection::Connection;
use crate::peers::messages::{PeerConnectionMessage, PeerMessage};
use crate::peers::shutdown::Shutdown;
use crate::server::messages::peer::{PeerConnectionRequest, RequestConnectionToPeer};
use crate::message_common::ConnectionType;
use crate::peers::messages::PeerPacket;
use rand::random;
use tokio::time::Duration;
use crate::SlskError;
use crate::peers::messages::PeerConnectionMessage::PierceFirewall;
use tokio::sync::mpsc::Sender;
use crate::server::messages::request::ServerRequest;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Handler {
    pub username: Option<String>,
    pub(crate) connection: Connection,
    pub(crate) limit_connections: Arc<Semaphore>,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn run(&mut self, mut peer_connection_outgoing_tx: Sender<ServerRequest>) -> crate::Result<()> {
        debug!("sending peer init");
        let token = random();


        self.connection.write_request(PeerPacket::ConnectionMessage(PeerConnectionMessage::PeerInit {
            username: "vessel".to_string(),
            connection_type: ConnectionType::PeerToPeer,
            token,
        })).await?;

        // Fixme : handle fallback connection here

        self.connection.connection_type = Some(ConnectionType::PeerToPeer);

        self.connection.write_request(PeerPacket::Message(PeerMessage::UserInfoRequest)).await?;

        while !self.shutdown.is_shutdown() {
            // While reading a request frame, also listen for the shutdown
            // signal.
            let maybe_message = tokio::select! {
                res = self.connection.read_response() => res,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
            };

            if let Ok(message) = maybe_message {
                match message {
                    PeerPacket::ConnectionMessage(message) => info!("Got connection message : {:?}", &message),
                    PeerPacket::Message(message) => info!("Got message : {:?}", &message),
                    PeerPacket::None => {}
                }
            }
        }

        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.limit_connections.add_permits(1);
    }
}
