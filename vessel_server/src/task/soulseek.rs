use tokio::sync::mpsc::{Receiver, Sender};
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::server::response::ServerResponse;
use tokio::task::JoinHandle;
use crate::peers::search_limit::SearchLimit;
use crate::slsk::connection::SlskConnection;

pub fn spawn_server_listener_task(
    http_rx: Receiver<ServerRequest>,
    sse_tx: Sender<ServerResponse>,
    peer_listener_tx: Sender<PeerConnectionRequest>,
    request_peer_connection_rx: Receiver<ServerRequest>,
    possible_parent_tx: Sender<Vec<Peer>>,
    connection: SlskConnection,
    logged_in_tx: Sender<()>,
    peer_address_tx: Sender<PeerAddress>,
    search_limit: SearchLimit,
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
            search_limit,
        )
        .await;
    })
}

async fn server_listener(
    mut http_rx: Receiver<ServerRequest>,
    sse_tx: Sender<ServerResponse>,
    peer_listener_tx: Sender<PeerConnectionRequest>,
    mut request_peer_connection_rx: Receiver<ServerRequest>,
    possible_parent_tx: Sender<Vec<Peer>>,
    mut connection: SlskConnection,
    logged_in_tx: Sender<()>,
    peer_address_tx: Sender<PeerAddress>,
    search_limit: SearchLimit,
) {
    info!("Starting Soulseek server TCP listener");
    loop {
        tokio::select! {
             server_message = connection.read_response() => {
                 match server_message {
                     Ok(server_message) => {
                        if let Some(server_message) = server_message {
                            info!("Got message from Server {:?}", server_message);

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

                    if let ServerRequest::FileSearch(ref search) = request {
                        search_limit.reset(search.ticket);
                    }

                    connection.write_request(&request).await.expect("Error writing to soulseek connection");
                  }
              }

              peer_connection_request = request_peer_connection_rx.recv() => {
                if let Some(request) = peer_connection_request {
                    connection.write_request(&request).await.expect("Error writing to soulseek connection");
                }
              }
        }
    }
}
