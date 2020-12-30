use crate::frame::ParseBytes;
use crate::server_message::chat::*;
use crate::server_message::login::*;
use crate::server_message::peer::{Parent, PeerConnection};
use crate::server_message::room::*;
use crate::server_message::user::*;
use crate::server_message::MessageCode::{ConnectToPeer, SetRoomTicker};
use crate::server_message::{Header, MessageCode, HEADER_LEN};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Deserialize, Serialize)]
/// All incoming message from the Soulseek server.
pub enum ServerResponse {
    LoginResponse(LoginResponse),
    RoomList(RoomList),
    PrivilegedUsers(UserList),
    ParentMinSpeed(u32),
    ParentSpeedRatio(u32),
    ParentInactivityTimeOut(u32),
    WishlistInterval(u32),
    ListenPort(u32),
    PeerAddress(PeerAddress),
    UserStatus(UserStatus),
    UserAdded(UserAdded),
    UserRemoved(UserRoomEvent),
    RemoveUser(UserRoomEvent),
    UserJoinedRoom(UserJoinedRoom),
    UserLeftRoom(UserRoomEvent),
    RoomJoined(RoomJoined),
    RoomTickers(RoomTickers),
    RoomLeft(String),
    RoomJoinRequestAck(String),
    ChatMessage(ChatMessage),
    UserStats(UserStats),
    ConnectToPeer(PeerConnection),
    PossibleParents(Vec<Parent>),
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
            ServerResponse::ParentInactivityTimeOut(_) => "ParentInactivityTimeOut",
            ServerResponse::WishlistInterval(_) => "WishlistInterval",
            ServerResponse::ListenPort(_) => "ListenPort",
            ServerResponse::PeerAddress(_) => "PeerAddress",
            ServerResponse::UserStatus(_) => "UserStatus",
            ServerResponse::UserAdded(_) => "UserAdded",
            ServerResponse::UserRemoved(_) => "UserRemoved",
            ServerResponse::RemoveUser(_) => "RemoveUser",
            ServerResponse::UserJoinedRoom(_) => "UserJoinedRoom",
            ServerResponse::UserLeftRoom(_) => "UserLeftRoom",
            ServerResponse::RoomJoined(_) => "RoomJoined",
            ServerResponse::RoomTickers(_) => "RoomTickers",
            ServerResponse::RoomLeft(_) => "RoomLeft",
            ServerResponse::RoomJoinRequestAck(_) => "RoomJoinRequestAck",
            ServerResponse::ChatMessage(_) => "ChatMessage",
            ServerResponse::UserStats(_) => "UserStats",
            ServerResponse::ConnectToPeer(_) => "PeerConnection",
            ServerResponse::Unknown(_, _, _) => "Unknown",
            ServerResponse::PossibleParents(_) => "PossibleParents",
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
        match header.code {
            MessageCode::Login => LoginResponse::parse(src).map(ServerResponse::LoginResponse),
            MessageCode::SetListenPort => Ok(ServerResponse::ListenPort(src.get_u32_le())),
            MessageCode::GetPeerAddress => PeerAddress::parse(src).map(ServerResponse::PeerAddress),
            MessageCode::AddUser => UserAdded::parse(src).map(ServerResponse::UserAdded),
            MessageCode::RemoveUser => UserRoomEvent::parse(src).map(ServerResponse::UserRemoved),
            MessageCode::GetUserStatus => UserStatus::parse(src).map(ServerResponse::UserStatus),
            MessageCode::SayInChatRoom => todo!(),
            MessageCode::JoinRoom => RoomJoined::parse(src).map(ServerResponse::RoomJoined),
            MessageCode::LeaveRoom => todo!(),
            MessageCode::UserJoinedRoom => {
                UserJoinedRoom::parse(src).map(ServerResponse::UserJoinedRoom)
            }
            MessageCode::UserLeftRoom => todo!(),
            MessageCode::ConnectToPeer => {
                PeerConnection::parse(src).map(ServerResponse::ConnectToPeer)
            }
            MessageCode::PrivateMessages => todo!(),
            MessageCode::AcknowledgePrivateMessage => todo!(),
            MessageCode::FileSearch => todo!(),
            MessageCode::SetOnlineStatus => todo!(),
            MessageCode::Ping => todo!(),
            MessageCode::SendConnectToken => todo!(),
            MessageCode::SendDownloadSpeed => todo!(),
            MessageCode::SharedFoldersAndFiles => todo!(),
            MessageCode::GetUserStats => todo!(),
            MessageCode::QueuedDownloads => todo!(),
            MessageCode::KickedfromServer => todo!(),
            MessageCode::UserSearch => todo!(),
            MessageCode::InterestAdd => todo!(),
            MessageCode::InterestRemove => todo!(),
            MessageCode::GetRecommendations => todo!(),
            MessageCode::GetGlobalRecommendations => todo!(),
            MessageCode::GetUserInterests => todo!(),
            MessageCode::AdminCommand => todo!(),
            MessageCode::PlaceInLineResponse => todo!(),
            MessageCode::RoomAdded => todo!(),
            MessageCode::RoomRemoved => todo!(),
            MessageCode::RoomList => RoomList::parse(src).map(ServerResponse::RoomList),
            MessageCode::ExactFileSearch => todo!(),
            MessageCode::GlobalAdminMessage => todo!(),
            MessageCode::GlobalUserList => todo!(),
            MessageCode::TunneledMessage => todo!(),
            MessageCode::PrivilegedUsers => {
                UserList::parse(src).map(ServerResponse::PrivilegedUsers)
            }
            MessageCode::HaveNoParents => todo!(),
            MessageCode::ParentsIP => todo!(),
            MessageCode::ParentMinSpeed => Ok(ServerResponse::ParentMinSpeed(src.get_u32_le())),
            MessageCode::ParentSpeedRatio => Ok(ServerResponse::ParentSpeedRatio(src.get_u32_le())),
            MessageCode::ParentInactivityTimeout => {
                Ok(ServerResponse::ParentInactivityTimeOut(src.get_u32_le()))
            }
            MessageCode::SearchInactivityTimeout => todo!(),
            MessageCode::MinimumParentsInCache => todo!(),
            MessageCode::DistributedAliveInterval => todo!(),
            MessageCode::AddPrivilegedUser => todo!(),
            MessageCode::CheckPrivileges => todo!(),
            MessageCode::SearchRequest => todo!(),
            MessageCode::AcceptChildren => todo!(),
            MessageCode::PossibleParents => Vec::parse(src).map(ServerResponse::PossibleParents),
            MessageCode::WishlistSearch => todo!(),
            MessageCode::WishlistInterval => Ok(ServerResponse::WishlistInterval(src.get_u32_le())),
            MessageCode::GetSimilarUsers => todo!(),
            MessageCode::GetItemRecommendations => todo!(),
            MessageCode::GetItemSimilarUsers => todo!(),
            MessageCode::RoomTickers => RoomTickers::parse(src).map(ServerResponse::RoomTickers),
            MessageCode::RoomTickerAdd => todo!(),
            MessageCode::RoomTickerRemove => todo!(),
            MessageCode::SetRoomTicker => todo!(),
            MessageCode::HatedInterestAdd => todo!(),
            MessageCode::HatedInterestRemove => todo!(),
            MessageCode::RoomSearch => todo!(),
            MessageCode::SendUploadSpeed => todo!(),
            MessageCode::UserPrivileges => todo!(),
            MessageCode::GivePrivileges => todo!(),
            MessageCode::NotifyPrivileges => todo!(),
            MessageCode::AcknowledgeNotifyPrivileges => todo!(),
            MessageCode::BranchLevel => todo!(),
            MessageCode::BranchRoot => todo!(),
            MessageCode::ChildDepth => todo!(),
            MessageCode::PrivateRoomUsers => todo!(),
            MessageCode::PrivateRoomAddUser => todo!(),
            MessageCode::PrivateRoomRemoveUser => todo!(),
            MessageCode::PrivateRoomDropMembership => todo!(),
            MessageCode::PrivateRoomDropOwnership => todo!(),
            MessageCode::PrivateRoomUnknown => todo!(),
            MessageCode::PrivateRoomAdded => todo!(),
            MessageCode::PrivateRoomRemoved => todo!(),
            MessageCode::PrivateRoomToggle => todo!(),
            MessageCode::NewPassword => todo!(),
            MessageCode::PrivateRoomAddOperator => todo!(),
            MessageCode::PrivateRoomRemoveOperator => todo!(),
            MessageCode::PrivateRoomOperatorAdded => todo!(),
            MessageCode::PrivateRoomOperatorRemoved => todo!(),
            MessageCode::PrivateRoomOwned => todo!(),
            MessageCode::MessageUsers => todo!(),
            MessageCode::AskPublicChat => todo!(),
            MessageCode::StopPublicChat => todo!(),
            MessageCode::PublicChatMessage => todo!(),
            MessageCode::RelatedSearches => todo!(),
            MessageCode::CantConnectToPeer => todo!(),
            MessageCode::CantCreateRoom => todo!(),
            MessageCode::MaybeRoomJoinRequestAck => todo!(),
            MessageCode::Unknown1004 => todo!(),
            MessageCode::Unknown => todo!(),
        }
    }
}
