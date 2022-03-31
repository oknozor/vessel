use crate::peers;
use crate::peers::peer_listener::{PeerListenerReceivers, PeerListenerSenders};
use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use vessel_database::Database;

pub fn spawn_peer_listener(
    senders: PeerListenerSenders,
    receivers: PeerListenerReceivers,
    mut logged_in_rx: Receiver<()>,
    listener: TcpListener,
    database: Database,
    channels: SenderPool,
    search_limit: SearchLimit,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while logged_in_rx.recv().await.is_none() {
            // Wait for soulseek login
        }

        peers::peer_listener::run(
            listener,
            signal::ctrl_c(),
            senders,
            receivers,
            database,
            channels.clone(),
            search_limit,
        )
        .await
        .expect("Unable to run peer listener");
    })
}
