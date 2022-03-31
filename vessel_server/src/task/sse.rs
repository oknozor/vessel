use tokio::sync::mpsc::Receiver;
use soulseek_protocol::peers::p2p::download::DownloadProgress;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::response::ServerResponse;
use tokio::task::JoinHandle;

pub fn spawn_sse_server(
    sse_rx: Receiver<ServerResponse>,
    sse_peer_rx: Receiver<PeerResponse>,
    download_progress_rx: Receiver<DownloadProgress>,
) -> JoinHandle<()> {
    tokio::spawn(async {
        vessel_sse::start_sse_listener(sse_rx, sse_peer_rx, download_progress_rx).await;
    })
}
