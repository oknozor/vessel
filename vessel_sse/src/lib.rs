#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use soulseek_protocol::{
    peers::p2p::{download::DownloadProgress, response::PeerResponse},
    server::response::ServerResponse,
};

use crate::broadcast::Broadcaster;
use tokio::sync::mpsc::Receiver;
use warp::Filter;

const MAX_SEARCH_RESULT: u32 = 500;

mod broadcast;
mod client;

pub async fn start_sse_listener(
    rx: Receiver<ServerResponse>,
    peer_rx: Receiver<PeerResponse>,
    download_progress_rx: Receiver<DownloadProgress>,
) {
    info!("Starting server sent event broadcast ...");
    let cors = warp::cors().allow_any_origin();

    let broadcaster = Broadcaster::default();

    // Dispatch soulseek messages to SSE clients
    let event_dispatcher = broadcaster.dispatch_soulseek_message(rx);

    // Dispatch peer messages to SSE clients
    let peer_event_dispatcher = broadcaster.dispatch_peer_message(peer_rx);

    // Dispatch download progress to SSE
    let download_progress = broadcaster.dispatch_download_progress(download_progress_rx);

    let users = warp::any().map(move || broadcaster.clone());

    let sse_events = warp::path!("events")
        .and(warp::get())
        .and(users)
        .map(|broadcaster: Broadcaster| {
            // Stream incoming server event to sse
            let stream = broadcaster.on_sse_event_received();
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        })
        .with(cors)
        .with(warp::log("api"));

    let _ = join!(
        warp::serve(sse_events).run(([127, 0, 0, 1], 3031)),
        event_dispatcher,
        peer_event_dispatcher,
        download_progress,
    );
}
