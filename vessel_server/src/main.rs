#[macro_use]
extern crate tokio;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate eyre;

use std::sync::Arc;
use tokio::{net::TcpListener, sync::mpsc};
use tracing_subscriber::fmt::format::FmtSpan;

use eyre::Result;
use futures::TryFutureExt;
use tokio::sync::Semaphore;
use soulseek_protocol::{
    peers::{p2p::response::PeerResponse, PeerRequestPacket},
    server::{
        peer::{Peer, PeerConnectionRequest},
        request::ServerRequest,
        response::ServerResponse,
    },
};
use soulseek_protocol::server::login::LoginRequest;
use state_manager::channel_manager::SenderPool;
use state_manager::search_limit::SearchLimit;
use vessel_database::Database;
use crate::peer_connection_manager::{MAX_CONNECTIONS, ShutdownHelper};
use crate::peer_connection_manager::distributed::DistributedConnectionManager;
use crate::peer_message_dispatcher::peer_message_dispatcher::Dispatcher;
use crate::slsk::SoulseekServerListener;

const PEER_LISTENER_ADDRESS: &str = "0.0.0.0:2255";

mod peer_connection_manager;
mod peer_message_dispatcher;
mod peers;
mod slsk;
mod state_manager;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// #[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Forward http request to the Soulseek server
    let channel_bound = 500;
    let (http_tx, http_rx) = mpsc::channel::<ServerRequest>(channel_bound);

    // Send http request directly to a peer connection
    let (peer_message_dispatcher_tx, peer_request_rx) =
        mpsc::channel::<(String, PeerRequestPacket)>(channel_bound);

    // Send Soulseek server response to the SSE client
    let (sse_tx, sse_rx) = mpsc::channel::<ServerResponse>(channel_bound);
    // Send Peer response to the SSE client
    let (sse_peer_tx, sse_peer_rx) = mpsc::channel::<PeerResponse>(channel_bound);

    // Dispatch incoming indirect connection request to the global peer handler
    let (peer_listener_tx, peer_listener_rx) =
        mpsc::channel::<PeerConnectionRequest>(channel_bound);

    // Request an indirect connection via http
    let (request_peer_connection_tx, request_peer_connection_rx) =
        mpsc::channel::<ServerRequest>(channel_bound);

    // Dispatch possible parrent to the global peer handler
    let (possible_parent_tx, possible_parent_rx) = mpsc::channel::<Vec<Peer>>(channel_bound);

    let connection = slsk::connection::connect().await;

    let login_sender = http_tx.clone();
    let (logged_in_tx, logged_in_rx) = mpsc::channel::<()>(1);
    let (peer_address_tx, peer_address_rx) = mpsc::channel(channel_bound);

    // Keep the UI updated about ongoing downloads
    let (download_progress_tx, download_progress_rx) = mpsc::channel(channel_bound);

    let database = Database::default();
    let search_limit = SearchLimit::new();
    // listen for incoming client commands and forward soulseek message to the sse service
    let mut soulseek_server_listener = SoulseekServerListener {
        http_rx,
        sse_tx,
        peer_listener_tx,
        request_peer_connection_rx,
        possible_parent_tx,
        connection,
        logged_in_tx,
        peer_address_tx,
        search_limit: search_limit.clone(),
    };

    // Start the warp SSE server with a soulseek mpsc event receiver
    // this task will proxy soulseek events to the web clients
    let sse_server = vessel_sse::start_sse_listener(sse_rx, sse_peer_rx, download_progress_rx);

    // Start the HTTP api proxy with the soulseek mpsc event sender
    // Here we are only sending request via HTTP and expect no other response
    // than 201/NO_CONTENT
    let http_server = vessel_http::start(http_tx, peer_message_dispatcher_tx, database.clone());

    // Once every thing is ready we need to login before talking to the soulseek server
    // Vessel support one and only one user connection, credentials are retrieved from vessel configuration
    let listen_port_sender = login_sender.clone();
    let parent_request_sender = login_sender.clone();
    let join_nicotine_room = login_sender.clone();
    let username = &vessel_database::settings::CONFIG.username;
    let password = &vessel_database::settings::CONFIG.password;

    let login =
        login_sender.send(ServerRequest::Login(LoginRequest::new(username, password)))
        .and_then(|_| listen_port_sender.send(ServerRequest::SetListenPort(2255)))
        .and_then(|_| parent_request_sender.send(ServerRequest::NoParents(true)))
        .and_then(|_| join_nicotine_room.send(ServerRequest::JoinRoom("nicotine".to_string())));

    let listener = TcpListener::bind(PEER_LISTENER_ADDRESS).await?;

    let channels = SenderPool::new(download_progress_tx);

    let (ready_tx, ready_rx) = mpsc::channel(32);

    let (notify_shutdown, _) = tokio::sync::broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);
    let shutdown_helper = ShutdownHelper {
        notify_shutdown,
        shutdown_complete_tx,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS))
        ,
    };

    let mut distributed_connection_manager = DistributedConnectionManager {
        request_peer_connection_tx: request_peer_connection_tx.clone(),
        sse_tx: sse_peer_tx.clone(),
        ready_tx: ready_tx.clone(),
        channels: channels.clone(),
        shutdown_helper: shutdown_helper.clone(),
        possible_parent_rx,
        database: database.clone(),
        search_limit: search_limit.clone(),
    };

    let mut dispatcher = Dispatcher {
        ready_rx,
        queue_rx: peer_request_rx,
        peer_address_rx,
        channels: channels.clone(),
        db: database.clone(),
        shutdown_helper: shutdown_helper.clone(),
        sse_tx: sse_peer_tx.clone(),
        ready_tx: ready_tx.clone(),
        server_request_tx: request_peer_connection_tx.clone(),
        message_queue: Default::default(),
        search_limit: search_limit.clone(),
    };

    // Listen for peer connection
    let peer_listener = crate::peer_connection_manager::run(
        listener,
        tokio::signal::ctrl_c(),
        sse_peer_tx,
        request_peer_connection_tx,
        peer_listener_rx,
        database.clone(),
        channels.clone(),
        search_limit.clone(),
        ready_tx.clone(),
        logged_in_rx,
        shutdown_helper.clone(),
        shutdown_complete_rx
    );

    // Wraps everything with tokio::join so we don't block on servers startup
    let _ = join!(
        distributed_connection_manager.run(),
        dispatcher.run(),
        sse_server,
        http_server,
        soulseek_server_listener.listen(),
        login,
        peer_listener
    );

    // TODO : gracefull shutdown
    Ok(())
}
