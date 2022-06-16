use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::download::DownloadProgress;
use soulseek_protocol::peers::PeerRequestPacket;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct SenderPool {
    ok_connections: Arc<Mutex<HashMap<u32, PeerConnectionState>>>,
    pending_connections: Arc<Mutex<Vec<PeerConnectionState>>>,
    download_progress_sender: Sender<DownloadProgress>,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionState {
    pub username: String,
    pub token: u32,
    pub channel: Option<mpsc::Sender<PeerRequestPacket>>,
    pub conn_type: ConnectionType,
}

impl SenderPool {
    pub fn new(download_sender_progress_sender: Sender<DownloadProgress>) -> Self {
        SenderPool {
            ok_connections: Arc::new(Mutex::new(HashMap::default())),
            pending_connections: Arc::new(Mutex::new(Default::default())),
            download_progress_sender: download_sender_progress_sender,
        }
    }
}

impl SenderPool {
    pub(crate) fn find_by_username_and_connection_type(
        &mut self,
        username: &str,
        conn_type: ConnectionType,
    ) -> Option<PeerConnectionState> {
        let channels = self.ok_connections.lock().unwrap();

        channels
            .iter()
            .find(|(_, state)| state.username == username)
            .filter(|(_, state)| state.conn_type == conn_type)
            .map(|(_, state)| state.clone())
    }

    pub fn get_parent_count(&self) -> usize {
        let channels = self.ok_connections.lock().unwrap();
        channels
            .values()
            .map(|state| state.conn_type)
            .filter(|conn_type| *conn_type == ConnectionType::DistributedNetwork)
            .count()
    }

    pub fn on_peer_init_received(
        &mut self,
        username: &str,
        conn_type: ConnectionType,
        token: u32,
        sender: Sender<PeerRequestPacket>,
    ) {
        let mut connection_states = self.ok_connections.lock().unwrap();
        let state = PeerConnectionState {
            username: username.to_string(),
            token,
            channel: Some(sender),
            conn_type,
        };
        debug!(
            "Inserting connection state on PeerInit received token={}, state={:?}",
            token, &state
        );
        connection_states.insert(token, state);
    }

    pub fn insert_indirect_connection_expected(
        &mut self,
        username: &str,
        conn_type: ConnectionType,
        token: u32,
    ) {
        let mut channels = self.pending_connections.lock().unwrap();

        let state = PeerConnectionState {
            username: username.to_string(),
            token,
            channel: None,
            conn_type,
        };

        debug!("Adding PierceFirewall expected state {:?}", state);
        channels.push(state);
    }

    pub fn get(&self, token: u32) -> Option<PeerConnectionState> {
        let channels = self.ok_connections.lock().unwrap();
        channels.get(&token).cloned()
    }

    pub fn ready(
        &self,
        token: u32,
        tx: Sender<PeerRequestPacket>,
    ) -> eyre::Result<PeerConnectionState> {
        let mut pending_connections = self.pending_connections.lock().unwrap();

        let (idx, ready_state) = pending_connections
            .iter()
            .enumerate()
            .find(|(_, state)| state.token == token)
            .ok_or_else(|| eyre!("Pending connection state not found token={}", token))?;

        let mut ready_state = ready_state.clone();
        ready_state.channel = Some(tx);

        // Clean up pending connection
        let _ = pending_connections.remove(idx);

        let mut ok_connections = self.ok_connections.lock().unwrap();
        ok_connections.insert(token, ready_state.clone());

        info!(
            "Connection  username={}, token={} is alive",
            ready_state.username, token
        );
        Ok(ready_state)
    }

    pub fn remove_channel(&mut self, token: u32) -> eyre::Result<()> {
        let mut channels = self.ok_connections.lock().unwrap();

        channels.remove(&token).map(|_| ()).ok_or_else(|| {
            eyre!(
                "Failed to drop channel for connection with token {}, channel not found",
                token
            )
        })
    }

    pub fn get_progress_sender(&self) -> Sender<DownloadProgress> {
        self.download_progress_sender.clone()
    }
}
