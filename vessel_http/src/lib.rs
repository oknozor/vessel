#[macro_use]
extern crate log;

use soulseek_protocol::server::messages::request::ServerRequest;
use std::sync::Mutex;
use tokio::sync::mpsc;

use soulseek_protocol::server::messages::chat::SayInChat;
use std::sync::Arc;
use warp::Filter;

pub async fn start(sender: mpsc::Sender<ServerRequest>) {
    let sender = Arc::new(Mutex::new(sender));

    let sender_copy = sender.clone();
    let get_peer_adress = warp::path!("user" / String / "peer_address").map(move |username| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::GetPeerAddress(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let join_room = warp::path!("rooms" / String / "join").map(move |room_name| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::JoinRoom(room_name))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let get_user_status = warp::path!("users" / String / "status").map(move |username| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::GetUserStatus(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let send_chat_message = warp::path!("chat" / String / String).map(move |room, message| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::SendChatMessage(SayInChat { room, message }))
            .unwrap();

        "ok"
    });

    let sender_copy = sender.clone();
    let add_user = warp::path!("user" / String / "add").map(move |username| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::AddUser(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let remove_user = warp::path!("user" / String / "remove").map(move |username| {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::RemoveUser(username))
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let start_public_chat = warp::path!("chat" / "start").map(move || {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::EnablePublicChat)
            .unwrap();
        "ok"
    });

    let sender_copy = sender.clone();
    let stop_public_chat = warp::path!("chat" / "stop").map(move || {
        let mut sender_copy = sender_copy.lock().unwrap();
        sender_copy
            .try_send(ServerRequest::DisablePublicChat)
            .unwrap();
        "ok"
    });

    let routes = warp::get()
        .and(get_peer_adress)
        .or(join_room)
        .or(get_user_status)
        .or(send_chat_message)
        .or(add_user)
        .or(remove_user)
        .or(start_public_chat)
        .or(stop_public_chat);

    info!("Starting vessel http ...");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
