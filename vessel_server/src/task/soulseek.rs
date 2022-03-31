use crate::slsk::connection::SlskConnection;
use crate::slsk::SoulseekServerListener;
use crate::state_manager::search_limit::SearchLimit;
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::server::response::ServerResponse;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub fn spawn_server_listener_task(
    http_rx: Receiver<ServerRequest>,
    sse_tx: Sender<ServerResponse>,
    peer_listener_tx: Sender<PeerConnectionRequest>,
    request_peer_connection_rx: Receiver<ServerRequest>,
    possible_parent_tx: Sender<Vec<Peer>>,
    connection: SlskConnection,
    logged_in_tx: Sender<()>,
    peer_address_tx: Sender<PeerAddress>,
    search_limit: SearchLimit,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        SoulseekServerListener {
            http_rx,
            sse_tx,
            peer_listener_tx,
            request_peer_connection_rx,
            possible_parent_tx,
            connection,
            logged_in_tx,
            peer_address_tx,
            search_limit,
        }
        .listen()
        .await;
    })
}
