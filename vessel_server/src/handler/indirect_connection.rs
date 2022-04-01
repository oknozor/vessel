use crate::peers::connection::PeerConnection;
use crate::peers::handler::{pierce_firewall, PeerHandler};
use crate::peers::shutdown::Shutdown;
use crate::{SearchLimit, SenderPool, ShutdownHelper, TryFutureExt};
use eyre::Result;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{PeerConnectionRequest, PeerConnectionTicket};
use soulseek_protocol::server::request::ServerRequest;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use vessel_database::Database;

pub struct IndirectConnectionRequestHandler {
    pub server_request_tx: mpsc::Sender<ServerRequest>,
    pub indirect_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    pub sse_tx: mpsc::Sender<PeerResponse>,
    pub ready_tx: mpsc::Sender<u32>,
    pub channels: SenderPool,
    pub shutdown_helper: ShutdownHelper,
    pub db: Database,
    pub search_limit: SearchLimit,
}

impl IndirectConnectionRequestHandler {
    // Receive connection request from server, and attempt direct connection to peer
    pub async fn run(&mut self) -> Result<()> {
        while let Some(connection_request) = self.indirect_connection_rx.recv().await {
            // Filter out our own connection requests
            if connection_request.username == "vessel" {
                continue;
            };

            self.shutdown_helper
                .limit_connections
                .acquire()
                .await
                .unwrap()
                .forget();

            info!(
                "Available permit : {:?}",
                self.shutdown_helper.limit_connections.available_permits()
            );

            info!(
                "Trying requested direct connection : {:?}",
                connection_request,
            );

            let token = connection_request.token;
            let username = connection_request.username.clone();
            let conn_type = connection_request.connection_type;

            self.channels
                .clone()
                .insert_indirect_connection_expected(&username, conn_type, token);

            if let Err(err) = self
                .prepare_direct_connection_to_peer(connection_request)
                .and_then(|handler| pierce_firewall(handler, token))
                .await
            {
                info!(
                    "Unable to establish requested connection to peer, {}, token = {}, cause = {}",
                    username, token, err
                );

                self.server_request_tx
                    .send(ServerRequest::CantConnectToPeer(PeerConnectionTicket {
                        token,
                        username,
                    }))
                    .await?;
            } else {
                info!("Direct connection established via server indirection")
            }
        }

        Ok(())
    }

    async fn prepare_direct_connection_to_peer(
        &self,
        connection_request: PeerConnectionRequest,
    ) -> Result<PeerHandler> {
        let username = &connection_request.username;
        let address = connection_request
            .get_address()
            .parse()
            .expect("Failed to parse socket address");


        match timeout(Duration::from_secs(1), TcpStream::connect(&address)).await {
            Ok(Ok(socket)) => {
                let shutdown = Shutdown::new(self.shutdown_helper.notify_shutdown.subscribe());
                let limit_connections = self.shutdown_helper.limit_connections.clone();
                let connection = PeerConnection::new_with_token(socket, connection_request.token);
                let peer_username = Some(username.clone());
                let sse_tx = self.sse_tx.clone();
                let ready_tx = self.ready_tx.clone();
                let limit_search = self.search_limit.clone();
                let shutdown_complete = self.shutdown_helper.shutdown_complete_tx.clone();
                let connection_states = self.channels.clone();
                let db = self.db.clone();

                Ok(PeerHandler {
                    peer_username,
                    connection,
                    sse_tx,
                    ready_tx,
                    shutdown,
                    limit_connections,
                    limit_search,
                    _shutdown_complete: shutdown_complete,
                    connection_states,
                    db,
                    address,
                })
            },
            Ok(Err(e)) => Err(eyre!(
                "Error connecting to peer {:?} via server requested connection {:?}, cause = {}",
                username,
                connection_request,
                e
            )),
            Err(e) => Err(e.into()),
        }
    }
}
