use crate::frame::ToBytes;
use crate::server_message::chat::SayInChat;
use crate::server_message::login::LoginRequest;
use crate::server_message::{MessageCode, HEADER_LEN};
use crate::write_string;
use tokio::io::{self, AsyncWriteExt, BufWriter, AsyncWrite};

#[derive(Debug, Deserialize, Serialize)]
/// This is undocumented yet, refer to [soulseek protocol documentation on nicotine+](https://nicotine-plus.github.io/nicotine-plus/doc/SLSKPROTOCOL.html#peer-messages)
pub enum ServerRequest {
    Login(LoginRequest),
    SetListenPort(u32),
    GetPeerAddress(String),
    AddUser(String),
    RemoveUser(String),
    GetUserStatus(String),
    SendChatMessage(SayInChat),
    JoinRoom(String),
    LeaveRoom(String),
    EnablePublicChat,
    DisablePublicChat,
    GetUserStats(String),
    /// TODO
    Unimplemented,
}

impl ServerRequest {
    pub fn kind(&self) -> &str {
        match self {
            ServerRequest::Login(_) => "Login",
            ServerRequest::SetListenPort(_) => "SetListenPort",
            ServerRequest::GetPeerAddress(_) => "GetPeerAddress",
            ServerRequest::AddUser(_) => "AddUser",
            ServerRequest::RemoveUser(_) => "RemoveUser",
            ServerRequest::GetUserStatus(_) => "GetUserStatus",
            ServerRequest::SendChatMessage(_) => "SendChatMessage",
            ServerRequest::JoinRoom(_) => "JoinRoom",
            ServerRequest::LeaveRoom(_) => "LeaveRoom",
            ServerRequest::EnablePublicChat => "EnablePublicChat",
            ServerRequest::DisablePublicChat => "DisablePublicChat",
            ServerRequest::GetUserStats(_) => "GetUserStats",
            ServerRequest::Unimplemented => "Unimplemented",
        }
    }
}
#[async_trait]
impl ToBytes for ServerRequest {
    async fn write_to_buf(&self, buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>) -> io::Result<()> {
        match self {
            ServerRequest::Login(login_request) => login_request.write_to_buf(buffer).await,
            ServerRequest::SetListenPort(_) => todo!(),
            ServerRequest::GetPeerAddress(username) => {
                write_str_msg(username, MessageCode::GetPeerAddress, buffer).await
            }
            ServerRequest::AddUser(_) => todo!(),
            ServerRequest::RemoveUser(_) => todo!(),
            ServerRequest::GetUserStatus(_) => todo!(),
            ServerRequest::SendChatMessage(_) => todo!(),
            ServerRequest::JoinRoom(join_room) => {
                write_str_msg(join_room, MessageCode::JoinRoom, buffer).await
            }
            ServerRequest::LeaveRoom(_) => todo!(),
            ServerRequest::EnablePublicChat => todo!(),
            ServerRequest::DisablePublicChat => todo!(),
            ServerRequest::GetUserStats(_) => todo!(),
            ServerRequest::Unimplemented => todo!(),
        }
    }
}

pub async fn write_str_msg(
    src: &str,
    code: MessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    let bytes = src.as_bytes();
    let message_len = bytes.len() as u32 + HEADER_LEN;
    buffer.write_u32_le(message_len).await?;
    buffer.write_u32_le(code as u32).await?;
    write_string(src, buffer).await?;
    Ok(())
}
