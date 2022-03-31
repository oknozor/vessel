use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::request::ServerRequest;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use vessel_database::Database;

pub fn spawn_http_listener(
    http_tx: Sender<ServerRequest>,
    peer_message_dispatcher_tx: Sender<(String, PeerRequestPacket)>,
    database: Database,
) -> JoinHandle<()> {
    tokio::spawn(
        async move { vessel_http::start(http_tx, peer_message_dispatcher_tx, database).await },
    )
}
