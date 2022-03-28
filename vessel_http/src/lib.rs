#[macro_use]
extern crate log;

use tokio::sync::mpsc;

use sender::VesselSender;
use soulseek_protocol::{peers::PeerRequestPacket, server::request::ServerRequest};
use vessel_database::Database;
use warp::Filter;

mod model;
mod routes;
mod sender;

pub async fn start(
    slsk_sender: mpsc::Sender<ServerRequest>,
    peer_message_sender: mpsc::Sender<(String, PeerRequestPacket)>,
    db: Database,
) {
    let sender = VesselSender::new(slsk_sender);
    let peer_sender = VesselSender::new(peer_message_sender);

    info!("Starting vessel http ...");
    warp::serve(routes::routes(db, sender, peer_sender)
        .with(warp::cors().allow_any_origin()))
        .run(([127, 0, 0, 1], 3030))
        .await;

}