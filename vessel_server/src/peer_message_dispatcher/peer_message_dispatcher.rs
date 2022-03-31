use std::collections::HashMap;

use eyre::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use soulseek_protocol::{
    message_common::ConnectionType,
    peers::{p2p::response::PeerResponse, PeerRequestPacket},
    server::{peer::PeerAddress, request::ServerRequest},
};
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;

use crate::peers::peer_listener::{connect_to_peer_with_fallback, ShutdownHelper};
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;

pub struct Dispatcher {
    // Receive connection state updates from peer handlers,
    // once the handshake has completed we can start to feed the handler with queued messages.
    pub(crate) ready_rx: Receiver<u32>,

    // Receive peer request from the HTTP server, if the connection is ready we can dispatch the messages
    // to the peer directly, otherwise the message is queued until the 'ready' signal is received for this peer.
    pub(crate) queue_rx: Receiver<(String, PeerRequestPacket)>,

    // When trying to initiate connection to a peer we ask the soulseek server for the address of a peer.
    // Upon receiving the address we can initiate the connection.
    pub(crate) peer_address_rx: Receiver<PeerAddress>,

    // Hold peer channels states and connection type, their are three kinds of connections :
    // - pending connection (i.e we want to connect to a peer but the address is unknown yet)
    // - active connection : we are connected already and we can dispatch HTTP message to the peer
    // - download connection : these requires a special mpsc sender to dispatch download progress to the SSE server
    pub(crate) channels: SenderPool,

    // Hold the sender to the SSE mpsc channel, each time a new connection is spawned we pass a copy
    // po the peer response are dispatched directly via SSE
    pub(crate) sse_tx: Sender<PeerResponse>,

    // Pass a copy of the ready sender to the peer connection so it can notify the connection handshake
    // has completed
    pub(crate) ready_tx: Sender<u32>,

    // Pass a copy to the peer connection to write download state
    pub(crate) db: Database,

    // Whenever a direct connection fails send a PeerConnectionRequest to the soulseek server
    pub(crate) server_request_tx: Sender<ServerRequest>,

    // Save message sent to peer if the connection is not ready yet
    pub(crate) message_queue: HashMap<String, Vec<PeerRequestPacket>>,

    // Whenever a search reply is issued by a peer decide to dispatch it via SSE or not if the limit
    // is reached.
    pub(crate) search_limit: SearchLimit,

    // handle graceful shutdown of peer connections
    pub(crate) shutdown_helper: ShutdownHelper,
}

impl Dispatcher {
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                request = self.queue_rx.recv() => {
                    if let Some((username, request)) = request {
                        debug!("Got peer message from HTTP server {}, {:?}", username, request);
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
                        self.on_peer_address_received(peer, self.search_limit.clone()).await;
                    }

                }
            }
        }
    }

    async fn on_peer_address_received(&mut self, peer: PeerEntity, search_limit: SearchLimit) {
        self.db.insert(&peer).unwrap();
        if let Some(queue) = self.message_queue.get(&peer.username) {
            if let Some(msg) = queue.last() {
                let conn_type = ConnectionType::from(msg);
                self.initiate_connection(conn_type, peer, search_limit)
                    .await;
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
        debug!(target:"vessel_http", "Pushing http message to message queue, peer={}, request={:?}", username, request);
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
            None => match self.db.get_by_key::<PeerEntity>(username) {
                Some(peer) => {
                    self.initiate_connection(
                        ConnectionType::PeerToPeer,
                        peer,
                        self.search_limit.clone(),
                    )
                    .await
                }
                _ => {
                    debug!("Requesting peer address");
                    self.server_request_tx
                        .send(ServerRequest::GetPeerAddress(username.to_string()))
                        .await
                        .expect("Server send error")
                }
            },
        }
    }

    async fn initiate_connection(
        &mut self,
        conn_type: ConnectionType,
        peer: PeerEntity,
        search_limit: SearchLimit,
    ) {
        let sender = self.server_request_tx.clone();
        let sse_tx = self.sse_tx.clone();
        let channels = self.channels.clone();
        let ready_tx = self.ready_tx.clone();
        let helpers = self.shutdown_helper.clone();
        let db = self.db.clone();

        let connection_result = connect_to_peer_with_fallback(
            sender,
            sse_tx,
            ready_tx,
            channels,
            helpers,
            db,
            &peer,
            conn_type,
            search_limit,
        )
        .await;

        if let Err(err) = connection_result {
            error!("An errored occurred during connection : {:?}", err);
        }
    }

    async fn push_to_queue(&mut self, username: String, request: PeerRequestPacket) {
        debug!(
            "Pushing peer request from {} to queue to message queue : {:?}",
            username, request
        );
        match self.message_queue.get_mut(&username) {
            Some(queue) => queue.push(request),
            None => {
                self.message_queue.insert(username, vec![request]);
            }
        }
    }
}
