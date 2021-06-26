use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::peers::channels::SenderPool;
use crate::peers::listener::{connect_to_peer_with_fallback, ShutdownHelper};
use eyre::Result;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::peer::PeerAddress;
use soulseek_protocol::server::request::ServerRequest;
use vessel_database::{entities::PeerEntity, Database};

pub struct Dispatcher {
    // Receive connection state updates from peer handler
    pub(crate) ready_rx: Receiver<u32>,

    // Receive peer request from the HTTP server
    pub(crate) queue_rx: Receiver<(String, PeerRequestPacket)>,
    // Once the server receive a peer address response it send it back
    // So we can start to pop message from queue
    pub(crate) peer_address_rx: Receiver<PeerAddress>,

    // Hold peer channels and connection type
    pub(crate) channels: SenderPool,

    // Used to initiate new peer connections
    pub(crate) db: Database,
    pub(crate) shutdown_helper: ShutdownHelper,
    pub(crate) sse_tx: Sender<PeerResponse>,
    pub(crate) ready_tx: Sender<u32>,
    pub(crate) server_request_tx: Sender<ServerRequest>,

    // Save message sent to peer if the connection is not ready yet
    pub(crate) message_queue: HashMap<String, Vec<PeerRequestPacket>>,
}

impl Dispatcher {
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                request = self.queue_rx.recv() => {
                    if let Some((username, request)) = request {
                        self.on_peer_request(&username, request).await;
                    }
                }
                token = self.ready_rx.recv() => {
                    if let Some(token) = token {
                        self.on_peer_ready(token).await.unwrap();
                    }
                }
                peer = self.peer_address_rx.recv() => {
                    if let Some(peer) = peer {
                        let peer = PeerEntity::from(peer);
                        self.on_peer_address_received(peer).await;
                    }

                }
            }
        }
    }

    async fn on_peer_address_received(&mut self, peer: PeerEntity) {
        self.db.insert_peer(&peer).unwrap();
        if let Some(queue) = self.message_queue.get(&peer.username) {
            if let Some(msg) = queue.last() {
                let conn_type = ConnectionType::from(msg);
                self.initiate_connection(conn_type, peer).await;
            }
        }
    }

    async fn on_peer_ready(&mut self, token: u32) -> Result<()> {
        let connection_state = self.channels.get(token);

        if let Some(connection_state) = connection_state {
            debug!(
                "Got peer ready with connection state : {:?}",
                connection_state
            );
            let username = connection_state.username;
            let sender = connection_state.channel.expect("Channel should be known");

            if let Some(queue) = self.message_queue.get_mut(&username) {
                debug!(
                    "Sending queued messages {:?}  peer={:?}, token={}",
                    queue, username, token
                );
                while let Some(msg) = queue.pop() {
                    sender.send(msg).await?;
                }
            }
        }

        Ok(())
    }

    async fn on_peer_request(&mut self, username: &str, request: PeerRequestPacket) {
        let conn_type = ConnectionType::from(&request);
        self.push_to_queue(username.to_string(), request).await;
        let peer_conn_state = self
            .channels
            .find_by_username_and_connection_type(username, conn_type);

        match peer_conn_state {
            // Connection is established already, we can send the message right away
            Some(state) => {
                if let Some(queue) = self.message_queue.get_mut(username) {
                    let request = queue.pop().unwrap();
                    state
                        .channel
                        .expect("Peer channel should be known at this point")
                        .send(request)
                        .await
                        .expect("Send error");
                }
            }
            None => match self.db.get_peer(username) {
                Some(peer) => {
                    self.initiate_connection(ConnectionType::PeerToPeer, peer)
                        .await
                }
                None => self
                    .server_request_tx
                    .send(ServerRequest::GetPeerAddress(username.to_string()))
                    .await
                    .expect("Server send error"),
            },
        }
    }

    async fn initiate_connection(&mut self, conn_type: ConnectionType, peer: PeerEntity) {
        let sender = self.server_request_tx.clone();
        let sse_tx = self.sse_tx.clone();
        let channels = self.channels.clone();
        let ready_tx = self.ready_tx.clone();
        let helpers = self.shutdown_helper.clone();
        let db = self.db.clone();

        let connection_result = connect_to_peer_with_fallback(
            sender, sse_tx, ready_tx, channels, helpers, db, &peer, conn_type,
        )
        .await;

        if let Err(err) = connection_result {
            error!("An errored occurred during connection : {:?}", err);
        }
    }

    async fn push_to_queue(&mut self, username: String, request: PeerRequestPacket) {
        println!("Pushing to queue  ({}, {:?})", username, request);
        match self.message_queue.get_mut(&username) {
            Some(queue) => queue.push(request),
            None => {
                self.message_queue.insert(username, vec![request]);
            }
        }
    }
}
