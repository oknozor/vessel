use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::database::entities::PeerEntity;
use crate::message_common::ConnectionType;
use crate::peers::messages::PeerRequestPacket;
use crate::SlskError;
use tokio::sync::mpsc::Receiver;

pub type PeerAddress = String;

#[derive(Debug, Clone)]
pub struct SenderPool {
    inner: Arc<Mutex<HashMap<PeerAddress, PeerConnectionState>>>,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionState {
    pub username: Option<String>,
    pub token: Option<u32>,
    pub channel: Option<mpsc::Sender<PeerRequestPacket>>,
    pub conn_state: ConnectionState,
}

impl PeerConnectionState {
    fn set_conn_state(&mut self, state: ConnectionState) {
        self.conn_state = state;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    // Successful connection
    Established(ConnectionType),
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
    pub(crate) fn to_connection_type(&self) -> ConnectionType {
        match self {
            ConnectionState::IndirectConnectionPending(conn_type)
            | ConnectionState::ExpectingIndirectConnection(conn_type)
            | ConnectionState::Established(conn_type) => *conn_type,
            _ => ConnectionType::HandShake,
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
    pub async fn set_connection_lost(&mut self, address: String) {
        let mut channels = self.inner.lock().await;
        let state = channels.get_mut(&address);

        if let Some(state) = state {
            state.conn_state = ConnectionState::Lost
        }
    }
    pub async fn new_incomming_indirect_connection_pending(
        &mut self,
        peer: &PeerEntity,
        token: u32,
        conn_type: ConnectionType,
    ) {
        let mut channels = self.inner.lock().await;
        let state = PeerConnectionState {
            username: Some(peer.username.clone()),
            token: Some(token),
            channel: None,
            conn_state: ConnectionState::IndirectConnectionPending(conn_type),
        };

        info!(
            "Indirect connection pending for address {:?} with state : {:?}",
            peer, state
        );
        channels.insert(peer.get_address(), state);
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
            token: None,
            channel: Some(tx),
            conn_state: ConnectionState::PeerInitPending,
        };

        channels.insert(address, state);

        rx
    }

    pub async fn new_peer_connection(
        &mut self,
        address: &str,
    ) -> (Receiver<PeerRequestPacket>, PeerConnectionState) {
        let (tx, rx) = mpsc::channel(100);
        let mut channels = self.inner.lock().await;

        let state = PeerConnectionState {
            username: None,
            token: None,
            channel: Some(tx),
            conn_state: ConnectionState::PeerInitPending,
        };

        let state_copy = state.clone();
        channels.insert(address.to_string(), state);
        (rx, state_copy)
    }

    pub async fn send_message(
        &mut self,
        address: String,
        message: PeerRequestPacket,
    ) -> crate::Result<()> {
        let sender;
        {
            let channel_pool = self.inner.lock().await;
            let channel = channel_pool.get(&address);

            sender = if let Some(state) = channel {
                info!("Found channel for peer {:?}@{}", state.username, address);

                if let ConnectionState::Lost = state.conn_state {
                    return Err(SlskError::PeerConnectionLost);
                }

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
        Ok(())
    }

    pub async fn get_parent_count(&self) -> usize {
        let channels = self.inner.lock().await;
        channels
            .values()
            .filter(|state| match state.conn_state {
                ConnectionState::Established(ConnectionType::DistributedNetwork)
                | ConnectionState::IndirectConnectionPending(ConnectionType::DistributedNetwork)
                | ConnectionState::ExpectingIndirectConnection(
                    ConnectionType::DistributedNetwork,
                ) => true,
                _ => false,
            })
            .count()
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
                info!(
                    "Successfully upgraded connection {:?} state to {:?}",
                    address, state
                );
                channel.set_conn_state(state)
            }
        }

        Ok(())
    }

    pub async fn update_or_create(
        &mut self,
        address: &str,
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
