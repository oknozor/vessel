use super::MAX_CONNECTIONS;
use crate::handler::connection::PeerConnectionListener;
use crate::handler::ShutdownHelper;
use crate::peers::connection::PeerConnection;
use crate::peers::handler::PeerHandler;
use crate::peers::shutdown::Shutdown;
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::SlskError;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use vessel_database::Database;

pub struct PeerConnectionManager {
    pub listener: PeerConnectionListener,
    pub sse_tx: Sender<PeerResponse>,
    pub db: Database,
    pub channels: SenderPool,
    pub shutdown_complete_rx: Receiver<()>,
    pub shutdown_helper: ShutdownHelper,
    pub search_limit: SearchLimit,
    pub ready_tx: Sender<u32>,
}

impl PeerConnectionManager {
    pub async fn run(&mut self, mut logged_in_rx: Receiver<()>) -> crate::Result<()> {
        while logged_in_rx.recv().await.is_none() {
            // Wait for soulseek login
        }

        let _ = tokio::join!(self.accept_peer_connection(
            self.sse_tx.clone(),
            self.ready_tx.clone(),
            self.channels.clone(),
            self.db.clone(),
            self.search_limit.clone()
        ),);

        Ok(())
    }

    async fn accept_peer_connection(
        &mut self,
        sse_tx: mpsc::Sender<PeerResponse>,
        ready_tx: mpsc::Sender<u32>,
        channels: SenderPool,
        db: Database,
        search_limit: SearchLimit,
    ) -> eyre::Result<()> {
        if self.shutdown_helper.limit_connections.available_permits() > 0 {
            info!("Accepting inbound connections");

            loop {
                match self.listener.accept().await {
                    Ok(socket) => {
                        self.shutdown_helper
                            .limit_connections
                            .acquire()
                            .await
                            .unwrap()
                            .forget();

                        debug!(
                            "Incoming direct connection from {:?} accepted",
                            socket.peer_addr()
                        );

                        debug!(
                            "Available connections : {}/{}",
                            self.shutdown_helper.limit_connections.available_permits(),
                            MAX_CONNECTIONS
                        );

                        let address = socket.peer_addr()?;

                        let mut handler = PeerHandler {
                            peer_username: None,
                            connection: PeerConnection::new(socket),
                            sse_tx: sse_tx.clone(),
                            ready_tx: ready_tx.clone(),
                            shutdown: Shutdown::new(self.shutdown_helper.notify_shutdown.subscribe()),
                            limit_connections: self.shutdown_helper.limit_connections.clone(),
                            limit_search: search_limit.clone(),
                            _shutdown_complete: self.shutdown_helper.shutdown_complete_tx.clone(),
                            connection_states: channels.clone(),
                            db: db.clone(),
                            address,
                        };

                        tokio::spawn(async move {
                            match handler.wait_for_connection_handshake().await {
                                Err(e) => {
                                    error!(cause = ?e, "Error accepting inbound connection {}", handler.address)
                                }
                                Ok(()) => debug!("Handler closed successfully"),
                            };
                        });
                    }
                    Err(err) => {
                        error!("Failed to accept incoming connection : {}", err);
                    }
                };
            }
        } else {
            Err(eyre!(SlskError::NoPermitAvailable))
        }
    }
}
