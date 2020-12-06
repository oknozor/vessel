use crate::server_message::chat::*;
use crate::server_message::login::*;
use crate::server_message::response::ServerResponse::UserRemoved;
use crate::server_message::room::*;
use crate::server_message::user::*;
use crate::server_message::{Header, MessageCode, ParseBytes, HEADER_LEN};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Deserialize, Serialize)]
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
    RoomLeft(String),
    RoomJoinRequestAck(String),
    ChatMessage(ChatMessage),
    UserStats(UserStats),
    Unknown(u32, u32, Vec<u8>), // length, code, raw bytes,
}

impl ServerResponse {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<Header, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < 8 {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let (len, code) = get_header(src)?;
        if src.remaining() < len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(HEADER_LEN as usize);
            Ok(Header {
                code,
                message_len: len,
            })
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
            MessageCode::JoinRoom => todo!(),
            MessageCode::LeaveRoom => todo!(),
            MessageCode::UserJoinedRoom => todo!(),
            MessageCode::UserLeftRoom => todo!(),
            MessageCode::ConnectToPeer => todo!(),
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
            MessageCode::PossibleParents => todo!(),
            MessageCode::WishlistSearch => todo!(),
            MessageCode::WishlistInterval => Ok(ServerResponse::WishlistInterval(src.get_u32_le())),
            MessageCode::GetSimilarUsers => todo!(),
            MessageCode::GetItemRecommendations => todo!(),
            MessageCode::GetItemSimilarUsers => todo!(),
            MessageCode::RoomTickers => todo!(),
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

fn get_header(src: &mut Cursor<&[u8]>) -> std::io::Result<(usize, MessageCode)> {
    let message_length = src.get_u32_le();
    let code = src.get_u32_le();
    let code = MessageCode::from(code);

    // We can subtract message code from the length since we already know it
    let message_length = message_length as usize - 4;

    Ok((message_length, code))
}
