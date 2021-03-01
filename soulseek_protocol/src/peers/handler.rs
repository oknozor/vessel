use std::sync::Arc;

use rand::random;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Semaphore};

use crate::message_common::ConnectionType;
use crate::peers::connection::Connection;
use crate::peers::messages::PeerRequestPacket;
use crate::peers::messages::{PeerConnectionMessage, PeerResponsePacket};
use crate::peers::request::PeerRequest;
use crate::peers::shutdown::Shutdown;
use crate::server::messages::request::ServerRequest;

#[derive(Debug)]
pub struct Handler {
    pub username: Option<String>,
    pub(crate) connection: Connection,
    pub(crate) limit_connections: Arc<Semaphore>,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn run(
        &mut self,
        peer_connection_outgoing_tx: Sender<ServerRequest>,
    ) -> crate::Result<()> {
        debug!("sending peer init");
        let token = random();

        self.connection
            .write_request(PeerRequestPacket::ConnectionMessage(
                PeerConnectionMessage::PeerInit {
                    username: "vessel".to_string(),
                    connection_type: ConnectionType::PeerToPeer,
                    token,
                },
            ))
            .await?;

        // Fixme : handle fallback connection here

        self.connection.connection_type = Some(ConnectionType::PeerToPeer);

        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::UserInfoRequest))
            .await?;

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
                    PeerResponsePacket::ConnectionMessage(message) => {
                        info!("Got connection message : {:?}", &message)
                    }
                    PeerResponsePacket::Message(message) => info!("Got message : {:?}", &message),
                    PeerResponsePacket::None => {}
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
