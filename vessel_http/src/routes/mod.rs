use warp::Filter;

use soulseek_protocol::server::request::ServerRequest;

use crate::sender::VesselSender;
use soulseek_protocol::peers::PeerRequestPacket;
use vessel_database::Database;

pub(crate) mod chat;
pub(crate) mod peers;
pub(crate) mod rooms;
pub(crate) mod search;
pub(crate) mod transfer;
pub(crate) mod users;

pub fn routes(
    db: Database,
    sender: VesselSender<ServerRequest>,
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    rooms_routes(sender.clone())
        .or(peers_routes(peer_sender))
        .or(chat_routes(sender.clone()))
        .or(users_routes(sender.clone(), db.clone()))
        .or(search_routes(sender.clone()))
        .or(transfer_routes(db))
        .or(rooms_routes(sender))
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
    peers::queue_upload_request(peer_sender.clone())
        .or(peers::send_share_resquest(peer_sender.clone()))
        .or(peers::send_user_info_request(peer_sender))
}

pub(crate) fn users_routes(
    sender: VesselSender<ServerRequest>,
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    users::get_all_connected_users(db)
        .or(users::get_user_status(sender.clone()))
        .or(users::get_peer_address(sender))
}

pub(crate) fn search_routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    search::search(sender)
}

pub(crate) fn transfer_routes(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    transfer::get_downloads(db.clone()).or(transfer::get_uploads(db))
}
