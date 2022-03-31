use crate::peer_message_dispatcher::peer_message_dispatcher::Dispatcher;
use crate::peers::connection::PeerConnection;
use crate::peers::handler::{pierce_firewall, PeerHandler};
use crate::peers::peer_listener::{
    connect_to_peer_with_fallback, PeerListener, PeerListenerReceivers, PeerListenerSenders,
    ShutdownHelper,
};
use crate::peers::shutdown::Shutdown;
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use eyre::Result;
use futures::TryFutureExt;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{Peer, PeerConnectionRequest, PeerConnectionTicket};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::SlskError;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;

/// TODO : Make this value configurable
pub const MAX_CONNECTIONS: usize = 10_000;
pub const MAX_PARENT: usize = 1;

pub struct GlobalConnectionHandler {
    pub peer_listener: PeerListener,
    pub senders: PeerListenerSenders,
    pub db: Database,
    pub channels: SenderPool,
    pub shutdown_complete_rx: mpsc::Receiver<()>,
    pub shutdown_helper: ShutdownHelper,
    pub search_limit: SearchLimit,
}

impl GlobalConnectionHandler {
    pub async fn run(&mut self, mut receivers: PeerListenerReceivers) -> crate::Result<()> {
        let server_request_tx = self.senders.server_request_tx.clone();
        let sse_tx = self.senders.sse_tx.clone();
        let channels = self.channels.clone();
        let db = self.db.clone();
        let (ready_tx, ready_rx) = mpsc::channel(32);
        let shutdown_helper = self.shutdown_helper.clone();

        let mut dispatcher = Dispatcher {
            ready_rx,
            queue_rx: receivers.peer_request_rx,
            peer_address_rx: receivers.peer_address_rx,
            channels: channels.clone(),
            db: db.clone(),
            shutdown_helper: shutdown_helper.clone(),
            sse_tx: sse_tx.clone(),
            ready_tx: ready_tx.clone(),
            server_request_tx: server_request_tx.clone(),
            message_queue: Default::default(),
            search_limit: self.search_limit.clone(),
        };

        let search_limit = self.search_limit.clone();

        let _ = tokio::join!(
            dispatcher.run(),
            listen_indirect_peer_connection_request(
                server_request_tx.clone(),
                receivers.peer_listener_rx,
                sse_tx.clone(),
                ready_tx.clone(),
                channels.clone(),
                shutdown_helper.clone(),
                db.clone(),
                search_limit.clone()
            ),
            self.accept_peer_connection(
                sse_tx.clone(),
                ready_tx.clone(),
                channels.clone(),
                db.clone(),
                search_limit.clone()
            ),
            connect_to_parents(
                server_request_tx,
                sse_tx,
                ready_tx,
                channels,
                shutdown_helper.clone(),
                &mut receivers.possible_parent_rx,
                db,
                search_limit
            )
        );

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
                match self.peer_listener.accept().await {
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
                            shutdown: Shutdown::new(
                                self.shutdown_helper.notify_shutdown.subscribe(),
                            ),
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

// Receive connection request from server, and attempt direct connection to peer
async fn listen_indirect_peer_connection_request(
    server_request_tx: mpsc::Sender<ServerRequest>,
    mut indirect_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    db: Database,
    search_limit: SearchLimit,
) -> Result<()> {
    while let Some(connection_request) = indirect_connection_rx.recv().await {
        let sse_tx = sse_tx.clone();

        // Filter out our own connection requests
        if connection_request.username == "vessel" {
            continue;
        };

        shutdown_helper
            .limit_connections
            .acquire()
            .await
            .unwrap()
            .forget();

        info!(
            "Available permit : {:?}",
            shutdown_helper.limit_connections.available_permits()
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

        channels
            .clone()
            .insert_indirect_connection_expected(&username, conn_type, token);

        let connection_result = prepare_direct_connection_to_peer(
            channels.clone(),
            sse_tx.clone(),
            ready_tx.clone(),
            shutdown_helper.clone(),
            connection_request,
            addr,
            db.clone(),
            search_limit.clone(),
        )
        .and_then(|handler| pierce_firewall(handler, token))
        .await;

        if let Err(err) = connection_result {
            info!(
                "Unable to establish requested connection to peer, {}, token = {}, cause = {}",
                username, token, err
            );

            server_request_tx
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

async fn connect_to_parents(
    request_peer_connection_tx: mpsc::Sender<ServerRequest>,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    possible_parent_rx: &mut Receiver<Vec<Peer>>,
    database: Database,
    search_limit: SearchLimit,
) -> Result<()> {
    loop {
        while let Some(parents) = possible_parent_rx.recv().await {
            for parent in parents {
                let parent = PeerEntity::from(parent);
                let parent_count = channels.get_parent_count();
                debug!("Connected to {}/{} parents", parent_count, MAX_PARENT);

                if parent_count >= MAX_PARENT {
                    info!("Max parent count reached");
                    let server_request_sender = request_peer_connection_tx.clone();
                    server_request_sender
                        .send(ServerRequest::NoParents(false))
                        .await?;

                    return Ok(());
                };

                connect_to_peer_with_fallback(
                    request_peer_connection_tx.clone(),
                    sse_tx.clone(),
                    ready_tx.clone(),
                    channels.clone(),
                    shutdown_helper.clone(),
                    database.clone(),
                    &parent,
                    ConnectionType::DistributedNetwork,
                    search_limit.clone(),
                )
                .await?;
            }
        }
    }
}

async fn prepare_direct_connection_to_peer(
    channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    shutdown_helper: ShutdownHelper,
    connection_request: PeerConnectionRequest,
    address: SocketAddr,
    db: Database,
    search_limit: SearchLimit,
) -> Result<PeerHandler> {
    let username = connection_request.username.clone();

    match timeout(Duration::from_secs(1), TcpStream::connect(address)).await {
        Ok(Ok(socket)) => Ok(PeerHandler {
            peer_username: Some(username),
            connection: PeerConnection::new_with_token(socket, connection_request.token),
            sse_tx: sse_tx.clone(),
            ready_tx,
            shutdown: Shutdown::new(shutdown_helper.notify_shutdown.subscribe()),
            limit_connections: shutdown_helper.limit_connections.clone(),
            limit_search: search_limit.clone(),
            _shutdown_complete: shutdown_helper.shutdown_complete_tx.clone(),
            connection_states: channels,
            db,
            address,
        }),
        Ok(Err(e)) => Err(eyre!(
            "Error connecting to peer {:?} via server requested connection {:?}, cause = {}",
            username,
            connection_request,
            e
        )),
        Err(e) => Err(e.into()),
    }
}
