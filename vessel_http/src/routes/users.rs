use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use soulseek_protocol::server::request::ServerRequest;

use crate::routes::{with_db, with_sender};
use crate::sender::VesselSender;
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;
use warp::reply::json;

pub fn routes(
    sender: VesselSender<ServerRequest>,
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let user_status = warp::path!("users" / String / "status")
        .and(warp::get())
        .and(with_sender(sender.clone()))
        .and_then(get_user_status_handler);

    let all_connected_users = warp::path!("users")
        .and(with_db(db))
        .and_then(get_all_connected_users_handler);

    let peer_address = warp::path!("users" / String / "address")
        .and(warp::get())
        .and(with_sender(sender))
        .and_then(get_peer_address_handler);

    all_connected_users.or(user_status).or(peer_address)
}

pub async fn get_user_status_handler(
    username: String,
    sender: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    sender.send(ServerRequest::GetUserStatus(username)).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_peer_address_handler(
    username: String,
    sender: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    sender.send(ServerRequest::GetPeerAddress(username)).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_all_connected_users_handler(database: Database) -> Result<impl Reply, Rejection> {
    Ok(json(&database.get_all::<PeerEntity>()))
}
