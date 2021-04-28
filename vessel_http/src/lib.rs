#[macro_use]
extern crate log;

use serde_derive::{Deserialize, Serialize};
use soulseek_protocol::server::messages::request::ServerRequest;
use std::sync::Mutex;
use tokio::sync::mpsc;

use soulseek_protocol::database::Database;
use soulseek_protocol::peers::messages::p2p::request::PeerRequest;
use soulseek_protocol::peers::messages::PeerRequestPacket;
use soulseek_protocol::server::messages::chat::SayInChat;
use soulseek_protocol::server::messages::search::SearchRequest;
use std::sync::Arc;
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct QueueRequest {
    filename: String,
}

pub async fn start(
    sender: mpsc::Sender<ServerRequest>,
    peer_message_dispatcher: mpsc::Sender<(String, PeerRequestPacket)>,
    database: Database,
) {
    let sender = Arc::new(Mutex::new(sender));
    let peer_sender = Arc::new(Mutex::new(peer_message_dispatcher));

    let sender_copy = sender.clone();
    let get_peer_adress = warp::path!("user" / String / "peer_address").map(move |username| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::GetPeerAddress(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let join_room = warp::path!("rooms" / String / "join").map(move |room_name| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::JoinRoom(room_name))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let get_user_status = warp::path!("users" / String / "status").map(move |username| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::GetUserStatus(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let send_chat_message = warp::path!("chat" / String / String).map(move |room, message| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::SendChatMessage(SayInChat { room, message }))
            .unwrap();

        "ok"
    });

    let sender_copy = sender.clone();
    let add_user = warp::path!("user" / String / "add").map(move |username| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::AddUser(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let remove_user = warp::path!("user" / String / "remove").map(move |username| {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::RemoveUser(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let _start_public_chat = warp::path!("chat" / "start").map(move || {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::EnablePublicChat)
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let stop_public_chat = warp::path!("chat" / "stop").map(move || {
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::DisablePublicChat)
            .unwrap();
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let send_user_info = warp::path!("peers" / String / "userinfo").map(move |peer_name| {
        let peer_sender_copy = peer_sender_copy.lock().unwrap();
        peer_sender_copy
            .try_send((
                peer_name,
                PeerRequestPacket::Message(PeerRequest::UserInfoRequest),
            ))
            .unwrap();
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let send_share_request = warp::path!("peers" / String / "shares").map(move |peer_name| {
        let peer_sender_copy = peer_sender_copy.lock().unwrap();
        peer_sender_copy
            .try_send((
                peer_name,
                PeerRequestPacket::Message(PeerRequest::SharesRequest),
            ))
            .unwrap();
        "ok"
    });

    let peer_sender_copy = peer_sender.clone();
    let queue_upload = warp::post()
        .and(warp::path!("peers" / String / "queue"))
        .and(warp::body::json())
        .map(move |peer_name, request: QueueRequest| {
            let peer_sender_copy = peer_sender_copy.lock().unwrap();
            peer_sender_copy
                .try_send((
                    peer_name,
                    PeerRequestPacket::Message(PeerRequest::QueueUpload {
                        filename: request.filename,
                    }),
                ))
                .unwrap();
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
        let sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::FileSearch(SearchRequest {
                ticket: rand::random(),
                query,
            }))
            .unwrap();
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
