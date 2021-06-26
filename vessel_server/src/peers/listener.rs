use std::future::Future;
use std::sync::Arc;

use rand::random;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::timeout;
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::peers::channels::SenderPool;
use crate::peers::connection::PeerConnection;
use crate::peers::dispatcher::Dispatcher;
use crate::peers::handler::{connect_direct, pierce_firewall, PeerHandler};
use crate::peers::shutdown::Shutdown;
use eyre::Result;
use futures::TryFutureExt;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::peer::{
    Peer, PeerAddress, PeerConnectionRequest, PeerConnectionTicket, RequestConnectionToPeer,
};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::SlskError;
use std::net::{IpAddr, SocketAddr};
use vessel_database::entities::PeerEntity;
use vessel_database::Database;

/// TODO : Make this value configurable
const MAX_CONNECTIONS: usize = 4096;
const MAX_PARENT: usize = 10;

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
}

pub struct PeerListenerSenders {
    pub sse_tx: mpsc::Sender<PeerResponse>,
    pub server_request_tx: Sender<ServerRequest>,
}

// This is mainly used to avoid having to much arguments in function definition
pub struct PeerListenerReceivers {
    pub peer_connection_rx: Receiver<PeerConnectionRequest>,
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
        };

        let _ = tokio::join!(
            dispatcher.run(),
            listen_indirect_peer_connection_request(
                server_request_tx.clone(),
                receivers.peer_connection_rx,
                sse_tx.clone(),
                ready_tx.clone(),
                channels.clone(),
                shutdown_helper.clone(),
                db.clone()
            ),
            self.accept_peer_connection(
                sse_tx.clone(),
                ready_tx.clone(),
                channels.clone(),
                db.clone()
            ),
            connect_to_parents(
                server_request_tx,
                sse_tx,
                ready_tx,
                channels,
                shutdown_helper.clone(),
                &mut receivers.possible_parent_rx,
                db
            )
        );

        Ok(())
    }

    #[instrument(
        name = "peer_connection_handler",
        level = "trace",
        skip(self, channels, db, sse_tx, ready_tx)
    )]
    async fn accept_peer_connection(
        &mut self,
        sse_tx: mpsc::Sender<PeerResponse>,
        ready_tx: mpsc::Sender<u32>,
        channels: SenderPool,
        db: Database,
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
                            _shutdown_complete: self.shutdown_helper.shutdown_complete_tx.clone(),
                            connection_states: channels.clone(),
                            db: db.clone(),
                            address,
                            token: None,
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

#[instrument(
    name = "peers_listener",
    level = "trace",
    skip(listener, shutdown, senders, receivers, db, channels)
)]
pub async fn run(
    listener: TcpListener,
    shutdown: impl Future,
    senders: PeerListenerSenders,
    receivers: PeerListenerReceivers,
    db: Database,
    channels: SenderPool,
) -> crate::Result<()> {
    debug!("Waiting for user to be logged in");

    info!("User logged in, starting peer listener");

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let shutdown_helper = ShutdownHelper {
        notify_shutdown,
        shutdown_complete_tx,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    // Initialize the listener state
    let peer_listener = PeerListener { listener };

    let mut server = GlobalConnectionHandler {
        peer_listener,
        senders,
        db,
        channels,
        shutdown_helper,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run(receivers) => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
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

// Receive connection request from server, and attempt direct connection to peeer
async fn listen_indirect_peer_connection_request(
    server_request_tx: mpsc::Sender<ServerRequest>,
    mut indirect_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    db: Database,
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

        debug!(
            "Available permit : {:?}",
            shutdown_helper.limit_connections.available_permits()
        );

        debug!(
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
        )
        .and_then(|handler| pierce_firewall(handler, token))
        .await;

        if let Err(err) = connection_result {
            debug!(
                "Unable to establish requested connection to peer, {}, token = {}, cause = {}",
                username, token, err
            );

            server_request_tx
                .send(ServerRequest::CantConnectToPeer(PeerConnectionTicket {
                    token,
                    username,
                }))
                .await?;
        }
    }

    Ok(())
}

#[instrument(
    level = "trace",
    skip(
        request_peer_connection_tx,
        channels,
        shutdown_helper,
        possible_parent_rx,
        database
    )
)]
async fn connect_to_parents(
    request_peer_connection_tx: mpsc::Sender<ServerRequest>,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    possible_parent_rx: &mut Receiver<Vec<Peer>>,
    database: Database,
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
                )
                .await?;
            }
        }
    }
}

// Try to connect directly to a peer and fallback to indirect connection
// if direct connection fails
pub(crate) async fn connect_to_peer_with_fallback(
    request_peer_connection_tx: Sender<ServerRequest>,
    sse_tx: Sender<PeerResponse>,
    ready_tx: Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    database: Database,
    peer: &PeerEntity,
    conn_type: ConnectionType,
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
    )
    .and_then(|handler| connect_direct(handler, conn_type))
    .or_else(|e| {
        warn!(
            "Unable to establish peer connection with {:?},  cause = {}",
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

// We were unable to connect to ths peer, we are now asking the server
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

#[instrument(level = "trace", skip(channels, shutdown_helper))]
async fn prepare_direct_connection_to_peer(
    channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    shutdown_helper: ShutdownHelper,
    connection_request: PeerConnectionRequest,
    address: SocketAddr,
    db: Database,
) -> Result<PeerHandler> {
    let username = connection_request.username.clone();

    match timeout(Duration::from_secs(4), TcpStream::connect(address)).await {
        Ok(Ok(socket)) => {
            let token = Some(connection_request.token);
            Ok(PeerHandler {
                peer_username: Some(username),
                connection: PeerConnection::new(socket),
                sse_tx: sse_tx.clone(),
                ready_tx,
                shutdown: Shutdown::new(shutdown_helper.notify_shutdown.subscribe()),
                limit_connections: shutdown_helper.limit_connections.clone(),
                _shutdown_complete: shutdown_helper.shutdown_complete_tx.clone(),
                connection_states: channels,
                db,
                address,
                token,
            })
        }
        Ok(Err(e)) => Err(eyre!(
            "Error connecting to peer {:?} via server requested connection {:?}, cause = {}",
            username,
            connection_request,
            e
        )),
        Err(e) => Err(e.into()),
    }
}

#[instrument(level = "trace", skip(channels, shutdown_helper))]
async fn prepare_direct_connection_to_peer_with_fallback(
    channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    shutdown_helper: ShutdownHelper,
    peer: PeerEntity,
    db: Database,
) -> Result<PeerHandler> {
    let address = peer.get_address();
    let username = peer.username;

    match timeout(Duration::from_millis(2000), TcpStream::connect(address)).await {
        Ok(Ok(socket)) => Ok(PeerHandler {
            peer_username: Some(username),
            connection: PeerConnection::new(socket),
            sse_tx: sse_tx.clone(),
            ready_tx,
            shutdown: Shutdown::new(shutdown_helper.notify_shutdown.subscribe()),
            limit_connections: shutdown_helper.limit_connections.clone(),
            _shutdown_complete: shutdown_helper.shutdown_complete_tx.clone(),
            connection_states: channels,
            db,
            address,
            token: None,
        }),
        Ok(Err(e)) => Err(eyre!(
            "Error in direct connection to peer {:?}, cause = {}",
            username,
            e
        )),
        Err(e) => Err(e.into()),
    }
}
