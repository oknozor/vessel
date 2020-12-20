#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use futures::TryFutureExt;
use soulseek_protocol::server_message::login::LoginRequest;
use soulseek_protocol::server_message::request::ServerRequest;
use soulseek_protocol::server_message::response::ServerResponse;
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
    let mut connection = soulseek_protocol::connection::connect().await;
    let mut login_sender = http_tx.clone();

    // listen for incoming client commands and forward soulseek message to the sse service
    let soulseek_listener = task::spawn(async move {
        loop {
            // Reveive all soulseek incoming messages, we stop reading from the soulseek connection
            // on [`SlskError::TimeOut`]
            while let Ok(Some(response)) = connection.read_response_with_timeout().await {
                info!("Got response {:?}", response.kind());
                sse_tx
                    .send(response)
                    .await
                    .expect("Unable to send message to sse listener");
            }

            // We try once to receive a command
            if let Ok(request) = http_rx.try_recv() {
                info!("Got request : {}", request.kind());
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
        login_sender
            .send(ServerRequest::Login(LoginRequest::new("vessel", "lessev")))
            .and_then(|_| listen_port_sender.send(ServerRequest::SetListenPort(2243)))
            .await
    });

    // Wraps everything with tokio::join so we don't block on servers startup
    let _ = join!(sse_server, http_server, soulseek_listener, login);

    // TODO : gracefull shutdown
    Ok(())
}
