use crate::slsk::connection::SlskConnection;
use crate::SearchLimit;
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::server::response::ServerResponse;
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) mod connection;

pub struct SoulseekServerListener {
    pub http_rx: Receiver<ServerRequest>,
    pub sse_tx: Sender<ServerResponse>,
    pub peer_listener_tx: Sender<PeerConnectionRequest>,
    pub request_peer_connection_rx: Receiver<ServerRequest>,
    pub possible_parent_tx: Sender<Vec<Peer>>,
    pub connection: SlskConnection,
    pub logged_in_tx: Sender<()>,
    pub peer_address_tx: Sender<PeerAddress>,
    pub search_limit: SearchLimit,
}

impl SoulseekServerListener {
    pub async fn listen(&mut self) {
        info!("Starting Soulseek server TCP listener");
        loop {
            tokio::select! {
                 server_message = self.connection.read_response() => {
                     match server_message {
                         Ok(server_message) => {
                            if let Some(server_message) = server_message {
                                info!("Got message from Server {:?}", server_message);

                                let err = match server_message {
                                    ServerResponse::PeerAddress(peer_address) => {
                                        self.peer_address_tx.send(peer_address)
                                        .await
                                        .map_err(|err| eyre!("Error sending peer address to message dispatcher: {}", err))
                                    }


                                  ServerResponse:: PeerConnectionRequest(connection_request) => {
                                      let token = connection_request.token;

                                      self.peer_listener_tx
                                          .send(connection_request)
                                          .await
                                          .map_err(|err| eyre!("Error dispatching connection request with token {} to peer listener: {}", token, err))
                                  }

                                    ServerResponse::PossibleParents(parents) => {
                                        self.possible_parent_tx
                                            .send(parents)
                                            .await
                                            .map_err(|err| eyre!("Error dispatching possible parents to peer listener : {}", err))
                                    }

                                    ServerResponse::PrivilegedUsers(_) => {
                                        self.logged_in_tx
                                            .send(())
                                            .await
                                            .map_err(|err| eyre!("Error sending privileged user to login task: {}", err))
                                    }

                                    response => {
                                        self.sse_tx
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

                  http_command = self.http_rx.recv() => {
                      if let Some(request) = http_command {
                        info!("Got http request {:?}", request);

                        if let ServerRequest::FileSearch(ref search) = request {
                            self.search_limit.reset(search.ticket);
                        }

                        self.connection.write_request(&request).await.expect("Error writing to soulseek connection");
                      }
                  }

                  peer_connection_request = self.request_peer_connection_rx.recv() => {
                    if let Some(request) = peer_connection_request {
                        self.connection.write_request(&request).await.expect("Error writing to soulseek connection");
                    }
                  }
            }
        }
    }
}
