use futures::TryFutureExt;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::JoinHandle;

use crate::peers;
use crate::peers::channels::SenderPool;
use crate::peers::listener::{PeerListenerReceivers, PeerListenerSenders};
use crate::slsk::connection::SlskConnection;
use soulseek_protocol::peers::p2p::download::DownloadProgress;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::login::LoginRequest;
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::server::response::ServerResponse;
use tokio::sync::mpsc::{Receiver, Sender};
use vessel_database::Database;

pub fn spawn_server_listener_task(
    http_rx: Receiver<ServerRequest>,
    sse_tx: Sender<ServerResponse>,
    peer_listener_tx: Sender<PeerConnectionRequest>,
    request_peer_connection_rx: Receiver<ServerRequest>,
    possible_parent_tx: Sender<Vec<Peer>>,
    connection: SlskConnection,
    logged_in_tx: Sender<()>,
    peer_address_tx: Sender<PeerAddress>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        server_listener(
            http_rx,
            sse_tx,
            peer_listener_tx,
            request_peer_connection_rx,
            possible_parent_tx,
            connection,
            logged_in_tx,
            peer_address_tx,
        )
        .await;
    })
}

#[instrument(
    name = "slsk_server_listener",
    level = "trace",
    skip(
        http_rx,
        sse_tx,
        peer_listener_tx,
        request_peer_connection_rx,
        possible_parent_tx,
        connection,
        logged_in_tx
    )
)]
async fn server_listener(
    mut http_rx: Receiver<ServerRequest>,
    sse_tx: Sender<ServerResponse>,
    peer_listener_tx: Sender<PeerConnectionRequest>,
    mut request_peer_connection_rx: Receiver<ServerRequest>,
    possible_parent_tx: Sender<Vec<Peer>>,
    mut connection: SlskConnection,
    logged_in_tx: Sender<()>,
    peer_address_tx: Sender<PeerAddress>,
) {
    info!("Starting Soulseek server TCP listener");
    loop {
        tokio::select! {
             server_message = connection.read_response() => {
                 match server_message {
                     Ok(server_message) => {
                        if let Some(server_message) = server_message {

                            info!("Got SERVER_MESSAGE::{:?}", server_message);
                            let err = match server_message {
                                ServerResponse::PeerAddress(peer_address) => {
                                    peer_address_tx.send(peer_address)
                                    .await
                                    .map_err(|err| eyre!("Error sending peer address to message dispatcher: {}", err))
                                }
                                ServerResponse:: PeerConnectionRequest(connection_request) => {
                                    let token = connection_request.token;

                                    peer_listener_tx
                                        .send(connection_request)
                                        .await
                                        .map_err(|err| eyre!("Error dispatching connection request with token {} to peer listener: {}", token, err))
                                }

                                ServerResponse::PossibleParents(parents) => {
                                    possible_parent_tx
                                        .send(parents)
                                        .await
                                        .map_err(|err| eyre!("Error dispatching possible parents to peer listener : {}", err))
                                }

                                ServerResponse::PrivilegedUsers(_) => {
                                    logged_in_tx
                                        .send(())
                                        .await
                                        .map_err(|err| eyre!("Error sending privileged user to login task: {}", err))
                                }
                                response => {
                                    sse_tx
                                        .send(response)
                                        .await
                                        .map_err(|err| eyre!("Error sending server response to SSE: {}", err))
                                }
                            };
                                if let Err(e) = err {
                                    return error!(" Error reading Soulseek stream : {}", e);
                            }
                        }
                     }
                     Err(err) => error!("An error occured while reading soulseek server response : {:?}", err),
                }
             },

              http_command = http_rx.recv() => {
                  if let Some(request) = http_command {
                    info!("Got http request {:?}", request);
                    connection.write_request(&request).await.expect("Error writing to soulseek connection")
                  }
              }

              peer_connection_request = request_peer_connection_rx.recv() => {
                if let Some(request) = peer_connection_request {
                    connection. write_request(&request).await.expect("Error writing to soulseek connection")
                }
              }
        }
    }
}

pub fn spawn_sse_server(
    sse_rx: Receiver<ServerResponse>,
    sse_peer_rx: Receiver<PeerResponse>,
    download_progress_rx: Receiver<DownloadProgress>,
) -> JoinHandle<()> {
    tokio::spawn(async {
        vessel_sse::start_sse_listener(sse_rx, sse_peer_rx, download_progress_rx).await;
    })
}

pub fn spawn_peer_listener(
    senders: PeerListenerSenders,
    receivers: PeerListenerReceivers,
    mut logged_in_rx: Receiver<()>,
    listener: TcpListener,
    database: Database,
    channels: SenderPool,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while logged_in_rx.recv().await.is_none() {
            // Wait for soulseek login
        }

        peers::listener::run(
            listener,
            signal::ctrl_c(),
            senders,
            receivers,
            database,
            channels.clone(),
        )
        .await
        .expect("Unable to run peer listener");
    })
}

pub fn spawn_login_task(login_sender: Sender<ServerRequest>) -> JoinHandle<()> {
    debug!("Spawning logging task");
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
    http_tx: Sender<ServerRequest>,
    peer_message_dispatcher_tx: Sender<(String, PeerRequestPacket)>,
    database: Database,
) -> JoinHandle<()> {
    tokio::spawn(async { vessel_http::start(http_tx, peer_message_dispatcher_tx, database).await })
}
