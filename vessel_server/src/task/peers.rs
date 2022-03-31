use crate::state_manager::channel_manager::SenderPool;
use crate::state_manager::search_limit::SearchLimit;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};
use soulseek_protocol::server::request::ServerRequest;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use vessel_database::Database;

pub fn spawn_peer_listener(
    sse_tx: Sender<PeerResponse>,
    server_request_tx: Sender<ServerRequest>,
    peer_listener_rx: Receiver<PeerConnectionRequest>,
    possible_parent_rx: Receiver<Vec<Peer>>,
    peer_request_rx: Receiver<(String, PeerRequestPacket)>,
    peer_address_rx: Receiver<PeerAddress>,
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

        crate::peer_connection_manager::run(
            listener,
            signal::ctrl_c(),
            sse_tx,
            server_request_tx,
            peer_listener_rx,
            possible_parent_rx,
            peer_request_rx,
            peer_address_rx,
            database,
            channels.clone(),
            search_limit,
        )
        .await
        .expect("Unable to run peer listener");
    })
}
