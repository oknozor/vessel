#[macro_use]
extern crate log;

use tokio::sync::mpsc;
use warp::Filter;

use sender::VesselSender;
use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::messages::request::ServerRequest;

use crate::routes::{chat_routes, peers_routes, rooms_routes, search_routes, users_routes};

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

    let routes = rooms_routes(sender.clone())
        .or(peers_routes(peer_sender.clone()))
        .or(chat_routes(sender.clone()))
        .or(users_routes(sender.clone(), db))
        .or(search_routes(sender.clone()))
        .or(rooms_routes(sender.clone()));

    info!("Starting vessel http ...");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
