use futures::future::FutureExt;
use futures::TryFutureExt;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::connection::SlskConnection;
use soulseek_protocol::server::messages::login::LoginRequest;
use soulseek_protocol::server::messages::peer::{Peer, PeerConnectionRequest};
use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::response::ServerResponse;

pub fn spawn_server_listener_task(
    mut http_rx: mpsc::Receiver<ServerRequest>,
    sse_tx: mpsc::Sender<ServerResponse>,
    peer_listener_tx: mpsc::Sender<PeerConnectionRequest>,
    mut request_peer_connection_from_server_rx: mpsc::Receiver<ServerRequest>,
    possible_parent_tx: mpsc::Sender<Vec<Peer>>,
    mut connection: SlskConnection,
    logged_in_tx: mpsc::Sender<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!("Starting Soulseek server TCP listener");
        loop {
            // Reveive all soulseek incoming messages, we stop reading from the soulseek connection
            // on [`SlskError::TimeOut`]
            if let Ok(Some(response)) = connection.read_response_with_timeout().await {
                info!(
                    "Got response from server {:?} : {:?}",
                    response.kind(),
                    response
                );

                match response {
                    ServerResponse::PeerConnectionRequest(connection_request) => {
                        info!(
                            "connect to peer request from server : {:?} ",
                            connection_request
                        );
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
                        logged_in_tx
                            .send(())
                            .await
                            .expect("error sending connection status to peer listener");
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
            if let Some(request) = http_rx.recv().now_or_never().flatten() {
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

            if let Some(request) = request_peer_connection_from_server_rx
                .recv()
                .now_or_never()
                .flatten()
            {
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
    })
}

pub fn spawn_sse_server(sse_rx: mpsc::Receiver<ServerResponse>) -> JoinHandle<()> {
    tokio::spawn(async {
        vessel_sse::start_sse_listener(sse_rx).await;
    })
}

pub fn spawn_peer_listener(
    peer_message_dispatcher: mpsc::Receiver<(String, PeerRequestPacket)>,
    peer_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    request_peer_connection_from_server_tx: mpsc::Sender<ServerRequest>,
    possible_parent_rx: mpsc::Receiver<Vec<Peer>>,
    logged_in_rx: mpsc::Receiver<()>,
    listener: TcpListener,
    database: Database,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        soulseek_protocol::peers::listener::run(
            listener,
            signal::ctrl_c(),
            peer_connection_rx,
            logged_in_rx,
            request_peer_connection_from_server_tx,
            possible_parent_rx,
            peer_message_dispatcher,
            database,
        )
        .await
        .expect("Unable to run peer listener");
    })
}

pub fn spawn_login_task(login_sender: mpsc::Sender<ServerRequest>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let listen_port_sender = login_sender.clone();
        let parent_request_sender = login_sender.clone();
        let join_nicotine_room = login_sender.clone();
        login_sender
            .send(ServerRequest::Login(LoginRequest::new("vessel", "lessev")))
            .and_then(|_| listen_port_sender.send(ServerRequest::SetListenPort(2255)))
            .and_then(|_| parent_request_sender.send(ServerRequest::NoParents(true)))
            .and_then(|_| join_nicotine_room.send(ServerRequest::JoinRoom("nicotine".to_string())))
            .await
            .expect("Unable to establish connection with soulseek server");
    })
}

pub fn spawn_http_listener(
    http_tx: mpsc::Sender<ServerRequest>,
    peer_message_dispatcher_tx: mpsc::Sender<(String, PeerRequestPacket)>,
    database: Database,
) -> JoinHandle<()> {
    tokio::spawn(async { vessel_http::start(http_tx, peer_message_dispatcher_tx, database).await })
}
