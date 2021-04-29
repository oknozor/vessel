use std::sync::Arc;

use rand::random;
use tokio::sync::{mpsc, Semaphore};

use crate::database::Database;
use crate::message_common::ConnectionType;
use crate::peers::connection::Connection;
use crate::peers::messages::connection::PeerConnectionMessage;
use crate::peers::messages::p2p::request::PeerRequest;
use crate::peers::messages::p2p::response::PeerResponse;
use crate::peers::messages::p2p::transfer::TransferReply::TransferReplyOk;
use crate::peers::messages::p2p::transfer::TransferRequest;
use crate::peers::messages::p2p::user_info::UserInfo;
use crate::peers::messages::PeerRequestPacket;
use crate::peers::messages::PeerResponsePacket;
use crate::peers::shutdown::Shutdown;
use std::time::Duration;

#[derive(Debug)]
pub struct Handler {
    pub peer_username: Option<String>,
    pub(crate) connection: Connection,
    pub(crate) handler_rx: mpsc::Receiver<PeerRequestPacket>,
    pub(crate) limit_connections: Arc<Semaphore>,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn listen(&mut self, db: Database) -> crate::Result<()> {
        let mut backoff = 1;

        while !self.shutdown.is_shutdown() {
            match self.connection.connection_type {
                Some(ConnectionType::FileTransfer) => {
                    return self
                        .connection
                        .download(db.clone())
                        .await
                        .map_err(|err| err.into());
                }
                Some(_) | None => {
                    // While reading a request frame, also listen for the shutdown
                    // signal.
                    tokio::select! {
                        response = self.connection.read_response() =>  {
                            match response {
                                Ok(message) => match message {
                                    Some(PeerResponsePacket::ConnectionMessage(message)) => {
                                        info!("Got connection message : {:?}", &message);
                                        if let Err(e) = self.handle_connection_message(&message, db.clone()).await {
                                            return Err(format!("Connection error : {}", e).into());
                                        }
                                    }
                                    Some(PeerResponsePacket::Message(message)) => {
                                        info!("Got message from peer {:?} : {:?}", self.peer_username, &message);
                                        if let Err(e) = self.handle_peer_message(&message, db.clone()).await {
                                            return Err(format!("Peer message error : {}", e).into());
                                        }
                                    },

                                    Some(PeerResponsePacket::DistributedMessage(message)) => {
                                        info!("Got Distributed message from parent {:?} : {:?}", self.peer_username, &message.kind());
                                    }

                                    None => {
                                        if backoff > 2048 {
                                            return Err(format!("Timed out listening for peer {:?} after {}ms", self.peer_username, backoff).into());
                                        }
                                        debug!("No message from peer, backing off for {}ms", backoff);
                                        tokio::time::sleep(Duration::from_millis(backoff)).await;
                                        backoff *= 2;
                                    }
                                },
                                Err(e) => {
                                    return Err(format!("Error in connection handler with {:?} : {}", self.peer_username, e).into());
                                }
                            }

                        },
                        request = self.handler_rx.recv() => {
                            if let Some(request) = request {
                                debug!("Sending request {:?} to {:?}", request, self.peer_username);
                                if let Err(err) = self.connection.write_request(request).await {
                                    error!("Handler write error, {:?}", err);
                                }
                            }
                        },
                        _ = self.shutdown.recv() => {
                            // If a shutdown signal is received, return from `run`.
                            // This will result in the task terminating.
                            break;
                        }
                    }
                }
            };
        }

        Ok(())
    }

    pub(crate) async fn connect(
        &mut self,
        database: Database,
        connection_type: ConnectionType,
    ) -> crate::Result<()> {
        self.send_peer_init().await?;
        self.connection.connection_type = Some(connection_type);
        self.listen(database).await
    }

    async fn handle_peer_message(
        &mut self,
        message: &PeerResponse,
        db: Database,
    ) -> tokio::io::Result<()> {
        match message {
            PeerResponse::SharesReply(_)
            | PeerResponse::UserInfoReply(_)
            | PeerResponse::SearchReply(_) => Ok(()),
            PeerResponse::SharesRequest => self.send_shares_reply().await,
            PeerResponse::UserInfoRequest => self.send_user_info().await,
            PeerResponse::FolderContentsRequest(_) => todo!(),
            PeerResponse::FolderContentsReply(_) => todo!(),
            PeerResponse::TransferRequest(request) => self.transfer(request, db.clone()).await,
            PeerResponse::TransferReply(_) => todo!(),
            PeerResponse::UploadPlaceholder => todo!(),
            PeerResponse::QueueDownload { .. } => todo!(),
            PeerResponse::PlaceInQueueReply(_) => todo!(),
            PeerResponse::UploadFailed(_) => todo!(),
            PeerResponse::QueueFailed(_) => todo!(),
            PeerResponse::PlaceInQueueRequest(_) => todo!(),
            PeerResponse::UploadQueueNotification => todo!(),
            PeerResponse::Unknown => {
                error!("Unknown Peer message kind : {:?}", message);
                panic!()
            }
        }
    }

    async fn handle_connection_message(
        &mut self,
        message: &PeerConnectionMessage,
        database: Database,
    ) -> Result<(), std::io::Error> {
        match message {
            PeerConnectionMessage::PierceFirewall(_pierce_firewall) => {
                debug!("Got PierceFirewall");
                Ok(())
            }
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                self.connection.connection_type = Some(connection_type.clone());
                self.peer_username = Some(username.clone());
                match self.connection.get_peer_address() {
                    Ok(peer_address) => {
                        database
                            .insert_peer(&username, peer_address)
                            .expect("Database error");

                        debug!("connected");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    #[instrument(level = "debug", skip(self))]
    async fn send_peer_init(&mut self) -> tokio::io::Result<()> {
        let token = random();

        self.connection
            .write_request(PeerRequestPacket::ConnectionMessage(
                PeerConnectionMessage::PeerInit {
                    username: "vessel".to_string(),
                    connection_type: ConnectionType::PeerToPeer,
                    token,
                },
            ))
            .await
    }

    #[instrument(level = "debug", skip(self))]
    async fn send_user_info(&mut self) -> tokio::io::Result<()> {
        // TODO : calculate correct values for total_upload, queue size and free slots
        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::UserInfoReply(
                UserInfo {
                    description: "Hello from vessel".to_string(),
                    picture: None,
                    total_upload: 0,
                    queue_size: 0,
                    slots_free: false,
                },
            )))
            .await
    }

    #[instrument(level = "debug", skip(self))]
    async fn send_shares_reply(&mut self) -> tokio::io::Result<()> {
        let shared_dirs_copy;
        {
            let shared_dirs = &crate::database::SHARED_DIRS;
            let shared_dirs = shared_dirs.lock();
            let shared_dirs = shared_dirs.unwrap();
            shared_dirs_copy = shared_dirs.clone();
        }

        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::SharesRequest))
            .await?;

        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::SharesReply(
                shared_dirs_copy,
            )))
            .await
    }

    async fn transfer(&mut self, request: &TransferRequest, db: Database) -> tokio::io::Result<()> {
        let ticket = request.ticket;
        let file_size = request.file_size.expect("Ok file size");

        db.insert_download(request)?;

        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::TransferReply(
                TransferReplyOk { ticket, file_size },
            )))
            .await?;

        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        debug!("Dropping handler");
        self.limit_connections.add_permits(1);
    }
}
