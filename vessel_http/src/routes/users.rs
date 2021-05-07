use warp::Filter;

use soulseek_protocol::database::Database;
use soulseek_protocol::server::messages::request::ServerRequest;

use crate::sender::VesselSender;

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
        database
            .find_all()
            .iter()
            .map(|(k, v)| format!("{}@{}", k, v))
            .collect::<Vec<String>>()
            .join(",")
    })
}
