use crate::frame::{read_bool, read_string, ParseBytes};
use crate::server::messages::chat::*;
use crate::server::messages::distributed::EmbeddedDistributedMessage;
use crate::server::messages::interest::{Interests, ItemRecommendations, Recommendations};
use crate::server::messages::login::*;
use crate::server::messages::peer::{Peer, PeerConnectionRequest, PeerConnectionTicket};
use crate::server::messages::room::*;
use crate::server::messages::search::SearchQuery;
use crate::server::messages::user::*;
use crate::server::messages::{Header, MessageCode, HEADER_LEN};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Deserialize, Serialize)]
/// All incoming message from the Soulseek server.
pub enum ServerResponse {
    LoginResponse(LoginResponse),
    ListenPort(u32),
    PeerAddress(PeerAddress),
    UserAdded(UserAdded),
    UserRemoved(UserRoomEvent),
    UserStatus(UserStatus),
    ChatMessage(ChatMessage),
    RoomJoined(RoomJoined),
    RoomLeft(String),
    PrivateMessage(PrivateMessage),
    UserJoinedRoom(UserJoinedRoom),
    UserLeftRoom(UserRoomEvent),
    PeerConnectionRequest(PeerConnectionRequest),
    SearchReply(SearchQuery),
    UserStats(UserStats),
    KickedFromServer,
    Recommendations(Recommendations),
    GlobalRecommendations(Recommendations),
    UserInterests(Interests),
    RoomList(RoomList),
    AdminMessage(String),
    PrivilegedUsers(UserList),
    ParentMinSpeed(u32),
    ParentSpeedRatio(u32),
    TimeLeft(u32),
    EmbeddedMessage(EmbeddedDistributedMessage),
    PossibleParents(Vec<Peer>),
    WishlistInterval(u32),
    SimilarUsers(UsersWithStatus),
    ItemRecommendations(ItemRecommendations),
    ItemSimilarUsers(ItemSimilarUsers),
    RoomTickers(RoomTickers),
    RoomTickersAdded(RoomTicker),
    RoomTickersRemoved(UserRoomEvent),
    PrivateRoomUsers(RoomUsers),
    PrivateRoomUserAdded(UserRoomEvent),
    PrivateRoomUserRemoved(UserRoomEvent),
    PrivateRoomUnknown(String),
    PrivateRoomAdded(String),
    PrivateRoomRemoved(String),
    PrivateRoomInvitationEnabled(bool),
    NewPassword(String),
    RoomOperatorAdd(UserRoomEvent),
    RoomOperatorRemove(String),
    RoomOperatorAdded(String),
    RoomOperatorRemoved(String),
    RoomOperators(RoomUsers),
    PublicChatMessage(ChatMessage),
    CantConnectToPeer(PeerConnectionTicket),
    CantCreateRoom(String),
    Unknown(u32, u32, Vec<u8>), // length, code, raw bytes,
}

impl ServerResponse {
    pub fn kind(&self) -> &str {
        match self {
            ServerResponse::LoginResponse(_) => "LoginResponse",
            ServerResponse::RoomList(_) => "RoomList",
            ServerResponse::PrivilegedUsers(_) => "PrivilegedUsers",
            ServerResponse::ParentMinSpeed(_) => "ParentMinSpeed",
            ServerResponse::ParentSpeedRatio(_) => "ParentSpeedRatio",
            ServerResponse::ListenPort(_) => "ListenPort",
            ServerResponse::PeerAddress(_) => "PeerAddress",
            ServerResponse::UserStatus(_) => "UserStatus",
            ServerResponse::UserAdded(_) => "UserAdded",
            ServerResponse::UserRemoved(_) => "UserRemoved",
            ServerResponse::UserJoinedRoom(_) => "UserJoinedRoom",
            ServerResponse::UserLeftRoom(_) => "UserLeftRoom",
            ServerResponse::RoomJoined(_) => "RoomJoined",
            ServerResponse::RoomTickers(_) => "RoomTickers",
            ServerResponse::RoomLeft(_) => "RoomLeft",
            ServerResponse::ChatMessage(_) => "ChatMessage",
            ServerResponse::UserStats(_) => "UserStats",
            ServerResponse::PeerConnectionRequest(_) => "PeerConnection",
            ServerResponse::PossibleParents(_) => "PossibleParents",
            ServerResponse::Unknown(_, _, _) => "Unknown",
            ServerResponse::PrivateMessage(_) => "PrivateMessage",
            ServerResponse::TimeLeft(_) => "TimeLeft",
            ServerResponse::SearchReply(_) => "SearchReply",
            ServerResponse::Recommendations(_) => "Recommendation",
            ServerResponse::GlobalRecommendations(_) => "GlobalRecommendation",
            ServerResponse::UserInterests(_) => "Interests",
            ServerResponse::SimilarUsers(_) => "SimilarUsers",
            ServerResponse::ItemRecommendations(_) => "ItemRecommendations",
            ServerResponse::ItemSimilarUsers(_) => "ItemSimilarUsers",
            ServerResponse::PrivateRoomInvitationEnabled(_) => "PrivateRoomInvitationEnabled",
            ServerResponse::NewPassword(_) => "NewPassword",
            ServerResponse::RoomOperatorAdd(_) => "RoomOperatorAdd",
            ServerResponse::RoomOperatorRemove(_) => "RoomOperatorRemove",
            ServerResponse::RoomOperatorAdded(_) => "RoomOperatorAdded",
            ServerResponse::RoomOperatorRemoved(_) => "RoomOperatorRemoved",
            ServerResponse::KickedFromServer => "KickedFromServer",
            ServerResponse::AdminMessage(_) => "AdminMessage",
            ServerResponse::EmbeddedMessage(_) => "EmbeddedMessage",
            ServerResponse::RoomTickersAdded(_) => "RoomTickersAdded",
            ServerResponse::RoomTickersRemoved(_) => "RoomTickersRemoved",
            ServerResponse::PrivateRoomUsers(_) => "PrivateRoomUsers",
            ServerResponse::PrivateRoomUserAdded(_) => "PrivateRoomUserAdded",
            ServerResponse::PrivateRoomUserRemoved(_) => "PrivateRoomUserRemoved",
            ServerResponse::PrivateRoomUnknown(_) => "PrivateUnknown",
            ServerResponse::PrivateRoomAdded(_) => "PrivateRoomAdded",
            ServerResponse::PrivateRoomRemoved(_) => "PrivateRoomRemoved",
            ServerResponse::RoomOperators(_) => "RoomOperators",
            ServerResponse::PublicChatMessage(_) => "PublicChatMessage",
            ServerResponse::CantConnectToPeer(_) => "CantConnectToPeer",
            ServerResponse::CantCreateRoom(_) => "CantCreateRoom",
            ServerResponse::WishlistInterval(_) => "WishlistInterval",
        }
    }
}

impl ServerResponse {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<Header, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = Header::read(src)?;

        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>, header: &Header) -> std::io::Result<ServerResponse> {
        match &header.code {
            MessageCode::Login => LoginResponse::parse(src).map(ServerResponse::LoginResponse),
            MessageCode::SetListenPort => Ok(ServerResponse::ListenPort(src.get_u32_le())),
            MessageCode::GetPeerAddress => PeerAddress::parse(src).map(ServerResponse::PeerAddress),
            MessageCode::AddUser => UserAdded::parse(src).map(ServerResponse::UserAdded),
            MessageCode::RemoveUser => UserRoomEvent::parse(src).map(ServerResponse::UserRemoved),
            MessageCode::GetUserStatus => UserStatus::parse(src).map(ServerResponse::UserStatus),
            MessageCode::SayInChatRoom => ChatMessage::parse(src).map(ServerResponse::ChatMessage),
            MessageCode::JoinRoom => RoomJoined::parse(src).map(ServerResponse::RoomJoined),
            MessageCode::LeaveRoom => read_string(src).map(ServerResponse::RoomLeft),
            MessageCode::UserJoinedRoom => {
                UserJoinedRoom::parse(src).map(ServerResponse::UserJoinedRoom)
            }
            MessageCode::UserLeftRoom => {
                UserRoomEvent::parse(src).map(ServerResponse::UserLeftRoom)
            }
            MessageCode::ConnectToPeer => {
                PeerConnectionRequest::parse(src).map(ServerResponse::PeerConnectionRequest)
            }
            MessageCode::PrivateMessages => {
                PrivateMessage::parse(src).map(ServerResponse::PrivateMessage)
            }
            MessageCode::FileSearch => SearchQuery::parse(src).map(ServerResponse::SearchReply),
            MessageCode::GetUserStats => UserStats::parse(src).map(ServerResponse::UserStats),
            MessageCode::KickedFromServer => Ok(ServerResponse::KickedFromServer),
            MessageCode::GetRecommendations => {
                Recommendations::parse(src).map(ServerResponse::Recommendations)
            }
            MessageCode::GetGlobalRecommendations => {
                Recommendations::parse(src).map(ServerResponse::GlobalRecommendations)
            }
            MessageCode::GetUserInterests => {
                Interests::parse(src).map(ServerResponse::UserInterests)
            }
            MessageCode::RoomList => RoomList::parse(src).map(ServerResponse::RoomList),
            MessageCode::GlobalAdminMessage => read_string(src).map(ServerResponse::AdminMessage),
            MessageCode::PrivilegedUsers => {
                UserList::parse(src).map(ServerResponse::PrivilegedUsers)
            }
            MessageCode::ParentMinSpeed => Ok(ServerResponse::ParentMinSpeed(src.get_u32_le())),
            MessageCode::ParentSpeedRatio => Ok(ServerResponse::ParentSpeedRatio(src.get_u32_le())),
            MessageCode::CheckPrivileges => Ok(ServerResponse::TimeLeft(src.get_u32_le())),
            MessageCode::EmbeddedMessage => {
                EmbeddedDistributedMessage::parse(src).map(ServerResponse::EmbeddedMessage)
            }
            MessageCode::PossibleParents => Vec::parse(src).map(ServerResponse::PossibleParents),
            MessageCode::WishlistInterval => Ok(ServerResponse::WishlistInterval(src.get_u32_le())),
            MessageCode::GetSimilarUsers => {
                UsersWithStatus::parse(src).map(ServerResponse::SimilarUsers)
            }
            MessageCode::GetItemRecommendations => {
                ItemRecommendations::parse(src).map(ServerResponse::ItemRecommendations)
            }
            MessageCode::GetItemSimilarUsers => {
                ItemSimilarUsers::parse(src).map(ServerResponse::ItemSimilarUsers)
            }
            MessageCode::RoomTickers => RoomTickers::parse(src).map(ServerResponse::RoomTickers),
            MessageCode::RoomTickerAdd => {
                RoomTicker::parse(src).map(ServerResponse::RoomTickersAdded)
            }
            MessageCode::RoomTickerRemove => {
                UserRoomEvent::parse(src).map(ServerResponse::RoomTickersRemoved)
            }
            MessageCode::PrivateRoomUsers => {
                RoomUsers::parse(src).map(ServerResponse::PrivateRoomUsers)
            }
            MessageCode::PrivateRoomAddUser => {
                UserRoomEvent::parse(src).map(ServerResponse::PrivateRoomUserAdded)
            }
            MessageCode::PrivateRoomRemoveUser => {
                UserRoomEvent::parse(src).map(ServerResponse::PrivateRoomUserRemoved)
            }
            MessageCode::PrivateRoomUnknown => {
                Ok(ServerResponse::PrivateRoomUnknown(read_string(src)?))
            }
            MessageCode::PrivateRoomAdded => {
                Ok(ServerResponse::PrivateRoomAdded(read_string(src)?))
            }
            MessageCode::PrivateRoomRemoved => {
                Ok(ServerResponse::PrivateRoomRemoved(read_string(src)?))
            }
            MessageCode::PrivateRoomToggle => Ok(ServerResponse::PrivateRoomInvitationEnabled(
                read_bool(src)?,
            )),
            MessageCode::NewPassword => Ok(ServerResponse::NewPassword(read_string(src)?)),
            MessageCode::PrivateRoomAddOperator => {
                UserRoomEvent::parse(src).map(ServerResponse::RoomOperatorAdd)
            }
            MessageCode::PrivateRoomRemoveOperator => {
                Ok(ServerResponse::RoomOperatorRemove(read_string(src)?))
            }
            MessageCode::PrivateRoomOperatorAdded => {
                Ok(ServerResponse::RoomOperatorRemoved(read_string(src)?))
            }
            MessageCode::PrivateRoomOperatorRemoved => {
                Ok(ServerResponse::RoomOperatorAdded(read_string(src)?))
            }
            MessageCode::RoomOperators => RoomUsers::parse(src).map(ServerResponse::RoomOperators),
            MessageCode::PublicChatMessage => {
                ChatMessage::parse(src).map(ServerResponse::PublicChatMessage)
            }
            MessageCode::CantConnectToPeer => {
                PeerConnectionTicket::parse(src).map(ServerResponse::CantConnectToPeer)
            }
            MessageCode::CantCreateRoom => Ok(ServerResponse::CantCreateRoom(read_string(src)?)),
            unknown => {
                error!("Unkown message code : {:?}", unknown);
                Ok(ServerResponse::Unknown(
                    header.message_len as u32,
                    header.code.clone() as u32,
                    src.bytes().to_vec(),
                ))
            }
        }
    }
}
