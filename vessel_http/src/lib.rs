#[macro_use]
extern crate log;

use serde_derive::{Deserialize, Serialize};
use soulseek_protocol::server::messages::request::ServerRequest;
use tokio::sync::mpsc;

use percent_encoding::percent_decode;
use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::p2p::request::PeerRequest;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::messages::chat::SayInChat;
use soulseek_protocol::server::messages::search::SearchRequest;
use warp::{Filter};
use std::fmt::Debug;

#[derive(Deserialize, Serialize)]
struct QueueRequest {
    filename: String,
}

#[derive(Deserialize, Serialize)]
struct ChatMessage {
    message: String,
}

#[derive(Debug)]
struct VesselSender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> VesselSender<T> where T: Debug {
    fn new(sender: mpsc::Sender<T>) -> Self {
        Self {
            inner: sender
        }
    }

    fn send(&self, t: T) {
        self.inner.try_send(t).unwrap();
    }
}

impl<T> Clone for VesselSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}


pub async fn start(
    slsk_sender: mpsc::Sender<ServerRequest>,
    peer_message_sender: mpsc::Sender<(String, PeerRequestPacket)>,
    database: Database,
) {
    let sender = VesselSender::new(slsk_sender);
    let peer_sender = VesselSender::new(peer_message_sender);

    let sender_copy = sender.clone();
    let get_peer_adress = warp::path!("user" / String / "peer_address").map(move |username| {
        sender_copy.send(ServerRequest::GetPeerAddress(username));
        "ok"
    });

    let sender_copy = sender.clone();
    let join_room = join_room(sender_copy);

    let sender_copy = sender.clone();
    let get_user_status = warp::path!("users" / String / "status").map(move |username| {
        sender_copy.send(ServerRequest::GetUserStatus(username));
        "ok"
    });

    let sender_copy = sender.clone();
    let send_chat_message = warp::post()
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
        });

    let sender_copy = sender.clone();
    let add_user = warp::path!("user" / String / "add").map(move |username| {
        sender_copy.send(ServerRequest::AddUser(username));
        "ok"
    });

    let sender_copy = sender.clone();
    let remove_user = warp::path!("user" / String / "remove").map(move |username| {
        sender_copy.send(ServerRequest::RemoveUser(username));
        "ok"
    });

    let sender_copy = sender.clone();
    let _start_public_chat = warp::path!("chat" / "start").map(move || {
        sender_copy.send(ServerRequest::EnablePublicChat);
        "ok"
    });

    let sender_copy = sender.clone();
    let stop_public_chat = warp::path!("chat" / "stop").map(move || {
        sender_copy.send(ServerRequest::DisablePublicChat);
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let send_user_info = warp::path!("peers" / String / "userinfo").map(move |peer_name| {
        peer_sender_copy.send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::UserInfoRequest),
        ));
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let send_share_request = warp::path!("peers" / String / "shares").map(move |peer_name| {
        peer_sender_copy
            .send((peer_name, PeerRequestPacket::Message(PeerRequest::SharesRequest), ));
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let queue_upload = warp::post()
        .and(warp::path!("peers" / String / "queue"))
        .and(warp::body::json())
        .map(move |peer_name, request: QueueRequest| {
            peer_sender_copy.send((
                peer_name,
                PeerRequestPacket::Message(PeerRequest::QueueUpload {
                    filename: request.filename,
                }),
            ));
            "ok"
        });

    let get_all_connected_users = warp::path!("peers").map(move || {
        database
            .find_all()
            .iter()
            .map(|(k, v)| format!("{}@{}", k, v))
            .collect::<Vec<String>>()
            .join(",")
    });

    let sender_copy = sender.clone();
    let search_resquest = warp::path!("search" / String).map(move |query| {
        sender_copy.send(ServerRequest::FileSearch(SearchRequest {
            ticket: rand::random(),
            query,
        }));
        "ok"
    });

    let routes = warp::get()
        .and(get_peer_adress)
        .or(join_room)
        .or(get_user_status)
        .or(add_user)
        .or(remove_user)
        .or(send_chat_message)
        .or(search_resquest)
        .or(stop_public_chat)
        .or(get_all_connected_users)
        .or(send_share_request)
        .or(queue_upload)
        .or(send_user_info);

    info!("Starting vessel http ...");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn join_room(sender: VesselSender<ServerRequest>) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    warp::path!("rooms" / String / "join").map(move |room: String| {
        let room = percent_decode(room.as_bytes())
            .decode_utf8()
            .unwrap()
            .to_string();

        sender.send(ServerRequest::JoinRoom(room));
        "ok"
    })
}
