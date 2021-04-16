/// Code and documentation from this module have been heavily inspired by tokio [mini-redis](https://github.com/tokio-rs/mini-redis/blob/master/src/server.rs)
/// tutorial.
use std::future::Future;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::database::Database;
use crate::message_common::ConnectionType;
use crate::peers::connection::Connection;
use crate::peers::handler::Handler;
use crate::peers::messages::PeerRequestPacket;
use crate::peers::shutdown::Shutdown;
use crate::server::messages::peer::{Peer, PeerConnectionRequest, RequestConnectionToPeer};
use crate::server::messages::request::ServerRequest;
use crate::SlskError;
use rand::random;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

type SenderPool = Arc<Mutex<HashMap<PeerAddress, mpsc::Sender<PeerRequestPacket>>>>;

#[derive(Debug, Clone, Eq)]
struct PeerAddress {
    address: String,
    is_parent: bool,
}

impl PeerAddress {
    fn new(address: String) -> Self {
        PeerAddress {
            address,
            is_parent: false,
        }
    }

    fn new_parent(address: String) -> Self {
        PeerAddress {
            address,
            is_parent: true,
        }
    }
}

impl Hash for PeerAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl PartialEq for PeerAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

/// TODO : Make this value configurable
const MAX_CONNECTIONS: usize = 2048;
const MAX_PARENT: usize = 10;

struct GlobalConnectionHandler {
    peer_listener: PeerListener,
    server_request_tx: mpsc::Sender<ServerRequest>,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl GlobalConnectionHandler {
    async fn run(
        &mut self,
        peer_connection_rx: Receiver<PeerConnectionRequest>,
        server_request_tx: mpsc::Sender<ServerRequest>,
        mut possible_parent_rx: Receiver<Vec<Peer>>,
        peer_message_dispatcher: Receiver<(String, PeerRequestPacket)>,
        database: Database,
    ) -> crate::Result<()> {
        let limit_connections = self.limit_connections.clone();
        let notify_shutdown = self.notify_shutdown.clone();
        let shutdown_complete_tx = self.shutdown_complete_tx.clone();

        let channels: SenderPool = Arc::new(Mutex::new(HashMap::default()));

        let _ = tokio::join!(
            dispatch_peer_message(peer_message_dispatcher, channels.clone(), database.clone()),
            listen_for_indirect_connection(
                peer_connection_rx,
                server_request_tx.clone(),
                channels.clone(),
                limit_connections.clone(),
                notify_shutdown.clone(),
                shutdown_complete_tx.clone(),
                database.clone()
            ),
            self.listen(channels.clone(), database.clone()),
            connect_to_parents(
                server_request_tx,
                channels.clone(),
                limit_connections,
                notify_shutdown,
                shutdown_complete_tx,
                &mut possible_parent_rx,
                database
            )
        );

        Ok(())
    }

    async fn listen(&mut self, channels: SenderPool, database: Database) -> crate::Result<()> {
        info!("accepting inbound connections");

        loop {
            self.limit_connections.acquire().await.forget();
            let socket = self.peer_listener.accept().await?;

            debug!(
                "Incoming direct connection from {:?} accepted",
                socket.peer_addr()
            );

            let address = socket.peer_addr()?.ip().to_string();
            let (tx, rx) = mpsc::channel(100);

            let channels = channels.clone();
            let mut channels = channels
                .lock()
                .expect("Unable to acquire lock on peer channels");

            channels.insert(PeerAddress::new(address.clone()), tx);

            let mut handler = Handler {
                peer_username: None,
                connection: Connection::new(socket),
                handler_rx: rx,
                limit_connections: self.limit_connections.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            debug!("Spawning new peer connection");
            let db_copy = database.clone();

            tokio::spawn(async move {
                if let Err(err) = handler.listen(db_copy).await {
                    error!(cause = ?err, "Error accepting inbound connection");
                }
            });
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
    peer_connection_rx: Receiver<PeerConnectionRequest>,
    mut logged_in_rx: mpsc::Receiver<()>,
    peer_connection_outgoing_tx: mpsc::Sender<ServerRequest>,
    possible_parent_rx: Receiver<Vec<Peer>>,
    peer_message_dispatcher: mpsc::Receiver<(String, PeerRequestPacket)>,
    database: Database,
) -> crate::Result<()> {
    debug!("Waiting for user to be logged in");

    while logged_in_rx.recv().await.is_none() {
        // Waiting for login
    }

    info!("User logged in, starting peer listener");

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    // Initialize the listener state
    let mut server = GlobalConnectionHandler {
        peer_listener: PeerListener { listener },
        server_request_tx: peer_connection_outgoing_tx.clone(),
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run(peer_connection_rx, peer_connection_outgoing_tx, possible_parent_rx, peer_message_dispatcher, database) => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _err = shutdown => {
            info!("shutting down");
        }
    }

    let GlobalConnectionHandler {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        server_request_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_complete_tx);

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
                    info!("{}", socket.peer_addr().unwrap());
                    return Ok(socket);
                }
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::delay_for(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}

async fn listen_for_indirect_connection(
    mut indirect_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    server_request_tx: mpsc::Sender<ServerRequest>,
    channels: SenderPool,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    database: Database,
) -> Result<(), SendError<ServerRequest>> {
    while let Some(connection_request) = indirect_connection_rx.recv().await {
        if connection_request.username == "vessel" {
            continue;
        };

        let peer = Peer {
            username: connection_request.username.clone(),
            ip: connection_request.ip,
            port: connection_request.port,
        };

        let handler = connect_to_peer(
            channels.clone(),
            limit_connections.clone(),
            notify_shutdown.clone(),
            shutdown_complete_tx.clone(),
            peer.clone(),
        )
        .await;

        match handler {
            Ok(mut handler) => {
                info!(
                    "Connected to peer: {}@{:?}",
                    &peer.username,
                    peer.get_address()
                );

                let db_copy = database.clone();

                tokio::spawn(async move {
                    match handler.connect(db_copy).await {
                        Ok(()) => {}
                        Err(err) => error!(cause = ?err, "connection error"),
                    }
                });
            }
            Err(_) => {
                warn!("Unable to establish indirect connection to peer, either port is closed or user is disconnected");
                warn!("Aborting");
            }
        }
    }

    Ok(())
}

async fn connect_to_parents(
    server_request_tx: mpsc::Sender<ServerRequest>,
    channels: SenderPool,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    possible_parent_rx: &mut Receiver<Vec<Peer>>,
    database: Database,
) -> Result<(), SendError<ServerRequest>> {
    while let Some(parents) = possible_parent_rx.recv().await {
        for parent in parents {
            let parent_count;

            {
                let channel_pool = channels
                    .lock()
                    .expect("Unable to acquire lock on sender pool");
                parent_count = channel_pool
                    .keys()
                    .filter(|address| address.is_parent)
                    .count();

                debug!("Connected to {}/{} parents", parent_count, MAX_PARENT);
            }

            if parent_count >= MAX_PARENT {
                debug!("Max parent count reached");
                let mut server_request_sender = server_request_tx.clone();
                server_request_sender
                    .send(ServerRequest::NoParents(false))
                    .await?;
                return Ok(());
            };

            let handler = connect_to_peer(
                channels.clone(),
                limit_connections.clone(),
                notify_shutdown.clone(),
                shutdown_complete_tx.clone(),
                parent.clone(),
            )
            .await;

            match handler {
                Ok(mut handler) => {
                    info!(
                        "Connected to distributed parent: {}@{:?}",
                        &parent.username,
                        parent.get_address()
                    );

                    database.insert_peer(&parent.username, parent.ip).unwrap();

                    let db_copy = database.clone();

                    tokio::spawn(async move {
                        match handler.connect(db_copy).await {
                            Ok(()) => {}
                            Err(err) => error!(cause = ?err, "connection error"),
                        }
                    });
                }
                Err(_) => {
                    warn!("Unable to establish direct connection to peer, either port is closed or user is disconnected");
                    warn!("Falling back to indirect connection");
                    let mut server_request_sender = server_request_tx.clone();
                    server_request_sender
                        .send(ServerRequest::ConnectToPeer(RequestConnectionToPeer {
                            token: random(),
                            username: "vessel".to_string(),
                            connection_type: ConnectionType::DistributedNetwork,
                        }))
                        .await?;
                }
            }
        }
    }

    unreachable!()
}

async fn connect_to_peer(
    channels: SenderPool,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    user: Peer,
) -> crate::Result<Handler> {
    limit_connections.acquire().await.forget();

    // Accept a new socket. This will attempt to perform error handling.
    // The `accept` method internally attempts to recover errors, so an
    // error here is non-recoverable.
    debug!(
        "Trying direct connection to peer at : {:?}",
        user.get_address()
    );

    if let Ok(Ok(socket)) = timeout(
        Duration::from_millis(2000),
        TcpStream::connect(user.get_address()),
    )
    .await
    {
        let (tx, rx) = mpsc::channel(100);
        let channels = channels.clone();
        let mut channels = channels
            .lock()
            .expect("Unable to acquire lock on peer channels");
        channels.insert(PeerAddress::new_parent(user.ip.to_string()), tx);

        Ok(Handler {
            peer_username: Some(user.username.clone()),
            connection: Connection::new(socket),
            handler_rx: rx,
            limit_connections: limit_connections.clone(),
            shutdown: Shutdown::new(notify_shutdown.subscribe()),
            _shutdown_complete: shutdown_complete_tx.clone(),
        })
    } else {
        warn!("timeout connecting to peer : {:?}", user);
        Err(SlskError::from(format!(
            "Unable to connect to {:?}, falling back to indirect connection",
            user.get_address()
        )))
    }
}

async fn dispatch_peer_message(
    mut peer_message_dispatcher: Receiver<(String, PeerRequestPacket)>,
    channels: SenderPool,
    database: Database,
) {
    while let Some((username, message)) = peer_message_dispatcher.recv().await {
        let address = database.get_peer_by_name(&username);

        if let Some(address) = address {
            debug!("Incoming peer message from HTTP API for peer {:?}", address);
            let sender;
            {
                let channel_pool = channels
                    .lock()
                    .expect("Unable to acquire lock on peer channel pool");
                let channel = channel_pool.get(&PeerAddress::new(address.clone()));

                sender = if let Some(peer_sender) = channel {
                    debug!("Found channel for peer {}@{}", username, address);
                    Some(peer_sender.clone())
                } else {
                    None
                };
            }

            if let Some(mut sender) = sender {
                debug!("Sending message to peer handler {:?}", message);
                sender.send(message).await.expect("Send error");
            }
        }
    }
}
