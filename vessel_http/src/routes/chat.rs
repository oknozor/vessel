use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::routes::with_sender;
use soulseek_protocol::server::request::ServerRequest;

use crate::sender::VesselSender;

pub fn routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let start_public_chat = warp::path!("chat" / "start")
        .and(warp::post())
        .and(with_sender(sender.clone()))
        .and_then(start_public_chat_handler);

    let stop_public_chat = warp::path!("chat" / "stop")
        .and(warp::post())
        .and(with_sender(sender))
        .and_then(stop_public_chat_handler);

    start_public_chat.or(stop_public_chat)
}

async fn start_public_chat_handler(
    sender_copy: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    sender_copy.send(ServerRequest::EnablePublicChat).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn stop_public_chat_handler(
    sender_copy: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    sender_copy.send(ServerRequest::DisablePublicChat).await;
    Ok(StatusCode::NO_CONTENT)
}
