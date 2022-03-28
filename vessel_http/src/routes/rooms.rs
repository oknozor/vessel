use percent_encoding::percent_decode;
use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;

use soulseek_protocol::server::{chat::SayInChat, request::ServerRequest};

use crate::{model::ChatMessage, sender::VesselSender};
use crate::routes::with_sender;


pub fn routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let send_chat_message = warp::path!("rooms" / String)
        .and(warp::post())
        .and(warp::body::json::<ChatMessage>())
        .and(with_sender(sender.clone()))
        .and_then(send_chat_message_handler);

    let all_rooms = warp::path!("rooms")
        .and(warp::get())
        .and(with_sender(sender.clone()))
        .and_then(all_rooms_handler);

    let join_room = warp::path!("rooms" / String / "join")
        .and(warp::post())
        .and(with_sender(sender))
        .and_then(join_room_handler);

    send_chat_message
        .or(all_rooms)
        .or(join_room)
}


async fn send_chat_message_handler(
    room: String,
    message: ChatMessage,
    sender: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    let room = percent_decode(room.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    sender.send(ServerRequest::SendChatMessage(SayInChat {
        room,
        message: message.message,
    })).await;

    Ok(StatusCode::NO_CONTENT)
}

async fn all_rooms_handler(sender: VesselSender<ServerRequest>) -> Result<impl Reply, Rejection> {
    info!("Dispatch HTTP RoomList request to Vessel server");
    sender.send(ServerRequest::RoomList).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn join_room_handler(
    room: String,
    sender: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {

    let room = percent_decode(room.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    sender.send(ServerRequest::JoinRoom(room)).await;

    Ok(StatusCode::NO_CONTENT)
}
