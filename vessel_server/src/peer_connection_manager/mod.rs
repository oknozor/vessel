use crate::peers::connection::PeerConnection;
use crate::peers::handler::{connect_direct, pierce_firewall, PeerHandler};
use crate::peers::shutdown::Shutdown;
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use connection::PeerConnectionListener;
use eyre::Result;
use futures::TryFutureExt;
use rand::random;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{
    Peer, PeerConnectionRequest, PeerConnectionTicket, RequestConnectionToPeer,
};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::SlskError;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::timeout;
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;

mod connection;

/// TODO : Make this value configurable
pub const MAX_CONNECTIONS: usize = 10_000;
pub const MAX_PARENT: usize = 1;

pub struct PeerConnectionManager {
    pub peer_listener: PeerConnectionListener,
    pub sse_tx: Sender<PeerResponse>,
    pub server_request_tx: Sender<ServerRequest>,
    pub db: Database,
    pub channels: SenderPool,
    pub shutdown_complete_rx: Receiver<()>,
    pub shutdown_helper: ShutdownHelper,
    pub search_limit: SearchLimit,
}

impl PeerConnectionManager {
    pub async fn run(
        &mut self,
        peer_listener_rx: Receiver<PeerConnectionRequest>,
        mut possible_parent_rx: Receiver<Vec<Peer>>,
        ready_tx: Sender<u32>,
        search_limit: SearchLimit
    ) -> crate::Result<()> {
        let server_request_tx = self.server_request_tx.clone();
        let sse_tx = self.sse_tx.clone();
        let channels = self.channels.clone();
        let db = self.db.clone();
        let shutdown_helper = self.shutdown_helper.clone();

        let _ = tokio::join!(
            listen_indirect_peer_connection_request(
                server_request_tx.clone(),
                peer_listener_rx,
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
                &mut possible_parent_rx,
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

pub async fn run(
    listener: TcpListener,
    shutdown: impl Future,
    sse_tx: Sender<PeerResponse>,
    server_request_tx: Sender<ServerRequest>,
    peer_listener_rx: Receiver<PeerConnectionRequest>,
    possible_parent_rx: Receiver<Vec<Peer>>,
    db: Database,
    channels: SenderPool,
    search_limit: SearchLimit,
    ready_tx: Sender<u32>,
    mut logged_in_rx: Receiver<()>,
    shutdown_helper: ShutdownHelper,
    shutdown_complete_rx: Receiver<()>
) -> crate::Result<()> {
    debug!("Waiting for user to be logged in");
    while logged_in_rx.recv().await.is_none() {
        // Wait for soulseek login
    }

    info!("User logged in, starting peer listener");

    // Initialize the listener state
    let peer_listener = PeerConnectionListener { listener };

    let mut server = PeerConnectionManager {
        peer_listener,
        sse_tx,
        db,
        channels,
        shutdown_complete_rx,
        shutdown_helper,
        search_limit: search_limit.clone(),
        server_request_tx,
    };

    tokio::select! {
        res = server.run(
            peer_listener_rx,
            possible_parent_rx,
            ready_tx.clone(),
            search_limit.clone(),
        ) => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        },
        _err = shutdown => {
            info!("shutting down");
            std::process::exit(0);
        }
    }

    let PeerConnectionManager {
        mut shutdown_complete_rx,
        shutdown_helper,
        ..
    } = server;

    debug!("Closing connection handler");
    drop(shutdown_helper.notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_helper.shutdown_complete_tx);

    // Wait for all active connections to finish processing. As the `Sender`
    // handle held by the listener has been dropped above, the only remaining
    // `Sender` instances are held by connection handler tasks. When those drop,
    // the `mpsc` channel will close and `recv()` will return `None`.
    let _ = shutdown_complete_rx.recv().await;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ShutdownHelper {
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown_complete_tx: mpsc::Sender<()>,
    pub limit_connections: Arc<Semaphore>,
}

/// Try to connect directly to a peer and fallback to indirect connection
/// if direct connection fails.
pub(crate) async fn connect_to_peer_with_fallback(
    request_peer_connection_tx: Sender<ServerRequest>,
    sse_tx: Sender<PeerResponse>,
    ready_tx: Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    database: Database,
    peer: &PeerEntity,
    conn_type: ConnectionType,
    search_limit: SearchLimit,
) -> Result<()> {
    shutdown_helper
        .limit_connections
        .acquire()
        .await
        .unwrap()
        .forget();

    debug!(
        "Available permit : {:?}",
        shutdown_helper.limit_connections.available_permits()
    );

    debug!("Trying direct {:?} connection to : {:?}", conn_type, peer);

    prepare_direct_connection_to_peer_with_fallback(
        channels.clone(),
        sse_tx.clone(),
        ready_tx.clone(),
        shutdown_helper.clone(),
        peer.clone(),
        database,
        search_limit.clone(),
    )
    .and_then(|handler| connect_direct(handler, conn_type))
    .or_else(|e| {
        warn!(
            "Direct connection to peer {:?} failed,  cause = {}",
            peer, e
        );

        request_indirect_connection(
            request_peer_connection_tx.clone(),
            channels.clone(),
            peer,
            conn_type,
        )
    })
    .await
}

// We were unable to connect to the peer, we are now asking the server
// to dispatch a connection request and expect a PierceFirewall message
// before upgrading the connection
async fn request_indirect_connection(
    request_peer_connection_tx: Sender<ServerRequest>,
    mut channels: SenderPool,
    peer: &PeerEntity,
    conn_type: ConnectionType,
) -> Result<()> {
    let token = random();

    // Save the channel state so we can later create the handler with the correct connection type
    channels.insert_indirect_connection_expected(&peer.username, conn_type, token);

    info!("Falling back to indirect connection with token {}", token);

    let server_request_sender = request_peer_connection_tx.clone();
    server_request_sender
        .send(ServerRequest::ConnectToPeer(RequestConnectionToPeer {
            token,
            username: peer.username.clone(),
            connection_type: conn_type,
        }))
        .await
        .map_err(|err| eyre!("Error sending connect to peer request {}", err))
}

async fn prepare_direct_connection_to_peer_with_fallback(
    channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    shutdown_helper: ShutdownHelper,
    peer: PeerEntity,
    db: Database,
    search_limit: SearchLimit,
) -> Result<PeerHandler> {
    let address = peer.get_address();
    let username = peer.username;

    match timeout(Duration::from_secs(1), TcpStream::connect(address)).await {
        Ok(Ok(socket)) => Ok(PeerHandler {
            peer_username: Some(username),
            connection: PeerConnection::new(socket),
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
            "Error in direct connection to peer {:?}, cause = {}",
            username,
            e
        )),
        Err(e) => Err(e.into()),
    }
}
