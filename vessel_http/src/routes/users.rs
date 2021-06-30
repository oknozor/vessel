use warp::Filter;

use soulseek_protocol::server::request::ServerRequest;

use crate::sender::VesselSender;
use vessel_database::Database;

pub fn get_user_status(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String / "status").map(move |username| {
        sender.send(ServerRequest::GetUserStatus(username));
        "ok"
    })
}

pub fn get_peer_address(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String / "address").map(move |username| {
        sender.send(ServerRequest::GetPeerAddress(username));
        "ok"
    })
}

pub fn get_all_connected_users(
    database: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users").map(move || {
        warp::reply::json(&database.all_users())
    })
}
