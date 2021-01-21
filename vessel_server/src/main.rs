#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use futures::TryFutureExt;
use soulseek_protocol::server::messages::login::LoginRequest;
use soulseek_protocol::server::messages::peer::{Parent, PeerConnectionRequest};
use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::response::ServerResponse;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let (http_tx, mut http_rx) = mpsc::channel::<ServerRequest>(32);
    let (mut sse_tx, sse_rx) = mpsc::channel::<ServerResponse>(32);
    let (mut peer_listener_tx, peer_connection_rx) = mpsc::channel::<PeerConnectionRequest>(32);
    let (peer_connection_outgoing_tx, mut peer_connection_outgoing_rx) = mpsc::channel::<ServerRequest>(32);
    let (mut possible_parent_tx, possible_parent_rx) = mpsc::channel::<Vec<Parent>>(32);
    let mut connection = soulseek_protocol::server::connection::connect().await;
    let mut login_sender = http_tx.clone();
    let (mut logged_in_tx, logged_in_rx) = mpsc::channel::<()>(1);

    // listen for incoming client commands and forward soulseek message to the sse service
    let soulseek_server_listener = task::spawn(async move {
        loop {
            // Reveive all soulseek incoming messages, we stop reading from the soulseek connection
            // on [`SlskError::TimeOut`]
            while let Ok(Some(response)) = connection.read_response_with_timeout().await {
                info!(
                    "Got response from server {:?} : {:?}",
                    response.kind(),
                    response
                );
                match response {
                    ServerResponse::ConnectToPeer(connection_request) => {
                        info!("connect to peer request from server : {:?} ", connection_request);
                        peer_listener_tx
                            .send(connection_request)
                            .await
                            .expect("Cannot send peer connection request to peer handler");
                    }

                    ServerResponse::PossibleParents(parents) => {
                        info!("Got possible parents from server");
                        possible_parent_tx
                            .send(parents)
                            .await
                            .expect("Unable to send possible parent to peer handler");
                    }

                    ServerResponse::PrivilegedUsers(_) => {
                        logged_in_tx.send(()).await.expect("error sending connection status to peer listener");
                    }

                    response => {
                        sse_tx
                            .send(response)
                            .await
                            .expect("Unable to send message to sse listener");
                    }
                }
            }

            // We try once to receive a command
            if let Ok(request) = http_rx.try_recv() {
                debug!(
                    "Sending {} request to server: {:?}",
                    request.kind(),
                    request
                );
                connection
                    .write_request(request)
                    .await
                    .expect("failed to write to soulseek connection");
            }

            if let Ok(request) = peer_connection_outgoing_rx.try_recv() {
                debug!(
                    "Sending {} peer connection request to server: {:?}",
                    request.kind(),
                    request
                );
                connection
                    .write_request(request)
                    .await
                    .expect("failed to write to soulseek connection");
            }
        }
    });

    // Start the warp SSE server with a soulseek mpsc event receiver
    // this task will proxy soulseek events to the web clients
    let sse_server = task::spawn(async {
        vessel_sse::start_sse_listener(sse_rx).await;
    });

    // Start the HTTP api proxy with the soulseek mpsc event sender
    // Here we are only sending request via HTTP and expect no other response
    // than 201/NO_CONTENT
    let http_server = task::spawn(async { vessel_http::start(http_tx).await });

    // Once every thing is ready we need to login before talking to the soulseek server
    // Vessel support one and only one user connection, credentials are retrieved from vessel configuration
    let login = task::spawn(async move {
        let mut listen_port_sender = login_sender.clone();
        let mut parent_request_sender = login_sender.clone();
        login_sender
            .send(ServerRequest::Login(LoginRequest::new("vessel", "lessev")))
            .and_then(|_| listen_port_sender.send(ServerRequest::SetListenPort(2255)))
            .and_then(|_| parent_request_sender.send(ServerRequest::NoParents(true)))
            .await
            .expect("Unable to establish connection with soulseek server");
    });

    let listener = TcpListener::bind("127.0.0.1:2255").await?;

    // Listen for peer connection
    let peer_listener = task::spawn(async move {
        soulseek_protocol::peers::listener::run(
            listener,
            signal::ctrl_c(),
            peer_connection_rx,
            logged_in_rx,
            peer_connection_outgoing_tx,
            possible_parent_rx,
        )
        .await
        .expect("Unable to run peer listener");
    });

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
