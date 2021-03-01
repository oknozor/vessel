/// Code and documentation from this module have been heavily inspired by tokio [mini-redis](https://github.com/tokio-rs/mini-redis/blob/master/src/server.rs)
/// tutorial.
use std::future::Future;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::peers::connection::Connection;
use crate::peers::handler::Handler;
use crate::peers::shutdown::Shutdown;
use crate::server::messages::peer::{Parent, PeerConnectionRequest};
use crate::server::messages::request::ServerRequest;
use crate::SlskError;
use std::net::Ipv4Addr;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

/// Maximum number of concurrent connections the peer server will accept.
///
/// When this limit is reached, the server will stop accepting connections until
/// an active connection terminates.
///
/// TODO : Make this value configurable
const MAX_CONNECTIONS: usize = 2048;

struct GlobalConnectionHandler {
    parent_connection: Arc<Mutex<Vec<Parent>>>,
    peer_listener: PeerListener,
    peer_connection_outgoing_tx: mpsc::Sender<ServerRequest>,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl GlobalConnectionHandler {
    async fn run(
        &mut self,
        peer_connection_rx: Receiver<PeerConnectionRequest>,
        mut possible_parent_rx: Receiver<Vec<Parent>>,
    ) -> crate::Result<()> {
        let limit_connections = self.limit_connections.clone();
        let notify_shutdown = self.notify_shutdown.clone();
        let shutdown_complete_tx = self.shutdown_complete_tx.clone();
        let parent_connections = self.parent_connection.clone();

        let parent = Parent {
            username: "adamka".to_string(),
            ip: Ipv4Addr::new(188, 156, 110, 114),
            port: 2234,
        };

        let handler = connect_to_peer(
            limit_connections.clone(),
            notify_shutdown.clone(),
            shutdown_complete_tx.clone(),
            &parent,
        )
        .await;
        handler?.run(self.peer_connection_outgoing_tx.clone()).await;

        tokio::join!(
            // self.listen(),
            connect_to_parents(
                limit_connections,
                notify_shutdown,
                shutdown_complete_tx,
                parent_connections,
                &mut possible_parent_rx
            )
        );
        Ok(())
    }

    async fn listen(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");

        loop {
            self.limit_connections.acquire().await.forget();
            debug!("Waiting for new connections");
            let socket = self.peer_listener.accept().await?;
            debug!("New connection accepted");

            // Create the necessary per-connection handler state.
            let mut handler = Handler {
                username: None,
                connection: Connection::new(socket),
                limit_connections: self.limit_connections.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            debug!("Spawning new distributed connection");
            let peer_connection_outgoing_tx = self.peer_connection_outgoing_tx.clone();
            tokio::spawn(async move {
                if let Err(err) = handler.run(peer_connection_outgoing_tx).await {
                    error!(cause = ?err, "connection error");
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

/// Run the soulseek peer server.
///
/// Accepts connections from the supplied listener. For each inbound connection,
/// a task is spawned to handle that connection. The server runs until the
/// `shutdown` future completes, at which point the server shuts down
/// gracefully.
///
/// `tokio::signal::ctrl_c()` can be used as the `shutdown` argument. This will
/// listen for a SIGINT signal.
pub async fn run(
    listener: TcpListener,
    shutdown: impl Future,
    peer_connection_rx: Receiver<PeerConnectionRequest>,
    mut logged_in_rx: mpsc::Receiver<()>,
    peer_connection_outgoing_tx: mpsc::Sender<ServerRequest>,
    possible_parent_rx: Receiver<Vec<Parent>>,
) -> crate::Result<()> {
    debug!("Waiting for user to be logged in");
    while let None = logged_in_rx.recv().await {
        // Waiting for login
    }
    debug!("User logged in, launching peer listener");

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    // Initialize the listener state
    let mut server = GlobalConnectionHandler {
        parent_connection: Arc::new(Mutex::new(vec![])),
        peer_listener: PeerListener { listener },
        peer_connection_outgoing_tx,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run(peer_connection_rx, possible_parent_rx) => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        err = shutdown => {
            info!("shutting down");
        }
    }

    let GlobalConnectionHandler {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        peer_connection_outgoing_tx,
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

async fn connect_to_parents(
    limit_connections: Arc<Semaphore>,
    notify_shutdown: Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    parent_connections: Arc<Mutex<Vec<Parent>>>,
    possible_parent_rx: &mut Receiver<Vec<Parent>>,
) {
    let mut connections = parent_connections.clone();

    while let Some(parents) = possible_parent_rx.recv().await {
        debug!("{:?}", parents);
        // for parent in parents {
        //     let mut connections = Arc::clone(&connections);
        //     let mut handler = connect_to_peer(
        //         limit_connections.clone(),
        //         notify_shutdown.clone(),
        //         shutdown_complete_tx.clone(),
        //         &parent,
        //     )
        //         .await;
        //     match handler {
        //         Ok(mut handler) => {
        //             let mut connections = connections.lock().expect("unable to acquire lock on parent connections");
        //             connections.push(parent.clone());
        //             info!("connected to parent:  {:?}", parent.get_address());
        //             info!("{} active parent connection", connections.len());

        //             tokio::spawn(async move {
        //                 // Process the connection. If an error is encountered, log it.
        //                 match handler.run(None).await {
        //                     Ok(()) => {}
        //                     Err(err) => error!(cause = ?err, "connection error"),
        //                 }
        //             });
        //         }
        //         Err(e) => error!("Unable to establish direct connection to peer, either port is closed or user is disconnected"),
        //     }
        // }
    }
}

async fn connect_to_peer(
    limit_connections: Arc<Semaphore>,
    notify_shutdown: Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
    user: &Parent,
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
        info!("connected");
        Ok(Handler {
            username: Some(user.username.clone()),
            connection: Connection::new(socket),
            limit_connections: limit_connections.clone(),
            shutdown: Shutdown::new(notify_shutdown.subscribe()),
            _shutdown_complete: shutdown_complete_tx.clone(),
        })
    } else {
        error!("timeout connection to peer : {:?}", user);
        Err(SlskError::from(format!(
            "Unable to connect to {:?}, falling back to indirect connection",
            user.get_address()
        )))
    }
}
