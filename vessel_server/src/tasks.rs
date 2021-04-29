use futures::TryFutureExt;
use soulseek_protocol::database::Database;
use soulseek_protocol::peers::listener::PeerAddress;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::connection;
use soulseek_protocol::server::connection::SlskConnection;
use soulseek_protocol::server::messages::login::LoginRequest;
use soulseek_protocol::server::messages::peer::{Peer, PeerConnectionRequest};
use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::response::ServerResponse;
use soulseek_protocol::server::messages::user::Status;
use soulseek_protocol::SlskError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub fn spawn_server_listener_task(
    http_rx: mpsc::Receiver<ServerRequest>,
    sse_tx: mpsc::Sender<ServerResponse>,
    peer_listener_tx: mpsc::Sender<PeerConnectionRequest>,
    request_peer_connection_from_server_rx: mpsc::Receiver<ServerRequest>,
    possible_parent_tx: mpsc::Sender<Vec<Peer>>,
    connection: SlskConnection,
    logged_in_tx: mpsc::Sender<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        server_listener(
            http_rx,
            sse_tx,
            peer_listener_tx,
            request_peer_connection_from_server_rx,
            possible_parent_tx,
            connection,
            logged_in_tx,
        )
        .await;
    })
}

#[instrument(
    name = "slsk_server_listener",
    level = "debug",
    skip(
        http_rx,
        sse_tx,
        peer_listener_tx,
        request_peer_connection_from_server_rx,
        possible_parent_tx,
        connection,
        logged_in_tx
    )
)]
async fn server_listener(
    mut http_rx: mpsc::Receiver<ServerRequest>,
    sse_tx: mpsc::Sender<ServerResponse>,
    peer_listener_tx: mpsc::Sender<PeerConnectionRequest>,
    mut request_peer_connection_from_server_rx: mpsc::Receiver<ServerRequest>,
    possible_parent_tx: mpsc::Sender<Vec<Peer>>,
    mut connection: SlskConnection,
    logged_in_tx: mpsc::Sender<()>,
) {
    info!("Starting Soulseek server TCP listener");

    loop {
        tokio::select! {
             server_message = connection.read_response() => {
                 match server_message {
                     Ok(server_message) => {
                        if let Some(server_message) = server_message {
                            let err = match server_message {
                                ServerResponse::PeerConnectionRequest(connection_request) => {
                                    info!(
                                        "connect to peer request from server : {:?} ",
                                        connection_request
                                    );
                                    peer_listener_tx
                                        .send(connection_request)
                                        .await
                                        .map_err(|err| SlskError::Other(Box::new(err)))
                                }

                                ServerResponse::PossibleParents(parents) => {
                                    info!("Got possible parents from server");
                                    possible_parent_tx
                                        .send(parents)
                                        .await
                                        .map_err(|err| SlskError::Other(Box::new(err)))
                                }

                                ServerResponse::PrivilegedUsers(_) => {
                                    logged_in_tx
                                        .send(())
                                        .await
                                        .map_err(|err| SlskError::Other(Box::new(err)))
                                }

                                response => {
                                    info!("Got message from Soulseek server {}", response.kind());
                                    sse_tx
                                        .send(response)
                                        .await
                                        .map_err(|err| SlskError::Other(Box::new(err)))
                                }
                            };

                            if let Err(e) = err {
                                error!(" Error reading Soulseek stream : {}", e);
                                info!("Reconnecting");
                                connection = connection::connect().await;
                            };
                         }
                     }
                     Err(err) => error!("An error occured while reading soulseek server response : {:?}", err),
                    };
             },

              http_command = http_rx.recv() => {
                  if let Some(request) = http_command {
                        if let Some(new_connection) =  try_write(&mut connection, request)
                        .await
                        .expect("Failed to write to Soulseek connection") {
                        connection = new_connection;
                    }
                  }

              }

              peer_connection_request = request_peer_connection_from_server_rx.recv() => {
                if let Some(request) = peer_connection_request {
                    if let Some(new_connection) =  try_write(&mut connection, request)
                        .await
                        .expect("Failed to write to Soulseek connection") {
                        connection = new_connection;
                    }
                }
              }
        }
    }
}

// Write to the soulseek TCP stream or attempt to reconnect and retry once
async fn try_write(
    connection: &mut SlskConnection,
    request: ServerRequest,
) -> tokio::io::Result<Option<SlskConnection>> {
    debug!(
        "Sending {} request to server: {:?}",
        request.kind(),
        request
    );

    match &mut connection.write_request(&request).await {
        Ok(_) => Ok(None),
        Err(e) => {
            error!("Error writing request to soulseek connection : {}", e);
            info!("Reconnecting");
            let mut new_connection = connection::connect().await;
            let login = ServerRequest::Login(LoginRequest::new("vessel", "lessev"));
            let listen_port = ServerRequest::SetListenPort(2255);
            let no_parent = ServerRequest::NoParents(true);
            let join_room = ServerRequest::JoinRoom("nicotine".to_string());
            new_connection.write_request(&login).await?;
            new_connection.write_request(&listen_port).await?;
            new_connection.write_request(&no_parent).await?;
            new_connection.write_request(&join_room).await?;

            // Wait for PrivilegedUser message, indicating the connection is ready to receive a message
            while let Ok(response) = new_connection.read_response().await {
                match response {
                    Some(ServerResponse::PrivilegedUsers(_)) => {
                        info!("Reconnected");
                        break;
                    }
                    Some(other) => info!("Reconnecting, got {:?} from server", other.kind()),
                    None => {}
                };
            }

            info!("Setting online status");
            new_connection
                .write_request(&ServerRequest::SetOnlineStatus(Status::Online as u32))
                .await?;

            Ok(Some(new_connection))
        }
    }
}

pub fn spawn_sse_server(sse_rx: mpsc::Receiver<ServerResponse>) -> JoinHandle<()> {
    tokio::spawn(async {
        vessel_sse::start_sse_listener(sse_rx).await;
    })
}

type SenderPool = Arc<Mutex<HashMap<PeerAddress, mpsc::Sender<PeerRequestPacket>>>>;

pub fn spawn_peer_listener(
    peer_message_dispatcher: mpsc::Receiver<(String, PeerRequestPacket)>,
    peer_connection_rx: mpsc::Receiver<PeerConnectionRequest>,
    request_peer_connection_from_server_tx: mpsc::Sender<ServerRequest>,
    possible_parent_rx: mpsc::Receiver<Vec<Peer>>,
    mut logged_in_rx: mpsc::Receiver<()>,
    listener: TcpListener,
    database: Database,
) -> JoinHandle<()> {
    let channels: SenderPool = Arc::new(Mutex::new(HashMap::default()));

    tokio::spawn(async move {
        while logged_in_rx.recv().await.is_none() {
            // Wait for soulseek login
        }

        soulseek_protocol::peers::listener::run(
            listener,
            signal::ctrl_c(),
            peer_connection_rx,
            request_peer_connection_from_server_tx,
            possible_parent_rx,
            peer_message_dispatcher,
            database,
            channels.clone(),
        )
        .await
        .expect("Unable to run peer listener");
    })
}

pub fn spawn_login_task(login_sender: mpsc::Sender<ServerRequest>) -> JoinHandle<()> {
    debug!("Spawing loggin task");
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
