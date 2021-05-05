/// Code and documentation from this module have been heavily inspired by tokio [mini-redis](https://github.com/tokio-rs/mini-redis/blob/master/src/server.rs)
/// tutorial.
use std::future::Future;
use std::sync::Arc;

use rand::random;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::timeout;
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::database::Database;
use crate::message_common::ConnectionType;
use crate::peers::channels::SenderPool;
use crate::peers::connection::Connection;
use crate::peers::handler::Handler;
use crate::peers::messages::p2p::response::PeerResponse;
use crate::peers::messages::PeerRequestPacket;
use crate::peers::shutdown::Shutdown;
use crate::server::messages::peer::{Peer, PeerConnectionRequest, RequestConnectionToPeer};
use crate::server::messages::request::ServerRequest;
use crate::SlskError;

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
}

impl GlobalConnectionHandler {
    async fn run(&mut self, mut receivers: PeerListenerReceivers) -> crate::Result<()> {
        let server_request_tx = self.senders.server_request_tx.clone();
        let sse_tx = self.senders.sse_tx.clone();
        let channels = self.channels.clone();
        let db = self.db.clone();

        let shutdown_helper = self.shutdown_helper.clone();

        let _ = tokio::join!(
            dispatch_peer_message(receivers.peer_request_rx, channels.clone(), db.clone()),
            listen_for_indirect_connection(
                receivers.peer_connection_rx,
                sse_tx.clone(),
                channels.clone(),
                shutdown_helper.clone(),
                db.clone()
            ),
            self.accept_peer_connection(sse_tx.clone(), channels.clone(), db.clone()),
            connect_to_parents(
                server_request_tx,
                sse_tx,
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
    level = "debug",
    skip(self, channels, db)
    )]
    async fn accept_peer_connection(
        &mut self,
        sse_tx: mpsc::Sender<PeerResponse>,
        mut channels: SenderPool,
        db: Database,
    ) -> crate::Result<()> {
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

                        info!(
                            "Incoming direct connection from {:?} accepted",
                            socket.peer_addr()
                        );

                        debug!(
                            "Available connections : {}/{}",
                            self.shutdown_helper.limit_connections.available_permits(),
                            MAX_CONNECTIONS
                        );

                        let address = socket.peer_addr()?.ip().to_string();
                        let (rx, state) = channels.update_or_create(&address).await;

                        let mut handler = Handler {
                            peer_username: state.username.clone(),
                            connection: Connection::new(socket),
                            handler_rx: rx,
                            sse_tx: sse_tx.clone(),
                            shutdown: Shutdown::new(
                                self.shutdown_helper.notify_shutdown.subscribe(),
                            ),
                            limit_connections: self.shutdown_helper.limit_connections.clone(),
                            _shutdown_complete: self.shutdown_helper.shutdown_complete_tx.clone(),
                        };

                        let db_copy = db.clone();
                        let mut channels_coppy = channels.clone();
                        tokio::spawn(async move {
                            match handler
                                .listen(db_copy, state.conn_state.to_connection_type())
                                .await
                            {
                                Err(e) => error!(cause = ?e, "Error accepting inbound connection"),
                                Ok(()) => info!("Closing handler"),
                            };

                            channels_coppy.set_connection_lost(address).await;
                        });
                    }
                    Err(err) => {
                        error!("Failed to accept incoming connection : {}", err);
                    }
                };
            }
        } else {
            Err(crate::SlskError::NoPermitAvailable)
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
level = "debug",
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
async fn listen_for_indirect_connection(
    mut indirect_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    sse_tx: mpsc::Sender<PeerResponse>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    database: Database,
) -> Result<(), crate::Error> {
    while let Some(connection_request) = indirect_connection_rx.recv().await {
        let sse_tx = sse_tx.clone();
        if connection_request.username == "vessel" {
            continue;
        };

        let peer = Peer {
            username: connection_request.username.clone(),
            ip: connection_request.ip,
            port: connection_request.port,
        };

        let handler = get_connection(
            channels.clone(),
            sse_tx.clone(),
            shutdown_helper.clone(),
            peer.clone(),
        )
            .await;

        match handler {
            Ok(handler) => {
                spawn_peer_connection(channels.clone(), database.clone(), peer.clone(), connection_request.connection_type, handler).await;
            }
            Err(e) => {
                warn!("Error trying indirect connection to {:?} : {}", peer, e);
            }
        }
    }

    Ok(())
}

#[instrument(
level = "debug",
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
    mut channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    possible_parent_rx: &mut Receiver<Vec<Peer>>,
    database: Database,
) -> Result<(), crate::Error> {
    loop {
        while let Some(parents) = possible_parent_rx.recv().await {
            for parent in parents {
                let parent_count = channels.get_parent_count().await;
                debug!("Connected to {}/{} parents", parent_count, MAX_PARENT);

                if parent_count >= MAX_PARENT {
                    info!("Max parent count reached");
                    let server_request_sender = request_peer_connection_tx.clone();
                    server_request_sender
                        .send(ServerRequest::NoParents(false))
                        .await?;

                    return Ok(());
                };

                connect_to_peer(request_peer_connection_tx.clone(), sse_tx.clone(), &mut channels, shutdown_helper.clone(), database.clone(), &parent, ConnectionType::DistributedNetwork).await?;
            };
        };
    };
}

async fn connect_to_peer(request_peer_connection_tx: Sender<ServerRequest>, sse_tx: Sender<PeerResponse>, mut channels: &mut SenderPool, shutdown_helper: ShutdownHelper, database: Database, peer: &Peer, connection_type: ConnectionType)
                         -> Result<(), crate::Error> {
    let handler = get_connection(
        channels.clone(),
        sse_tx.clone(),
        shutdown_helper.clone(),
        peer.clone(),
    )
        .await;

    match handler {
        Ok(handler) => {
            spawn_peer_connection(channels.clone(), database.clone(), peer.clone(), connection_type, handler).await;
            Ok(())
        }
        Err(e) => {
            warn!(
                "Unable to establish parent connection with {:?},  cause = {}",
                peer, e
            );
            request_indirect_connection(
                request_peer_connection_tx.clone(),
                &mut channels,
                peer,
            ).await
                .map_err(|err| err.into())
        }
    }
}

async fn request_indirect_connection(
    request_peer_connection_tx: Sender<ServerRequest>,
    channels: &mut SenderPool,
    peer: &Peer,
) -> Result<(), SendError<ServerRequest>> {
    let token = random();

    // We were unable to connect to ths peer, we are now asking the server
    // to dispatch a connection request and expect a PierceFirewall message
    // before upgrading the connection to DistributedNetwork
    channels
        .new_incomming_indirect_parent_connection_pending(
            peer.username.clone(),
            peer.get_address(),
            token,
        )
        .await;

    info!("Falling back to indirect connection with token {}", token);
    let server_request_sender = request_peer_connection_tx.clone();

    server_request_sender
        .send(ServerRequest::ConnectToPeer(RequestConnectionToPeer {
            token,
            username: peer.username.clone(),
            connection_type: ConnectionType::DistributedNetwork,
        }))
        .await
}

async fn spawn_peer_connection(
    channels: SenderPool,
    database: Database,
    peer: Peer,
    connection_type: ConnectionType,
    mut handler: Handler,
) {
    info!(
        "Connected to distributed parent: {}@{:?}",
        &peer.username,
        peer.get_address()
    );

    database.insert_peer(&peer.username, peer.ip).unwrap();

    tokio::spawn(async move {
        match handler
            .connection_handshake(
                database.clone(),
                channels.clone(),
                connection_type,
            )
            .await
        {
            Ok(()) => info!("Connection with {:?}, closed", peer),
            Err(err) => error!(cause = ?err, "connection error"),
        }

        channels.clone().set_connection_lost(peer.ip.to_string()).await;
    });
}

#[instrument(level = "debug", skip(channels, shutdown_helper))]
async fn get_connection(
    mut channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    shutdown_helper: ShutdownHelper,
    user: Peer,
) -> crate::Result<Handler> {
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

    // Accept a new socket. This will attempt to perform error handling.
    // The `accept` method internally attempts to recover errors, so an
    // error here is non-recoverable.
    debug!(
        "Trying direct connection to peer at : {:?}",
        user.get_address()
    );

    match timeout(
        Duration::from_millis(2000),
        TcpStream::connect(user.get_address_with_port()),
    )
        .await
    {
        Ok(Ok(socket)) => {
            let rx = channels
                .new_named_peer_connection(user.username.clone(), user.get_address())
                .await;

            Ok(Handler {
                peer_username: Some(user.username),
                connection: Connection::new(socket),
                handler_rx: rx,
                sse_tx: sse_tx.clone(),
                shutdown: Shutdown::new(shutdown_helper.notify_shutdown.subscribe()),
                limit_connections: shutdown_helper.limit_connections.clone(),
                _shutdown_complete: shutdown_helper.shutdown_complete_tx.clone(),
            })
        }
        Ok(Err(_e)) => Err(SlskError::InvalidSocketAddress(user.get_address())),
        Err(e) => Err(SlskError::TimeOut(e)),
    }
}

#[instrument(
name = "peer_message_dispatcher",
level = "debug",
skip(peer_message_dispatcher, channels, database)
)]
async fn dispatch_peer_message(
    mut peer_message_dispatcher: Receiver<(String, PeerRequestPacket)>,
    mut channels: SenderPool,
    database: Database,
) {
    info!("Ready to dispatch peer messages");
    while let Some((username, message)) = peer_message_dispatcher.recv().await {
        // Resolve peer name against local db
        let address = database.get_peer_by_name(&username);

        // If we have an adress, try to get the sender for this peer
        match address {
            Some(address) => {
                info!("Incoming peer message from HTTP API for peer {:?}", address);
                match channels.send_message(address, message).await {
                    Ok(()) => {}
                    Err(e) => {
                        error!(cause = ?e, "Failed to send message to peer")
                    }
                };
            }
            None => error!("Peer address is unknown cannot send message"),
        };
    }
}
