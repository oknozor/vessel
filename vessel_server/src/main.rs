#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing_subscriber::fmt::format::FmtSpan;

use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::messages::peer::{Peer, PeerConnectionRequest};
use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::response::ServerResponse;

use crate::tasks::spawn_server_listener_task;

const PEER_LISTENER_ADDRESS: &str = "0.0.0.0:2255";

mod tasks;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let (http_tx, http_rx) = mpsc::channel::<ServerRequest>(32);
    let (peer_message_dispatcher_tx, peer_message_dispatcher_rx) =
        mpsc::channel::<(String, PeerRequestPacket)>(32);
    let (sse_tx, sse_rx) = mpsc::channel::<ServerResponse>(32);
    let (peer_listener_tx, peer_connection_rx) = mpsc::channel::<PeerConnectionRequest>(32);
    let (request_peer_connection_from_server_tx, request_peer_connection_from_server_rx) =
        mpsc::channel::<ServerRequest>(32);
    let (possible_parent_tx, possible_parent_rx) = mpsc::channel::<Vec<Peer>>(32);
    let connection = soulseek_protocol::server::connection::connect().await;
    let login_sender = http_tx.clone();
    let (logged_in_tx, logged_in_rx) = mpsc::channel::<()>(1);


    let database = Database::new();

    // listen for incoming client commands and forward soulseek message to the sse service
    let soulseek_server_listener = spawn_server_listener_task(
        http_rx,
        sse_tx,
        peer_listener_tx,
        request_peer_connection_from_server_rx,
        possible_parent_tx,
        connection,
        logged_in_tx,
    );

    // Start the warp SSE server with a soulseek mpsc event receiver
    // this task will proxy soulseek events to the web clients
    let sse_server = tasks::spawn_sse_server(sse_rx);

    // Start the HTTP api proxy with the soulseek mpsc event sender
    // Here we are only sending request via HTTP and expect no other response
    // than 201/NO_CONTENT
    let http_server = tasks::spawn_http_listener(http_tx, peer_message_dispatcher_tx, database.clone());

    // Once every thing is ready we need to login before talking to the soulseek server
    // Vessel support one and only one user connection, credentials are retrieved from vessel configuration
    let login = tasks::spawn_login_task(login_sender);

    let listener = TcpListener::bind(PEER_LISTENER_ADDRESS).await?;

    // Listen for peer connection
    let peer_listener = tasks::spawn_peer_listener(
        peer_message_dispatcher_rx,
        peer_connection_rx,
        request_peer_connection_from_server_tx,
        possible_parent_rx,
        logged_in_rx,
        listener,
        database.clone()
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
