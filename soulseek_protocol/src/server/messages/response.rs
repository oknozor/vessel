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
    // Fixme : replace with partial debug
    pub fn kind(&self) -> String  {
        match self {
            ServerResponse::LoginResponse(_) => "LoginResponse".to_string(),
            ServerResponse::RoomList(_) => "RoomList".to_string(),
            ServerResponse::PrivilegedUsers(_) => "PrivilegedUsers".to_string(),
            ServerResponse::ParentMinSpeed(_) => "ParentMinSpeed".to_string(),
            ServerResponse::ParentSpeedRatio(_) => "ParentSpeedRatio".to_string(),
            ServerResponse::ListenPort(_) => "ListenPort".to_string(),
            ServerResponse::PeerAddress(_) => "PeerAddress".to_string(),
            ServerResponse::UserStatus(_) => "UserStatus".to_string(),
            ServerResponse::UserAdded(_) => "UserAdded".to_string(),
            ServerResponse::UserRemoved(_) => "UserRemoved".to_string(),
            ServerResponse::UserJoinedRoom(_) => "UserJoinedRoom".to_string(),
            ServerResponse::UserLeftRoom(_) => "UserLeftRoom".to_string(),
            ServerResponse::RoomJoined(_) => "RoomJoined".to_string(),
            ServerResponse::RoomTickers(_) => "RoomTickers".to_string(),
            ServerResponse::RoomLeft(_) => "RoomLeft".to_string(),
            ServerResponse::ChatMessage(_) => "ChatMessage".to_string(),
            ServerResponse::UserStats(_) => "UserStats".to_string(),
            ServerResponse::PeerConnectionRequest(r) => format!("{:?}", r),
            ServerResponse::PossibleParents(_) => "PossibleParents".to_string(),
            ServerResponse::Unknown(_, _, _) => "Unknown".to_string(),
            ServerResponse::PrivateMessage(_) => "PrivateMessage".to_string(),
            ServerResponse::TimeLeft(_) => "TimeLeft".to_string(),
            ServerResponse::SearchReply(_) => "SearchReply".to_string(),
            ServerResponse::Recommendations(_) => "Recommendation".to_string(),
            ServerResponse::GlobalRecommendations(_) => "GlobalRecommendation".to_string(),
            ServerResponse::UserInterests(_) => "Interests".to_string(),
            ServerResponse::SimilarUsers(_) => "SimilarUsers".to_string(),
            ServerResponse::ItemRecommendations(_) => "ItemRecommendations".to_string(),
            ServerResponse::ItemSimilarUsers(_) => "ItemSimilarUsers".to_string(),
            ServerResponse::PrivateRoomInvitationEnabled(_) => "PrivateRoomInvitationEnabled".to_string(),
            ServerResponse::NewPassword(_) => "NewPassword".to_string(),
            ServerResponse::RoomOperatorAdd(_) => "RoomOperatorAdd".to_string(),
            ServerResponse::RoomOperatorRemove(_) => "RoomOperatorRemove".to_string(),
            ServerResponse::RoomOperatorAdded(_) => "RoomOperatorAdded".to_string(),
            ServerResponse::RoomOperatorRemoved(_) => "RoomOperatorRemoved".to_string(),
            ServerResponse::KickedFromServer => "KickedFromServer".to_string(),
            ServerResponse::AdminMessage(_) => "AdminMessage".to_string(),
            ServerResponse::EmbeddedMessage(_) => "EmbeddedMessage".to_string(),
            ServerResponse::RoomTickersAdded(_) => "RoomTickersAdded".to_string(),
            ServerResponse::RoomTickersRemoved(_) => "RoomTickersRemoved".to_string(),
            ServerResponse::PrivateRoomUsers(_) => "PrivateRoomUsers".to_string(),
            ServerResponse::PrivateRoomUserAdded(_) => "PrivateRoomUserAdded".to_string(),
            ServerResponse::PrivateRoomUserRemoved(_) => "PrivateRoomUserRemoved".to_string(),
            ServerResponse::PrivateRoomUnknown(_) => "PrivateUnknown".to_string(),
            ServerResponse::PrivateRoomAdded(_) => "PrivateRoomAdded".to_string(),
            ServerResponse::PrivateRoomRemoved(_) => "PrivateRoomRemoved".to_string(),
            ServerResponse::RoomOperators(_) => "RoomOperators".to_string(),
            ServerResponse::PublicChatMessage(_) => "PublicChatMessage".to_string(),
            ServerResponse::CantConnectToPeer(_) => "CantConnectToPeer".to_string(),
            ServerResponse::CantCreateRoom(_) => "CantCreateRoom".to_string(),
            ServerResponse::WishlistInterval(_) => "WishlistInterval".to_string(),
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
            MessageCode::JoinRoom => {
                RoomJoined::parse(src, header.message_len).map(ServerResponse::RoomJoined)
            }
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
                read_bool(src),
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
                    src.chunk().to_vec(),
                ))
            }
        }
    }
}
