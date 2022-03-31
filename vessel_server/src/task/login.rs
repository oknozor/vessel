use futures::TryFutureExt;
use soulseek_protocol::server::login::LoginRequest;
use soulseek_protocol::server::request::ServerRequest;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub fn spawn_login_task(login_sender: Sender<ServerRequest>) -> JoinHandle<()> {
    debug!("Spawning logging task");
    tokio::spawn(async move {
        let listen_port_sender = login_sender.clone();
        let parent_request_sender = login_sender.clone();
        let join_nicotine_room = login_sender.clone();
        let username = &vessel_database::settings::CONFIG.username;
        let password = &vessel_database::settings::CONFIG.password;
        login_sender
            .send(ServerRequest::Login(LoginRequest::new(username, password)))
            .and_then(|_| listen_port_sender.send(ServerRequest::SetListenPort(2255)))
            .and_then(|_| parent_request_sender.send(ServerRequest::NoParents(true)))
            .and_then(|_| join_nicotine_room.send(ServerRequest::JoinRoom("nicotine".to_string())))
            .await
            .expect("Unable to establish connection with soulseek server");
    })
}
