use std::sync::Arc;

use rand::random;
use tokio::sync::{mpsc, Semaphore};

use crate::database::Database;
use crate::message_common::ConnectionType;
use crate::peers::connection::Connection;
use crate::peers::messages::connection::PeerConnectionMessage;
use crate::peers::messages::user_info::UserInfo;
use crate::peers::messages::PeerRequestPacket;
use crate::peers::messages::PeerResponsePacket;
use crate::peers::request::PeerRequest;
use crate::peers::response::PeerResponse;
use crate::peers::shutdown::Shutdown;

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
    pub(crate) async fn listen(&mut self, database: Database) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            if let Ok(request) = self.handler_rx.try_recv() {
                debug!("Sending request {:?} to {:?}", request, self.peer_username);
                if let Err(err) = self.connection.write_request(request).await {
                    error!("Handler write error, {:?}", err);
                }
            }

            // While reading a request frame, also listen for the shutdown
            // signal.
            let maybe_message = tokio::select! {
                res = self.connection.read_response() => res,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
            };

            match maybe_message {
                Ok(message) => match message {
                    PeerResponsePacket::ConnectionMessage(message) => {
                        info!("Got connection message : {:?}", &message);
                        self.handle_connection_message(&message, database.clone())
                            .await;
                    }
                    PeerResponsePacket::Message(message) => {
                        info!("Got message : {:?}", &message);
                        self.handle_peer_message(&message)
                            .await
                            .expect("Peer message flow error");
                    }
                    PeerResponsePacket::None => {}
                },
                Err(crate::SlskError::ConnectionResetByPeer) => {
                    debug!("connection with {:?} shutting down", self.peer_username);
                    return Ok(());
                }
                Err(_) => {
                    // Timeout
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn connect(&mut self, database: Database) -> crate::Result<()> {
        self.send_peer_init().await?;

        self.connection.connection_type = Some(ConnectionType::PeerToPeer);

        self.listen(database).await
    }

    async fn handle_peer_message(&mut self, message: &PeerResponse) -> tokio::io::Result<()> {
        match message {
            PeerResponse::SharesReply(_) | PeerResponse::UserInfoReply(_) | PeerResponse::SearchReply(_) => Ok(()),
            PeerResponse::SharesRequest => self.send_shares_reply().await,
            PeerResponse::UserInfoRequest => self.send_user_info().await,
            PeerResponse::FolderContentsRequest(_) => todo!(),
            PeerResponse::FolderContentsReply(_) => todo!(),
            PeerResponse::TransferRequest(_) => todo!(),
            PeerResponse::TransferReply(_) => todo!(),
            PeerResponse::UploadPlaceholder => todo!(),
            PeerResponse::QueueDownload(_) => todo!(),
            PeerResponse::PlaceInQueueReply(_) => todo!(),
            PeerResponse::UploadFailed(_) => todo!(),
            PeerResponse::QueueFailed(_) => todo!(),
            PeerResponse::PlaceInQueueRequest(_) => todo!(),
            PeerResponse::UploadQueueNotification => todo!(),
            PeerResponse::Unknown => {
                warn!("Unknown Peer message kind : {:#?}", message);
                Ok(())
            }
        }
    }

    async fn handle_connection_message(
        &mut self,
        message: &PeerConnectionMessage,
        database: Database,
    ) {
        match message {
            PeerConnectionMessage::PierceFirewall(_pierce_firewall) => {}
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                self.connection.connection_type = Some(connection_type.clone());
                self.peer_username = Some(username.clone());

                database
                    .insert_peer(&username, self.connection.get_peer_address())
                    .expect("Database error");
            }
        }
        debug!("connected");
    }

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
}

impl Drop for Handler {
    fn drop(&mut self) {
        debug!("HANDLER DROPPED");
        self.limit_connections.add_permits(1);
    }
}
