use crate::peers::connection::PeerConnection;
use crate::peers::handler::PeerHandler;
use crate::peers::shutdown::Shutdown;
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use connection::PeerConnectionListener;
use eyre::Result;
use rand::random;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{
    PeerConnectionRequest, RequestConnectionToPeer,
};
use soulseek_protocol::server::request::ServerRequest;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::timeout;
use peer_manager::PeerConnectionManager;
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;

mod connection;

/// TODO : Make this value configurable
pub const MAX_CONNECTIONS: usize = 10_000;

pub mod distributed;
pub mod peer_to_peer;
pub mod indirect_connection;
pub mod peer_manager;

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
    };

    tokio::select! {
        res = server.run(
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
