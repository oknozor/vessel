use std::net::{IpAddr, SocketAddr};
use tokio::sync::mpsc;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{PeerConnectionRequest, PeerConnectionTicket};
use soulseek_protocol::server::request::ServerRequest;
use vessel_database::Database;
use crate::{SearchLimit, SenderPool, ShutdownHelper, TryFutureExt};
use eyre::Result;
use crate::peer_connection_manager::prepare_direct_connection_to_peer;
use crate::peers::handler::{pierce_firewall};

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
    pub async fn run(&mut self,) -> Result<()> {
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

            let addr = SocketAddr::new(
                IpAddr::V4(connection_request.ip),
                connection_request.port as u16,
            );
            let token = connection_request.token;
            let username = connection_request.username.clone();
            let conn_type = connection_request.connection_type;

            self.channels
                .clone()
                .insert_indirect_connection_expected(&username, conn_type, token);

            let connection_result = prepare_direct_connection_to_peer(
                self.channels.clone(),
                self.sse_tx.clone(),
                self.ready_tx.clone(),
                self.shutdown_helper.clone(),
                connection_request,
                addr,
                self.db.clone(),
                self.search_limit.clone(),
            )
                .and_then(|handler| pierce_firewall(handler, token))
                .await;

            if let Err(err) = connection_result {
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
}

