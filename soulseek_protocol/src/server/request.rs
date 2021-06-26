use crate::frame::write_string;
use crate::frame::ToBytes;
use crate::server::admin::AdminCommand;
use crate::server::chat::{GroupMessage, SayInChat};
use crate::server::login::LoginRequest;
use crate::server::peer::{PeerConnectionTicket, RequestConnectionToPeer};
use crate::server::privilege::PrivilegesGift;
use crate::server::room::{Ticker, UserRoomEvent};
use crate::server::search::{RoomSearchQuery, SearchQuery, SearchRequest};
use crate::server::shares::SharedFolderAndFiles;
use crate::server::{MessageCode, HEADER_LEN};
use tokio::io::{self, AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Deserialize, Serialize)]
/// All outgoing message we can send to the soulseek server.
pub enum ServerRequest {
    ///  **Description** : Send your username, password, and client version.
    ///
    /// **Response** : [`ServerResponse::LoginResponse`][`crate::server::response::ServerResponse::LoginResponse`]
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
    /// **Response** : [`ServerResponse::UserAdded`][`crate::server::response::ServerResponse::UserAdded`]
    AddUser(String),
    ///  **Description** : Used when we no longer want to be kept updated about a user's stats.
    ///
    /// **Response** : [`ServerResponse::UserRemoved`][`crate::server::response::ServerResponse::UserRemoved`]
    RemoveUser(String),
    ///  **Description** : We want to known if a user has gone away or has returned.
    ///
    /// **Response** : [`ServerResponse::UserStatus`][`crate::server::response::ServerResponse::UserStatus`]
    GetUserStatus(String),
    ///  **Description** : We want to say something in the chatroom.
    ///
    /// **Response** : [`ServerResponse::ChatMessage`][`crate::server::response::ServerResponse::ChatMessage`]
    ///
    /// **Note** : once [`ServerRequest::EnablePublicChat`] has been sent we will receive chat message
    /// continuously.
    SendChatMessage(SayInChat),
    ///  **Description** : We want to join a room.
    ///
    /// **Response** : [`ServerResponse::RoomJoined`][`crate::server::response::ServerResponse::RoomJoined`]
    JoinRoom(String),
    ///  **Description** : We send this to the server when we want to leave a room.
    ///
    /// **Response** : [`ServerResponse::RoomLeft`][`crate::server::response::ServerResponse::RoomLeft`]
    LeaveRoom(String),
    /// **Description** : Either we ask server to tell someone else we want to establish a connection with them.
    ///
    /// **Response** : [`ServerResponse::ConnectToPeer`][`crate::server::response::ServerResponse::ConnectToPeer`]
    ConnectToPeer(RequestConnectionToPeer),

    ///  **Description** : We send this to the server to confirm that we received a
    /// private message. If we don't send it, the server will keep sending the chat phrase to us.
    ///
    /// **Response** : no message
    AcknowledgePrivateMessage(u32),

    ///  **Description** : We send this to the server when we search for something. Alternatively,
    /// the server sends this message outside the distributed network to tell us that someone
    /// is searching for something, currently used for UserSearch and RoomSearch requests.
    //
    // The ticket/search id is a random number generated by the client and is used
    // to track the search results.
    ///
    /// **Response** : [`ServerResponse::SearchReply`][`crate::server::response::ServerResponse::SearchReply`]
    FileSearch(SearchRequest),

    ///  **Description** : We send our new status to the server. Status is a way to define whether you're available or busy.
    ///
    ///-1 = Unknown
    /// 0 = Offline
    /// 1 = Away
    /// 2 = Online
    ///
    /// **Response** : no message
    SetOnlineStatus(u32),

    ///  **Description** :We send this to server to indicate the number of folder and files that we share.
    ///
    /// **Response** : no message
    SharedFolderAndFiles(SharedFolderAndFiles),

    /// **Description** : The server sends this to indicate a change in a user’s statistics,
    /// if we’ve requested to watch the user in AddUser previously. A user’s stats can also be
    /// requested by sending a GetUserStats message to the server, but AddUser should be used instead.
    ///
    /// **Response** : [`ServerResponse::UserStats`][`crate::server::response::ServerResponse::UserStats`]
    GetUserStats(String),

    ///  **Description** : We send this to the server when we search a specific user's shares. The ticket/search id
    /// is a random number generated by the client and is used to track the search results
    ///
    /// **Response** : no message
    UserSearch(SearchQuery),

    ///  **Description** :
    //
    // We send this to the server when we add an item to our likes list.
    ///
    /// **Response** : no message
    AddLinkedInterest(String),

    ///  **Description** :
    //
    // We send this to the server when we remove an item from our likes list.
    ///
    /// **Response** : no message
    RemoveLinkedInterest(String),

    ///  **Description** : We ask the server to send us a list of personal recommendations and a number for each.
    ///
    /// **Response** : [`ServerResponse::Recommendations`][`crate::server::response::ServerResponse::Recommendations`]
    Recommendations,

    ///  **Description** : We ask the server to send us a list of global recommendations and a number for each.
    ///
    /// **Response** : [`ServerResponse::Recommendations`][`crate::server::response::ServerResponse::Recommendations`]
    GlobalRecommendations,

    ///  **Description** : We ask the server for a user's liked and hated interests. The server responds with a list of interests.
    ///
    /// **Response** : [`ServerResponse::Interests`][`crate::server::response::ServerResponse::Interests`]
    GetUserInterest(String),

    ///  **Description** : Unknown
    ///
    /// **Response** : no message
    AdminCommand(AdminCommand),

    ///  **Description** : The server tells us a list of rooms and the number of users in them.
    /// When connecting to the server, the server only sends us rooms with at least 5 users.
    /// A few select rooms are also excluded, such as nicotine and The Lobby. Requesting the room
    /// list yields a response containing the missing rooms.
    ///
    /// **Response** :  [`ServerResponse::RoomList`][`crate::server::response::ServerResponse::RoomList`]
    RoomList,

    /// **Description** : We inform the server if we have a distributed parent or not. If not, the server eventually
    /// sends us a PossibleParents message with a list of 10 possible parents to connect to.
    ///
    /// **Response** : no message
    NoParents(bool),

    ///  **Description** : We ask the server how much time we have left of our privileges.
    /// The server responds with the remaining time, in seconds.
    ///
    /// **Response** :  [`ServerResponse::TimeLeft`][`crate::server::response::ServerResponse::TimeLeft`]
    CheckPrivileges,

    ///  **Description** : We tell the server if we want to accept child nodes.
    ///
    /// **Response** : no message
    AcceptChildren(bool),

    ///  **Description** : unknown.
    ///
    /// **Response** : no message
    WishlistSearch(SearchRequest),

    ///  **Description** : The server sends us a list of similar users related to our interests.
    ///
    /// **Response** :  [`ServerResponse::SimilarUsers`][`crate::server::response::ServerResponse::SimilarUsers`]
    GetSimilarUsers,

    ///  **Description** :The server sends us a list of recommendations related to a specific item,
    /// which is usually present in the like/dislike list or an existing recommendation list.
    ///
    /// **Response** :  [`ServerResponse::ItemRecommendations`][`crate::server::response::ServerResponse::ItemRecommendations`]
    GetItemRecommendations(String),

    ///  **Description** :The server sends us a list of similar users related to a specific item,
    /// which is usually present in the like/dislike list or recommendation list.
    ///
    /// **Response** :  [`ServerResponse::ItemSimilarUsers`][`crate::server::response::ServerResponse::ItemSimilarUsers`]
    GetItemSimilarUsers(String),

    ///  **Description** : We send this to the server when we change our own ticker in a chat room.
    //
    // Tickers are customizable, user-specific messages that appear in a banner at the top of a chat room.
    ///
    /// **Response** : no message
    SetRoomTicker(Ticker),

    ///  **Description** : We send this to the server when we add an item to our hate list.
    ///
    /// **Response** : no message
    AddHatedInterest(String),

    ///  **Description** : We send this to the server when we remove an item from our hate list.
    ///
    /// **Response** : no message
    RemoveHatedInterest(String),

    ///  **Description** : We send this to the server to search files shared by users who have
    /// joined a specific chat room. The ticket/search id is a random number generated by the
    /// client and is used to track the search results.
    ///
    /// **Response** : no message
    RoomSearch(RoomSearchQuery),

    ///  **Description** : We send this after a finished upload to let the server update the
    /// speed statistics for ourselves.
    /// public chat.
    ///
    /// **Response** : no message
    SendUploadSpeed(u32),

    ///  **Description** : We give (part of) our privileges, specified in days, to another user on *
    /// the network.
    /// public chat.
    ///
    /// **Response** : no message
    GivePrivileges(PrivilegesGift),

    ///  **Description** : We tell the server what our position is in our branch (xth generation) on
    /// the distributed network.
    /// public chat.
    ///
    /// **Response** : no message
    BranchLevel(u32),

    ///  **Description** : We tell the server the username of the root of the branch we're in on the
    /// distributed network.
    ///
    /// **Response** : no message
    BranchRoot(String),

    ///  **Description** : We tell the server the maximum number of generation of children we have on
    /// the distributed network.
    ///
    /// **Response** : no message
    ChildDepth(u32),

    ///  **Description** : We send this to inform the server that we've added a user to a private room.
    ///
    /// **Response** : no message
    AddUserToPrivateRoom(UserRoomEvent),

    ///  **Description** : We send this to inform the server that we've removed a user from a private room.
    ///
    /// **Response** : no message
    RemoveUserFromPrivateRoom(UserRoomEvent),

    ///  **Description** : We send this to the server to remove our own membership of a private room.
    ///
    /// **Response** : no message
    PrivateRoomDropMemberShip(String),

    ///  **Description** : We send this to the server to stop owning a private room.
    ///
    /// **Response** : no message
    PrivateRoomDropOwnerShip(String),

    ///  **Description** : Unknown purpose
    ///
    /// **Response** : room : String
    PrivateRoomUnknown(String),

    ///  **Description** : We send this when we want to enable or disable invitations to private rooms.
    ///
    /// **Response** :  [`ServerResponse::PrivateRoomInvitationEnabled`][`crate::server::response::ServerResponse::PrivateRoomInvitationEnabled`]
    PrivateRoomToggle(bool),

    ///  **Description** : We send this to the server to change our password. We receive a response if our password changes.
    ///
    /// **Response** :  [`ServerResponse::NewPassword`][`crate::server::response::ServerResponse::NewPassword`]
    NewPassWord(String),

    ///  **Description** : We send this to the server to add private room operator abilities to a user.
    ///
    /// **Response** :  [`ServerResponse::RoomOperatorAdd`][`crate::server::response::ServerResponse::RoomOperatorAdd`]
    PrivateRoomAddOperator(UserRoomEvent),

    ///  **Description** : The server send us this message when our operator abilities are removed in a private room.
    ///
    /// **Response** :  [`ServerResponse::RoomOperatorRemove`][`crate::server::response::ServerResponse::RoomOperatorRemove`]
    PrivateRoomRemoveOperator(String),

    ///  **Description** : Sends a broadcast private message to the given list of users.
    ///
    /// **Response** : no message
    MessageUsers(GroupMessage),

    ///  **Description** : We ask the server to send us messages from all public rooms, also known as
    /// public chat.
    ///
    /// **Response** : no message
    EnablePublicChat,

    ///  **Description** : We ask the server to stop sending us messages from all public rooms, also known
    /// as public chat.
    ///
    /// **Response** : no message
    DisablePublicChat,

    ///  **Description** : We send this to say we can't connect to peer after it has asked
    /// us to connect. We receive this if we asked peer to connect and it can't do this. This message means a connection can't be established either way.
    ///
    /// **Response** : no message
    CantConnectToPeer(PeerConnectionTicket),
}

#[async_trait]
impl ToBytes for ServerRequest {
    #[instrument(level = "trace", skip(buffer, self))]
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
            ServerRequest::AddUser(username) => {
                write_str_msg(&username, MessageCode::AddUser, buffer).await
            }
            ServerRequest::RemoveUser(username) => {
                write_str_msg(&username, MessageCode::RemoveUser, buffer).await
            }
            ServerRequest::GetUserStatus(username) => {
                write_str_msg(&username, MessageCode::GetUserStatus, buffer).await
            }
            ServerRequest::SendChatMessage(message) => message.write_to_buf(buffer).await,
            ServerRequest::JoinRoom(join_room) => {
                write_str_msg(join_room, MessageCode::JoinRoom, buffer).await
            }
            ServerRequest::LeaveRoom(room) => {
                write_str_msg(room, MessageCode::LeaveRoom, buffer).await
            }
            ServerRequest::EnablePublicChat => {
                write_empty_msg(MessageCode::AskPublicChat, buffer).await
            }
            ServerRequest::DisablePublicChat => {
                write_empty_msg(MessageCode::StopPublicChat, buffer).await
            }
            ServerRequest::GetUserStats(username) => {
                write_str_msg(&username, MessageCode::GetUserStats, buffer).await
            }
            ServerRequest::NoParents(value) => {
                write_bool_msg(*value, MessageCode::HaveNoParents, buffer).await
            }
            ServerRequest::ConnectToPeer(connection_request) => {
                connection_request.write_to_buf(buffer).await
            }
            ServerRequest::AcknowledgePrivateMessage(message_id) => {
                write_u32_msg(*message_id, MessageCode::AcknowledgePrivateMessage, buffer).await
            }
            ServerRequest::FileSearch(query) => {
                query
                    .write_to_buf_with_code(buffer, MessageCode::FileSearch)
                    .await
            }
            ServerRequest::SetOnlineStatus(status) => {
                write_u32_msg(*status, MessageCode::SetOnlineStatus, buffer).await
            }
            ServerRequest::SharedFolderAndFiles(folders) => folders.write_to_buf(buffer).await,
            ServerRequest::UserSearch(query) => query.write_to_buf(buffer).await,
            ServerRequest::AddLinkedInterest(item) => {
                write_str_msg(item, MessageCode::InterestAdd, buffer).await
            }
            ServerRequest::RemoveLinkedInterest(item) => {
                write_str_msg(item, MessageCode::InterestAdd, buffer).await
            }
            ServerRequest::Recommendations => {
                write_empty_msg(MessageCode::GetRecommendations, buffer).await
            }
            ServerRequest::GlobalRecommendations => {
                write_empty_msg(MessageCode::GetGlobalRecommendations, buffer).await
            }
            ServerRequest::GetUserInterest(username) => {
                write_str_msg(username, MessageCode::GetUserInterests, buffer).await
            }
            ServerRequest::AdminCommand(command) => command.write_to_buf(buffer).await,
            ServerRequest::RoomList => write_empty_msg(MessageCode::RoomList, buffer).await,
            ServerRequest::CheckPrivileges => {
                write_empty_msg(MessageCode::CheckPrivileges, buffer).await
            }
            ServerRequest::AcceptChildren(value) => {
                write_bool_msg(*value, MessageCode::AcceptChildren, buffer).await
            }
            ServerRequest::WishlistSearch(query) => {
                query
                    .write_to_buf_with_code(buffer, MessageCode::WishlistSearch)
                    .await
            }
            ServerRequest::GetSimilarUsers => {
                write_empty_msg(MessageCode::GetSimilarUsers, buffer).await
            }
            ServerRequest::GetItemRecommendations(item) => {
                write_str_msg(item, MessageCode::GetItemRecommendations, buffer).await
            }
            ServerRequest::GetItemSimilarUsers(item) => {
                write_str_msg(item, MessageCode::GetItemSimilarUsers, buffer).await
            }
            ServerRequest::SetRoomTicker(_) => todo!(),
            ServerRequest::AddHatedInterest(_) => todo!(),
            ServerRequest::RemoveHatedInterest(_) => todo!(),
            ServerRequest::RoomSearch(_) => todo!(),
            ServerRequest::SendUploadSpeed(_) => todo!(),
            ServerRequest::GivePrivileges(_) => todo!(),
            ServerRequest::BranchLevel(_) => todo!(),
            ServerRequest::BranchRoot(_) => todo!(),
            ServerRequest::ChildDepth(_) => todo!(),
            ServerRequest::AddUserToPrivateRoom(_) => todo!(),
            ServerRequest::RemoveUserFromPrivateRoom(_) => todo!(),
            ServerRequest::PrivateRoomDropMemberShip(room) => {
                write_str_msg(room, MessageCode::GetItemSimilarUsers, buffer).await
            }
            ServerRequest::PrivateRoomDropOwnerShip(room) => {
                write_str_msg(room, MessageCode::GetItemSimilarUsers, buffer).await
            }
            ServerRequest::PrivateRoomUnknown(room) => {
                write_str_msg(room, MessageCode::GetItemSimilarUsers, buffer).await
            }
            ServerRequest::PrivateRoomToggle(_) => todo!(),
            ServerRequest::NewPassWord(_) => todo!(),
            ServerRequest::PrivateRoomAddOperator(_) => todo!(),
            ServerRequest::PrivateRoomRemoveOperator(_) => todo!(),
            ServerRequest::MessageUsers(_) => todo!(),
            ServerRequest::CantConnectToPeer(ticket) => ticket.write_to_buf(buffer).await,
        }
    }
}

pub(crate) async fn write_str_msg(
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

pub(crate) async fn write_empty_msg(
    code: MessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    buffer.write_u32_le(4).await?;
    buffer.write_u32_le(code as u32).await?;
    Ok(())
}

pub(crate) async fn write_bool_msg(
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

pub(crate) async fn write_u32_msg(
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

#[cfg(test)]
mod tests {
    use crate::frame::ToBytes;
    use crate::server::chat::SayInChat;
    use crate::server::login::LoginRequest;
    use crate::server::request::ServerRequest;
    use crate::server::room::UserRoomEvent;
    use crate::server::shares::SharedFolderAndFiles;
    use tokio::io::{AsyncWriteExt, BufWriter};
    use tokio_test::block_on;

    fn write_to_buff_blocking(request: ServerRequest) -> Vec<u8> {
        let mut data = Vec::new();
        let mut buffer = BufWriter::new(&mut data);

        let _result = block_on(async {
            request.write_to_buf(&mut buffer).await.unwrap();
            buffer.flush().await
        });

        data
    }

    #[test]
    fn write_login() {
        let login = ServerRequest::Login(LoginRequest::new("test", "s33cr3t"));

        let data = write_to_buff_blocking(login);

        assert_eq!(&data[8..], b"\x04\x00\x00\x00test\x07\x00\x00\x00s33cr3t\x9d\x00\x00\x00 \x00\x00\x00dbc93f24d8f3f109deed23c3e2f8b74c\x13\x00\x00\x00");
    }

    #[test]
    fn set_listen_port() {
        let listen_port = ServerRequest::SetListenPort(1337);

        let data = write_to_buff_blocking(listen_port);

        assert_eq!(&data[8..], b"9\x05\x00\x00");
    }

    #[test]
    fn get_peer_address() {
        let peer_address = ServerRequest::GetPeerAddress("test".to_string());

        let data = write_to_buff_blocking(peer_address);

        assert_eq!(&data[8..], b"\x04\x00\x00\x00test");
    }

    #[test]
    fn get_add_user() {
        let add_user = ServerRequest::AddUser("test".to_string());

        let data = write_to_buff_blocking(add_user);

        assert_eq!(&data[8..], b"\x04\x00\x00\x00test");
    }

    #[test]
    fn interest_add() {
        let add_interest = ServerRequest::AddUser("hip hop".to_string());

        let data = write_to_buff_blocking(add_interest);

        assert_eq!(&data[8..], [7, 0, 0, 0, 104, 105, 112, 32, 104, 111, 112]);
    }

    #[test]
    fn remove_user() {
        let remove_user = ServerRequest::RemoveUser("test".to_string());

        let data = write_to_buff_blocking(remove_user);

        assert_eq!(&data[8..], b"\x04\x00\x00\x00test");
    }

    #[test]
    fn get_user_status() {
        let get_user_status = ServerRequest::GetUserStatus("test".to_string());

        let data = write_to_buff_blocking(get_user_status);

        assert_eq!(&data[8..], b"\x04\x00\x00\x00test");
    }

    #[test]
    fn sets_status() {
        let sets_status = ServerRequest::SetOnlineStatus(1);

        let data = write_to_buff_blocking(sets_status);

        assert_eq!(&data[8..], b"\x01\x00\x00\x00");
    }

    #[test]
    fn start_chat() {
        let start_chat = ServerRequest::EnablePublicChat;

        let data = write_to_buff_blocking(start_chat);

        assert_eq!(&data[8..], b"");
    }

    #[test]
    fn chat_message() {
        let chat_message = ServerRequest::SendChatMessage(SayInChat {
            room: "nicotine".to_string(),
            message: "Wassup?".to_string(),
        });

        let data = write_to_buff_blocking(chat_message);

        assert_eq!(
            &data[8..],
            b"\x08\x00\x00\x00nicotine\x07\x00\x00\x00Wassup?"
        );
    }

    #[test]
    fn join_room() {
        let join_room = ServerRequest::JoinRoom("indie".to_string());

        let data = write_to_buff_blocking(join_room);

        assert_eq!(&data[8..], [5, 0, 0, 0, 105, 110, 100, 105, 101]);
    }

    #[test]
    fn drop_private_room_membership() {
        let drop_private_room_membership =
            ServerRequest::PrivateRoomDropMemberShip("nicotine".to_string());

        let data = write_to_buff_blocking(drop_private_room_membership);

        assert_eq!(&data[8..], b"\x08\x00\x00\x00nicotine");
    }

    #[test]
    fn drop_private_room_ownership() {
        let drop_private_room_ownership =
            ServerRequest::PrivateRoomDropOwnerShip("nicotine".to_string());

        let data = write_to_buff_blocking(drop_private_room_ownership);

        assert_eq!(&data[8..], b"\x08\x00\x00\x00nicotine");
    }

    #[test]
    fn private_room_unknown() {
        let private_room_unknown = ServerRequest::PrivateRoomUnknown("nicotine".to_string());

        let data = write_to_buff_blocking(private_room_unknown);

        assert_eq!(&data[8..], b"\x08\x00\x00\x00nicotine");
    }

    #[test]
    fn private_room_remove_user() {
        let private_room_remove_user = ServerRequest::RemoveUserFromPrivateRoom(UserRoomEvent {
            room: "nicotine".to_string(),
            username: "admin".to_string(),
        });

        let data = write_to_buff_blocking(private_room_remove_user);

        assert_eq!(&data[8..], b"\x08\x00\x00\x00nicotine\x05\x00\x00\x00admin");
    }

    #[test]
    fn check_privileges() {
        let check_privileges = ServerRequest::CheckPrivileges;

        let data = write_to_buff_blocking(check_privileges);

        assert_eq!(&data[8..], []);
    }

    #[test]
    fn have_no_parent() {
        let no_parent = ServerRequest::NoParents(true);

        let data = write_to_buff_blocking(no_parent);

        assert_eq!(&data[8..], [1]);
    }

    #[test]
    fn have_parent() {
        let have_parent = ServerRequest::NoParents(false);

        let data = write_to_buff_blocking(have_parent);

        assert_eq!(&data[8..], [0]);
    }

    #[test]
    fn room_list() {
        let room_list = ServerRequest::RoomList;

        let data = write_to_buff_blocking(room_list);

        assert_eq!(&data[8..], []);
    }

    #[test]
    fn accept_children() {
        let accept_children = ServerRequest::AcceptChildren(true);

        let data = write_to_buff_blocking(accept_children);

        assert_eq!(&data[8..], [1]);
    }

    #[test]
    fn deny_children() {
        let deny_children = ServerRequest::AcceptChildren(false);

        let data = write_to_buff_blocking(deny_children);

        assert_eq!(&data[8..], [0]);
    }

    #[test]
    fn shared_folders() {
        let shared_folders =
            ServerRequest::SharedFolderAndFiles(SharedFolderAndFiles { dirs: 2, files: 3 });

        let data = write_to_buff_blocking(shared_folders);

        assert_eq!(&data[8..], [4, 0, 0, 0, 47, 0, 0, 0]);
    }

    #[test]
    fn branch_level() {
        let branch_level = ServerRequest::BranchLevel(0);

        let data = write_to_buff_blocking(branch_level);

        assert_eq!(&data[8..], [0, 0, 0, 0]);
    }

    #[test]
    fn branch_root() {
        let branch_root = ServerRequest::BranchRoot("oknozor".to_string());

        let data = write_to_buff_blocking(branch_root);

        assert_eq!(&data[8..], [7, 0, 0, 0, 111, 107, 110, 111, 122, 111, 114]);
    }

    #[test]
    fn toggle_private_rooms() {
        let branch_root = ServerRequest::PrivateRoomToggle(true);

        let data = write_to_buff_blocking(branch_root);

        assert_eq!(&data[8..], [1]);
    }

    #[test]
    fn disable_private_rooms() {
        let branch_root = ServerRequest::PrivateRoomToggle(false);

        let data = write_to_buff_blocking(branch_root);

        assert_eq!(&data[8..], [0]);
    }
}