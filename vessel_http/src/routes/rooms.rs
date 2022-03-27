use percent_encoding::percent_decode;
use warp::Filter;

use soulseek_protocol::server::{chat::SayInChat, request::ServerRequest};

use crate::{model::ChatMessage, sender::VesselSender};


pub fn list(sender_copy: VesselSender<ServerRequest>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("rooms"))
        .map(move || {
            sender_copy.send(ServerRequest::RoomList);
            "ok"
        })
}

pub fn send_chat_message(
    sender_copy: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("rooms" / String))
        .and(warp::body::json())
        .map(move |room: String, chat_message: ChatMessage| {
            let room = percent_decode(room.as_bytes())
                .decode_utf8()
                .unwrap()
                .to_string();

            sender_copy.send(ServerRequest::SendChatMessage(SayInChat {
                room,
                message: chat_message.message,
            }));
            "ok"
        })
}

pub fn join_room(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("rooms" / String / "join").map(move |room: String| {
        let room = percent_decode(room.as_bytes())
            .decode_utf8()
            .unwrap()
            .to_string();

        sender.send(ServerRequest::JoinRoom(room));
        "ok"
    })
}
