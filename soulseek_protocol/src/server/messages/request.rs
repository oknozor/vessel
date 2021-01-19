use crate::frame::write_string;
use crate::frame::ToBytes;
use crate::server::messages::chat::SayInChat;
use crate::server::messages::login::LoginRequest;
use crate::server::messages::{MessageCode, HEADER_LEN};
use tokio::io::{self, AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Deserialize, Serialize)]
/// All outgoing message we can send to the soulseek server.
pub enum ServerRequest {
    ///  **Description** : Send your username, password, and client version.
    ///
    /// **Response** : [`ServerResponse::LoginResponse`][`crate::server.messages::response::ServerResponse::LoginResponse`]
    Login(LoginRequest),
    ///  **Description** : We send this to the server to indicate the port number that we listen on (2234 by default).
    ///
    /// **Response** : no message
    SetListenPort(u32),
    ///  **Description** : We send this to the server to ask for a peer's address (IP address and port), given the peer's username.
    ///
    /// **Response** : no message
    GetPeerAddress(String),
    ///  **Description** : Used to be kept updated about a user's stats. When a user's stats have changed, the server
    ///  sends a [`ServerRequest::GetUserStats`] response message with the new user stats.
    ///
    /// **Response** : [`ServerResponse::UserAdded`][`crate::server.messages::response::ServerResponse::UserAdded`]
    AddUser(String),
    ///  **Description** : Used when we no longer want to be kept updated about a user's stats.
    ///
    /// **Response** : [`ServerResponse::UserRemoved`][`crate::server.messages::response::ServerResponse::UserRemoved`]
    RemoveUser(String),
    ///  **Description** : We want to known if a user has gone away or has returned.
    ///
    /// **Response** : [`ServerResponse::UserStatus`][`crate::server.messages::response::ServerResponse::UserStatus`]
    GetUserStatus(String),
    ///  **Description** : We want to say something in the chatroom.
    ///
    /// **Response** : [`ServerResponse::ChatMessage`][`crate::server.messages::response::ServerResponse::ChatMessage`]
    ///
    /// **Note** : once [`ServerRequest::EnablePublicChat`] has been sent we will receive chat message
    /// continuously.
    SendChatMessage(SayInChat),
    ///  **Description** : We want to join a room.
    ///
    /// **Response** : [`ServerResponse::RoomJoined`][`crate::server.messages::response::ServerResponse::RoomJoined`]
    JoinRoom(String),
    ///  **Description** : We send this to the server when we want to leave a room.
    ///
    /// **Response** : [`ServerResponse::RoomLeft`][`crate::server.messages::response::ServerResponse::RoomLeft`]
    LeaveRoom(String),
    ///  **Description** : We ask the server to send us messages from all public rooms, also known as public chat.
    ///
    /// **Response** : no message
    EnablePublicChat,
    ///  **Description** : We ask the server to stop sending us messages from all public rooms, also known as public chat.
    ///
    /// **Response** : no message
    DisablePublicChat,
    /// **Description** : The server sends this to indicate a change in a user’s statistics,
    /// if we’ve requested to watch the user in AddUser previously. A user’s stats can also be
    /// requested by sending a GetUserStats message to the server, but AddUser should be used instead.
    ///
    /// **Response** : [`ServerResponse::UserStats`][`crate::server.messages::response::ServerResponse::UserStats`]
    GetUserStats(String),
    /// **Description** : We inform the server if we have a distributed parent or not. If not, the server eventually
    /// sends us a PossibleParents message with a list of 10 possible parents to connect to.
    ///
    /// **Response** : no message
    NoParents(bool),

    /// TODO
    Unimplemented,
}

#[async_trait]
impl ToBytes for ServerRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> io::Result<()> {
        match self {
            ServerRequest::Login(login_request) => login_request.write_to_buf(buffer).await,
            ServerRequest::SetListenPort(port) => {
                write_u32_msg(*port, MessageCode::SetListenPort, buffer).await
            }
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
            ServerRequest::NoParents(value) => {
                write_bool_msg(*value, MessageCode::HaveNoParents, buffer).await
            }
            ServerRequest::Unimplemented => todo!(),
        }
    }
}

impl ServerRequest {
    /// Pretty print the request kind for logging purpose
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
            ServerRequest::NoParents(_) => "NoParent",
            ServerRequest::Unimplemented => "Unimplemented",
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

pub async fn write_bool_msg(
    src: bool,
    code: MessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    buffer.write_u32_le(5).await?;
    buffer.write_u32_le(code as u32).await?;

    if src {
        buffer.write_u8(1).await?;
    } else {
        buffer.write_u8(0).await?;
    }

    Ok(())
}

pub async fn write_u32_msg(
    src: u32,
    code: MessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    let message_len = HEADER_LEN;
    buffer.write_u32_le(message_len).await?;
    buffer.write_u32_le(code as u32).await?;
    buffer.write_u32_le(src).await?;
    Ok(())
}
