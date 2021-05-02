use warp::Filter;

use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::messages::request::ServerRequest;

use crate::sender::VesselSender;

pub(crate) mod chat;
pub(crate) mod peers;
pub(crate) mod rooms;
pub(crate) mod search;
pub(crate) mod users;

pub fn routes(
    db: Database,
    sender: VesselSender<ServerRequest>,
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    rooms_routes(sender.clone())
        .or(peers_routes(peer_sender.clone()))
        .or(chat_routes(sender.clone()))
        .or(users_routes(sender.clone(), db))
        .or(search_routes(sender.clone()))
        .or(rooms_routes(sender.clone()))
}

pub(crate) fn rooms_routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    rooms::join_room(sender.clone()).or(rooms::send_chat_message(sender))
}

pub(crate) fn chat_routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    chat::start_public_chat(sender.clone()).or(chat::stop_public_chat(sender))
}

pub(crate) fn peers_routes(
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    peers::queue_upload(peer_sender.clone())
        .or(peers::send_share_resquest(peer_sender.clone()))
        .or(peers::send_user_info(peer_sender))
}

pub(crate) fn users_routes(
    sender: VesselSender<ServerRequest>,
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    users::get_all_connected_users(db).or(users::get_user_status(sender))
}

pub(crate) fn search_routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    search::search(sender)
}
