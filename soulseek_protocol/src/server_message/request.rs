use crate::server_message::{ToBytes};
use tokio::io::{self, BufWriter};
use tokio::net::TcpStream;
use crate::server_message::login::LoginRequest;
use crate::server_message::chat::SendChatMessage;
use tokio::prelude::AsyncWrite;

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerRequest {
    /// TODO
    Login(LoginRequest),
    SetListenPort(u32),
    GetPeerAddress(String),
    AddUser(String),
    RemoveUser(String),
    GetUserStatus(String),
    SendChatMessage(SendChatMessage),
    JoinRoom(String),
    LeaveRoom(String),
    EnablePublicChat,
    DisablePublicChat,
    GetUserStats(String),
    /// TODO
    Unimplemented,
}

#[async_trait]
impl ToBytes for ServerRequest {
    async fn write_to_buf(&self, buffer: &mut BufWriter<TcpStream>) -> io::Result<()> {
        match self {
            ServerRequest::Login(login_request) => login_request.write_to_buf(buffer).await,
            ServerRequest::SetListenPort(_) => todo!(),
            ServerRequest::GetPeerAddress(_) => todo!(),
            ServerRequest::AddUser(_) => todo!(),
            ServerRequest::RemoveUser(_) => todo!(),
            ServerRequest::GetUserStatus(_) => todo!(),
            ServerRequest::SendChatMessage(_) => todo!(),
            ServerRequest::JoinRoom(_) => todo!(),
            ServerRequest::LeaveRoom(_) => todo!(),
            ServerRequest::EnablePublicChat => todo!(),
            ServerRequest::DisablePublicChat => todo!(),
            ServerRequest::GetUserStats(_) => todo!(),
            ServerRequest::Unimplemented => todo!(),
        }
    }
}
