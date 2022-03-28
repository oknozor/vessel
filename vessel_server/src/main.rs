#[macro_use]
extern crate tokio;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate eyre;

use tokio::{net::TcpListener, sync::mpsc};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{
    peers::{
        channels::SenderPool,
        listener::{PeerListenerReceivers, PeerListenerSenders},
    },
    tasks::spawn_server_listener_task,
};
use eyre::Result;
use soulseek_protocol::{
    peers::{p2p::response::PeerResponse, PeerRequestPacket},
    server::{
        peer::{Peer, PeerConnectionRequest},
        request::ServerRequest,
        response::ServerResponse,
    },
};
use vessel_database::Database;
use crate::peers::SearchLimit;

const PEER_LISTENER_ADDRESS: &str = "0.0.0.0:2255";

mod peers;
mod slsk;
mod tasks;

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
    let (peer_message_dispatcher_tx, peer_message_dispatcher_rx) =
        mpsc::channel::<(String, PeerRequestPacket)>(channel_bound);

    // Send Soulseek server response to the SSE client
    let (sse_tx, sse_rx) = mpsc::channel::<ServerResponse>(channel_bound);
    // Send Peer response to the SSE client
    let (sse_peer_tx, sse_peer_rx) = mpsc::channel::<PeerResponse>(channel_bound);

    // Dispatch incoming indirect connection request to the global peer handler
    let (peer_listener_tx, peer_connection_rx) =
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
    let soulseek_server_listener = spawn_server_listener_task(
        http_rx,
        sse_tx,
        peer_listener_tx,
        request_peer_connection_rx,
        possible_parent_tx,
        connection,
        logged_in_tx,
        peer_address_tx,
        search_limit.clone()
    );

    // Start the warp SSE server with a soulseek mpsc event receiver
    // this task will proxy soulseek events to the web clients
    let sse_server = tasks::spawn_sse_server(sse_rx, sse_peer_rx, download_progress_rx);

    // Start the HTTP api proxy with the soulseek mpsc event sender
    // Here we are only sending request via HTTP and expect no other response
    // than 201/NO_CONTENT
    let http_server =
        tasks::spawn_http_listener(http_tx, peer_message_dispatcher_tx, database.clone());

    // Once every thing is ready we need to login before talking to the soulseek server
    // Vessel support one and only one user connection, credentials are retrieved from vessel configuration
    let login = tasks::spawn_login_task(login_sender);

    let listener = TcpListener::bind(PEER_LISTENER_ADDRESS).await?;

    let channels = SenderPool::new(download_progress_tx);

    // Listen for peer connection
    let peer_listener = tasks::spawn_peer_listener(
        PeerListenerSenders {
            sse_tx: sse_peer_tx,
            server_request_tx: request_peer_connection_tx,
        },
        PeerListenerReceivers {
            peer_connection_rx,
            possible_parent_rx,
            peer_request_rx: peer_message_dispatcher_rx,
            peer_address_rx,
        },
        logged_in_rx,
        listener,
        database,
        channels,
        search_limit
    );

    // Wraps everything with tokio::join so we don't block on servers startup
    let _ = join!(
        sse_server,
        http_server,
        soulseek_server_listener,
        login,
        peer_listener
    );

    // TODO : gracefull shutdown
    Ok(())
}
