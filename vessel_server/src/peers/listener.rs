use std::net::{IpAddr, SocketAddr};
use std::{future::Future, sync::Arc};

use eyre::Result;
use futures::TryFutureExt;
use rand::random;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast, mpsc,
        mpsc::{Receiver, Sender},
        Semaphore,
    },
    time::{self, timeout, Duration},
};
use tracing::{error, info};

use soulseek_protocol::{
    message_common::ConnectionType,
    peers::{p2p::response::PeerResponse, PeerRequestPacket},
    server::{
        peer::{
            Peer, PeerAddress, PeerConnectionRequest, PeerConnectionTicket, RequestConnectionToPeer,
        },
        request::ServerRequest,
    },
    SlskError,
};
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;

use crate::peers::{
    channels::SenderPool,
    connection::PeerConnection,
    dispatcher::Dispatcher,
    handler::{connect_direct, pierce_firewall, PeerHandler},
    shutdown::Shutdown,
    SearchLimit,
};

/// TODO : Make this value configurable
const MAX_CONNECTIONS: usize = 10_000;
const MAX_PARENT: usize = 1;

#[derive(Debug, Clone)]
pub(crate) struct ShutdownHelper {
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    limit_connections: Arc<Semaphore>,
}

struct GlobalConnectionHandler {
    peer_listener: PeerListener,
    senders: PeerListenerSenders,
    db: Database,
    channels: SenderPool,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_helper: ShutdownHelper,
    search_limit: SearchLimit,
}

pub struct PeerListenerSenders {
    pub sse_tx: mpsc::Sender<PeerResponse>,
    pub server_request_tx: Sender<ServerRequest>,
}

// This is mainly used to avoid having to much arguments in function definition
pub struct PeerListenerReceivers {
    pub peer_listener_rx: Receiver<PeerConnectionRequest>,
    pub possible_parent_rx: Receiver<Vec<Peer>>,
    pub peer_request_rx: Receiver<(String, PeerRequestPacket)>,
    pub peer_address_rx: Receiver<PeerAddress>,
}

impl GlobalConnectionHandler {
    async fn run(&mut self, mut receivers: PeerListenerReceivers) -> crate::Result<()> {
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
    ) -> Result<()> {
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

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
#[derive(Debug)]
struct PeerListener {
    /// TCP listener supplied by the `run` caller.
    listener: TcpListener,
}

pub async fn run(
    listener: TcpListener,
    shutdown: impl Future,
    senders: PeerListenerSenders,
    receivers: PeerListenerReceivers,
    db: Database,
    channels: SenderPool,
    search_limit: SearchLimit,
) -> crate::Result<()> {
    debug!("Waiting for user to be logged in");

    info!("User logged in, starting peer listener");

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let connection_limit = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let shutdown_helper = ShutdownHelper {
        notify_shutdown,
        shutdown_complete_tx,
        limit_connections: connection_limit.clone(),
    };

    // Initialize the listener state
    let peer_listener = PeerListener { listener };

    let mut server = GlobalConnectionHandler {
        peer_listener,
        senders,
        db,
        channels,
        shutdown_complete_rx,
        shutdown_helper,
        search_limit: search_limit.clone(),
    };

    tokio::select! {
        res = server.run(receivers) => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        },
        _err = shutdown => {
            info!("shutting down");
            std::process::exit(0);
        }
    }

    let GlobalConnectionHandler {
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

impl PeerListener {
    /// Accept an inbound connection.
    ///
    /// Errors are handled by backing off and retrying. An exponential backoff
    /// strategy is used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each subsequent
    /// failure doubles the wait time. If accepting fails on the 6th try after
    /// waiting for 64 seconds, then this function returns with an error.
    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    if let Ok(_address) = socket.peer_addr() {
                        return Ok(socket);
                    };
                }
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
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
) -> Result<(), soulseek_protocol::Error> {
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
