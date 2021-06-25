use warp::Filter;

use soulseek_protocol::server::request::ServerRequest;

use crate::sender::VesselSender;

pub fn start_public_chat(
    sender_copy: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("chat" / "start").map(move || {
        sender_copy.send(ServerRequest::EnablePublicChat);
        "ok"
    })
}

pub fn stop_public_chat(
    sender_copy: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("chat" / "stop").map(move || {
        sender_copy.send(ServerRequest::DisablePublicChat);
        "ok"
    })
}
