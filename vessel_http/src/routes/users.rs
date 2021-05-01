use warp::Filter;

use soulseek_protocol::database::Database;
use soulseek_protocol::server::messages::request::ServerRequest;

use crate::sender::VesselSender;

pub fn get_user_status(
    sender_copy: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String / "status").map(move |username| {
        sender_copy.send(ServerRequest::GetUserStatus(username));
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
