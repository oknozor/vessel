use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::message_common::ConnectionType;
use crate::peers::messages::PeerRequestPacket;
use tokio::sync::mpsc::Receiver;

pub type PeerAddress = String;

#[derive(Debug, Clone)]
pub struct SenderPool {
    inner: Arc<Mutex<HashMap<PeerAddress, PeerConnectionState>>>,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionState {
    pub username: Option<String>,
    pub is_parent: bool,
    pub token: Option<u32>,
    pub channel: Option<mpsc::Sender<PeerRequestPacket>>,
    pub conn_state: ConnectionState,
}

impl PeerConnectionState {
    fn set_conn_state(&mut self, state: ConnectionState) {
        self.conn_state = state;
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    // Successful connection
    Established,
    // We still need to send or receive a PeerInit message to upgrade this connection
    PeerInitPending,
    // We received a peer ConnectToPeer request from soulseek we need to attemp a connection
    // and send a PierceFireWall message
    IndirectConnectionPending(ConnectionType),
    // We sent a ConnectToPeer request to soulseek an are now expecting the peer to connect
    // and send a PierceFireWall message
    ExpectingIndirectConnection(ConnectionType),
    // PierceFirewall failed, this connection is doomed
    Failed,
    // We used to have a connection to this peer
    Lost,
}

impl ConnectionState {
    pub(crate) fn to_connection_type(&self) -> Option<ConnectionType> {
        match self {
            ConnectionState::IndirectConnectionPending(conn_type)
            | ConnectionState::ExpectingIndirectConnection(conn_type) => Some(conn_type.clone()),
            _ => None,
        }
    }
}

impl Default for SenderPool {
    fn default() -> Self {
        SenderPool {
            inner: Arc::new(Mutex::new(HashMap::default())),
        }
    }
}

impl SenderPool {
    pub async fn new_incomming_indirect_parent_connection_pending(
        &mut self,
        username: String,
        address: String,
        token: u32,
    ) {
        let mut channels = self.inner.lock().await;
        let state = PeerConnectionState {
            username: Some(username),
            is_parent: true,
            token: Some(token),
            channel: None,
            conn_state: ConnectionState::IndirectConnectionPending(
                ConnectionType::DistributedNetwork,
            ),
        };

        info!(
            "Indirect connection pending for address {:?} with state : {:?}",
            address, state
        );
        channels.insert(address, state);
    }

    pub async fn new_named_peer_connection(
        &mut self,
        username: String,
        address: String,
    ) -> Receiver<PeerRequestPacket> {
        let (tx, rx) = mpsc::channel(100);
        let mut channels = self.inner.lock().await;

        let state = PeerConnectionState {
            username: Some(username),
            is_parent: false,
            token: None,
            channel: Some(tx),
            conn_state: ConnectionState::PeerInitPending,
        };

        channels.insert(address, state);

        rx
    }

    pub async fn new_peer_connection(
        &mut self,
        address: String,
    ) -> (Receiver<PeerRequestPacket>, PeerConnectionState) {
        let (tx, rx) = mpsc::channel(100);
        let mut channels = self.inner.lock().await;

        let state = PeerConnectionState {
            username: None,
            is_parent: true,
            token: None,
            channel: Some(tx),
            conn_state: ConnectionState::PeerInitPending,
        };

        let state_copy = state.clone();
        channels.insert(address, state);
        (rx, state_copy)
    }

    pub async fn send_message(&mut self, address: String, message: PeerRequestPacket) {
        let sender;
        {
            let channel_pool = self.inner.lock().await;
            let channel = channel_pool.get(&address);

            sender = if let Some(state) = channel {
                info!("Found channel for peer {:?}@{}", state.username, address);
                Some(state.channel.clone())
            } else {
                error!("Channel Not found for peer {}", address);
                None
            };

            // We have a sender, let's send the message
            if let Some(sender) = sender.expect("Handler channel should be known at this point") {
                debug!("Sending message to peer handler {:?}", message);
                sender.send(message).await.expect("Send error");
            }
        }
    }

    pub async fn get_parent_count(&self) -> usize {
        let channels = self.inner.lock().await;
        channels.values().filter(|state| state.is_parent).count()
    }

    pub async fn set_connection_state(
        &mut self,
        address: String,
        state: ConnectionState,
    ) -> crate::Result<()> {
        let address = address.to_string();
        let mut channels = self.inner.lock().await;
        match channels.get_mut(&address.to_string()) {
            None => error!("Channel not found for address {:?}", address),
            Some(channel) => {
                info!("Successfully upgraded connection state to {:?}", state);
                channel.set_conn_state(state)
            }
        }

        Ok(())
    }

    pub async fn update_or_create(
        &mut self,
        address: String,
    ) -> (Receiver<PeerRequestPacket>, PeerConnectionState) {
        match self.update_state(&address).await {
            None => self.new_peer_connection(address).await,
            Some((rx, state)) => (rx, state),
        }
    }

    async fn update_state(
        &mut self,
        address: &str,
    ) -> Option<(Receiver<PeerRequestPacket>, PeerConnectionState)> {
        let mut channels = self.inner.lock().await;

        match channels.get_mut(address) {
            Some(state) => {
                let (tx, rx) = mpsc::channel(100);
                state.channel = Some(tx);
                info!(
                    "Existing channel state found for address {:?} : {:?}",
                    address, state
                );
                Some((rx, state.clone()))
            }
            None => None,
        }
    }
}
