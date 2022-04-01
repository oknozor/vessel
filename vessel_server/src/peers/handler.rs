use std::{net::SocketAddr, sync::Arc};

use eyre::Result;
use rand::random;
use tokio::sync::{
    mpsc,
    mpsc::{channel, Receiver},
    Semaphore,
};

use soulseek_protocol::peers::p2p::transfer::{QueueUpload, TransferReply};
use soulseek_protocol::{
    message_common::ConnectionType,
    peers::{
        connection::PeerConnectionMessage,
        distributed::DistributedMessage,
        p2p::{
            request::PeerRequest,
            response::PeerResponse,
            transfer::{TransferReply::TransferReplyOk, TransferRequest},
            user_info::UserInfo,
        },
        PeerRequestPacket,
    },
};
use vessel_database::entity::download::DownloadEntity;
use vessel_database::entity::upload::UploadEntity;
use vessel_database::Database;

use crate::peers::{connection::PeerConnection, shutdown::Shutdown};
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;

pub struct PeerHandler {
    pub peer_username: Option<String>,
    pub(crate) connection: PeerConnection,
    pub(crate) sse_tx: mpsc::Sender<PeerResponse>,
    pub(crate) ready_tx: mpsc::Sender<u32>,
    pub(crate) shutdown: Shutdown,
    pub(crate) limit_connections: Arc<Semaphore>,
    pub(crate) limit_search: SearchLimit,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
    pub(crate) connection_states: SenderPool,
    pub(crate) db: Database,
    // We store this to drop the channel state because calling get_address on a dropped connection
    // Can produce Err NotConnected
    pub(crate) address: SocketAddr,
}

/// Spawn a new peer handler attempting a direct connection
pub(crate) async fn connect_direct(
    mut handler: PeerHandler,
    conn_type: ConnectionType,
) -> Result<()> {
    tokio::spawn(async move {
        match handler.init_connection_outgoing(conn_type).await {
            Ok(_) => info!(
                "Direct outgoing established with peer {:?} token={:?}",
                handler.peer_username, handler.connection.token
            ),
            Err(e) => error!("Error during direct peer connection, cause = {}", e),
        }
    });

    Ok(())
}

pub(crate) async fn pierce_firewall(mut handler: PeerHandler, token: u32) -> Result<()> {
    tokio::spawn(async move {
        match handler.pierce_firewall(token).await {
            Ok(()) => info!(
                "Connection with {:?} ended successfully",
                handler.peer_username
            ),
            Err(e) => error!("Error during direct peer connection, cause = {}", e),
        }
    });
    Ok(())
}

impl PeerHandler {
    pub(crate) async fn listen(&mut self, handler_rx: Receiver<PeerRequestPacket>) -> Result<()> {
        debug!("{:?}", self.connection_type());

        match self.connection_type() {
            ConnectionType::PeerToPeer => {
                info!(
                    "Peer to Peer connection established with peer {:?}, token={:?}",
                    self.peer_username, self.connection.token
                );
                self.listen_p2p(handler_rx).await?;
            }
            ConnectionType::FileTransfer => {
                let progress_sender = self.connection_states.get_progress_sender();
                let user_name = self.peer_username.as_ref().unwrap().clone();
                self.connection
                    .download(&self.db, progress_sender, user_name)
                    .await?;
            }
            ConnectionType::DistributedNetwork => {
                self.listen_distributed().await?;
            }
            ConnectionType::HandShake => {
                unreachable!("Connection type should be known at this point")
            }
        }

        Ok(())
    }

    pub(crate) async fn listen_p2p(
        &mut self,
        mut handler_rx: Receiver<PeerRequestPacket>,
    ) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                response = self.connection.read_message::<PeerResponse>() =>  {
                    match response {
                        Ok(message) => {
                            // When receiving a SearchReply we want to close the connection asap
                            if let PeerResponse::SearchReply(ref search_reply) = message {
                               if self.limit_search.get() > 0 && search_reply.ticket == self.limit_search.ticket()  {
                                    self.limit_search.decrement();

                                    return self.sse_tx.send(message)
                                        .await
                                        .map_err(|err| eyre!(err));
                                }

                            } else {
                               // Don't flood the log with search replies
                               info!("[token={:?}] - Got Peer message {:?}", self.connection.token, message);

                            if let Err(e) = self.handle_peer_message(&message).await {
                                return Err(eyre!("Error handling peer message : {}", e));
                            }

                            self.sse_tx.send(message)
                                .await
                                .map_err(|err| eyre!(err))?;
                            }
                        }
                        Err(e) => {
                            return Err(eyre!("Error in connection handler with {:?} : {}", self.peer_username, e));
                        }
                    }

                },
                request = handler_rx.recv() => if let Some(request) = request  {
                    debug!("Sending request to {:?}", self.peer_username);
                    if let Err(err) = self.connection.write_request(request).await {
                        error!("Handler write error, {:?}", err);
                    }
                },
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn listen_distributed(&mut self) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                response = self.connection.read_message::<DistributedMessage>() =>  {
                    match response {
                        Ok(message) => trace!("Got distributed message {:?}", message),
                        Err(e) => {
                            return Err(eyre!("Error in connection handler with {:?} : {}", self.peer_username, e));
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

        Ok(())
    }

    pub(crate) async fn init_connection_outgoing(
        &mut self,
        conn_type: ConnectionType,
    ) -> Result<()> {
        let token = random();
        debug!(
            "Initiate direct connection with peer init, token = {}",
            token
        );
        self.send_peer_init(token, conn_type).await?;
        let (tx, rx) = channel(32);
        let username = &self.peer_username.as_ref().unwrap();
        let connection_type = self.connection_type();
        self.connection_states
            .peer_init(username, connection_type, token, tx);
        self.connection.token = Some(token);
        self.connection.connection_type = conn_type;
        self.ready_tx.send(token).await?;
        self.listen(rx).await
    }

    pub(crate) async fn pierce_firewall(&mut self, token: u32) -> Result<()> {
        debug!(
            "Initiate connection with pierce firewall, token = {}",
            token
        );
        self.send_pierce_firewall(token).await?;
        let (tx, rx) = channel(32);
        let state = self.connection_states.ready(token, tx)?;
        self.connection.token = Some(token);
        self.peer_username = Some(state.username);
        self.connection.connection_type = state.conn_type;

        self.ready_tx.send(token).await?;

        self.listen(rx).await
    }

    pub(crate) async fn wait_for_connection_handshake(&mut self) -> Result<()> {
        let message = self
            .connection
            .read_message::<PeerConnectionMessage>()
            .await?;
        let rx = self.handle_connection_message(&message).await?;
        self.listen(rx).await
    }

    async fn handle_peer_message(&mut self, message: &PeerResponse) -> tokio::io::Result<()> {
        info!("Got peer message {:?}", message);
        match message {
            PeerResponse::SharesReply(_)
            | PeerResponse::UserInfoReply(_)
            | PeerResponse::SearchReply(_) => Ok(()),
            PeerResponse::SharesRequest => self.send_shares_reply().await,
            PeerResponse::UserInfoRequest => self.send_user_info().await,
            PeerResponse::FolderContentsRequest(_) => todo!(),
            PeerResponse::FolderContentsReply(_) => todo!(),
            PeerResponse::TransferRequest(request) => self.transfer(request).await,
            PeerResponse::TransferReply(transfer_reply) => {
                self.transfer_reply(transfer_reply).await
            }
            PeerResponse::UploadPlaceholder => todo!(),
            PeerResponse::QueueUpload(queue_upload) => self.queue_upload(queue_upload).await,
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
    ) -> Result<Receiver<PeerRequestPacket>> {
        debug!("Incoming connection handshake message : {:?}", message);
        let (tx, rx) = channel(32);
        let token = match message {
            PeerConnectionMessage::PierceFirewall(token) => {
                let state = self.connection_states.ready(*token, tx)?;
                self.connection.connection_type = state.conn_type;
                self.peer_username = Some(state.username);
                self.connection.token = Some(*token);
                *token
            }
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                self.connection.connection_type = *connection_type;
                self.peer_username = Some(username.clone());

                // Token = 0 indicate an incoming search reply
                if *token != 0 {
                    self.connection_states
                        .peer_init(username, *connection_type, *token, tx);
                };

                self.connection.token = Some(*token);
                *token
            }
        };

        // Unless this is a search reply connection, notify this connection is ready to receive
        // requests from the dispatcher
        if token != 0 {
            self.ready_tx.send(token).await.map_err(|e| {
                eyre!(
                    "Error sending ready state in connection with token {}, cause = {}",
                    token,
                    e
                )
            })?;
        }

        Ok(rx)
    }

    async fn send_peer_init(
        &mut self,
        token: u32,
        conn_type: ConnectionType,
    ) -> tokio::io::Result<()> {
        self.connection
            .write_request(PeerRequestPacket::ConnectionMessage(
                PeerConnectionMessage::PeerInit {
                    username: "vessel".to_string(),
                    connection_type: conn_type,
                    token,
                },
            ))
            .await
    }

    async fn send_pierce_firewall(&mut self, token: u32) -> tokio::io::Result<()> {
        self.connection
            .write_request(PeerRequestPacket::ConnectionMessage(
                PeerConnectionMessage::PierceFirewall(token),
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
            let shared_dirs = &vessel_database::SHARED_DIRS;
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

    async fn transfer(&mut self, request: &TransferRequest) -> tokio::io::Result<()> {
        let ticket = request.ticket;
        let file_size = request.file_size.expect("Ok file size");

        let username = self
            .peer_username
            .clone()
            .expect("Username should be known when initiating a transfer");

        let download_entity = DownloadEntity::from((username, request));
        self.db.insert(&download_entity)?;

        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::TransferReply(
                TransferReplyOk { ticket, file_size },
            )))
            .await?;

        Ok(())
    }

    async fn transfer_reply(&mut self, transfer_reply: &TransferReply) -> tokio::io::Result<()> {
        info!("Transfer reply : {:?}", transfer_reply);
        Ok(())
    }

    async fn queue_upload(&mut self, queue_upload: &QueueUpload) -> tokio::io::Result<()> {
        info!("{:?}", queue_upload);
        let file_name = queue_upload.file_name.clone();
        let user_name = self.peer_username.as_ref().unwrap().clone();
        // FIXME: NO MORE RANDOM TICKET
        let ticket = random();
        let upload = UploadEntity::new(file_name.clone(), user_name, ticket);
        self.db.insert(&upload)?;
        self.connection
            .write_request(PeerRequestPacket::Message(PeerRequest::TransferRequest(
                TransferRequest {
                    direction: 1,
                    ticket,
                    file_name,
                    file_size: None,
                },
            )))
            .await?;
        Ok(())
    }

    fn connection_type(&self) -> ConnectionType {
        self.connection.connection_type
    }
}

impl Drop for PeerHandler {
    fn drop(&mut self) {
        // Release a connection permit
        self.limit_connections.add_permits(1);
        debug!(
            "Peer handler [{:?}, {:?}] dropped, connection permit released",
            self.peer_username, self.connection.token
        );

        // If we have a token we need to clean up channel state
        // Otherwise connection was never ready and does not have a ready channel
        if let Some(token) = self.connection.token {
            if token != 0 {
                if let Err(e) = self.connection_states.remove_channel(token) {
                    panic!(
                        "Error dropping channel with address {}, {}",
                        self.address, e
                    )
                };
            }
        }
    }
}
