use bytes::Buf;
use std::io::Cursor;

pub mod admin;
pub mod chat;
pub mod distributed;
pub mod interest;
pub mod login;
pub mod peer;
pub mod privilege;
pub mod request;
pub mod response;
pub mod room;
pub mod search;
pub mod shares;
pub mod user;

/// length of a server message header (8 bytes)
pub const HEADER_LEN: u32 = 8;

/// # [`ServerResponse`] header
///
/// | content length   | message code    |
/// | :-------------: | :-------------: |
/// | [u32] (8 bytes) | [u32] (8 bytes) |
///
/// We use this to known incoming message length and code before parsing them and consuming the buffer.
/// It's important to parse header using a [`Cursor`] in order to preserve the buffer while
/// the whole message has not been received, this way we can reset the cursor and retry later.
///
/// **Note** :
///
/// this is not used to write [`ServerRequest`] since write only once to the buffer.
///
/// [`Cursor`]: std::io::Cursor
/// [`ServerResponse`]: crate::server::response::ServerResponse
/// [`ServerRequest`]: crate::server::request::ServerRequest
#[derive(Debug)]
pub struct Header {
    pub(crate) code: MessageCode,
    pub message_len: usize,
}

impl Header {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u32_le();
        let code = MessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 4;

        Ok(Self { message_len, code })
    }
}

/// [u32] enum representation of server message code used by [`ServerRequest`] to query the server and
/// [`ServerResponse`] to read incoming messages.
/// **Note** :
///
/// soulseek protocol is not open source and change might happen in the main server in the future.
/// [`MessageCode::Unknown`] is used handle unknown message without causing panic on deserialization.
/// This should be used to track and implement unkown/new messages in the future.
///
/// [`ServerRequest`]: crate::server::request::ServerRequest
/// [`MessageCode::Unknown`]: MessageCode::Unknown
/// [`ServerResponse`]: crate::server::response::ServerResponse
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum MessageCode {
    Login = 1,
    SetListenPort = 2,
    GetPeerAddress = 3,
    AddUser = 5,
    RemoveUser = 6,
    GetUserStatus = 7,
    SayInChatRoom = 13,
    JoinRoom = 14,
    LeaveRoom = 15,
    UserJoinedRoom = 16,
    UserLeftRoom = 17,
    ConnectToPeer = 18,
    PrivateMessages = 22,
    AcknowledgePrivateMessage = 23,
    FileSearch = 26,
    SetOnlineStatus = 28,
    SharedFoldersAndFiles = 35,
    GetUserStats = 36,
    KickedFromServer = 41,
    UserSearch = 42,
    InterestAdd = 51,
    InterestRemove = 52,
    GetRecommendations = 54,
    GetGlobalRecommendations = 56,
    GetUserInterests = 57,
    AdminCommand = 58,
    RoomList = 64,
    ExactFileSearch = 65,
    GlobalAdminMessage = 66,
    PrivilegedUsers = 69,
    HaveNoParents = 71,
    ParentsIp = 73,
    ParentMinSpeed = 83,
    ParentSpeedRatio = 84,
    CheckPrivileges = 92,
    EmbeddedMessage = 93,
    AcceptChildren = 100,
    PossibleParents = 102,
    WishlistSearch = 103,
    WishlistInterval = 104,
    GetSimilarUsers = 110,
    GetItemRecommendations = 111,
    GetItemSimilarUsers = 112,
    RoomTickers = 113,
    RoomTickerAdd = 114,
    RoomTickerRemove = 115,
    SetRoomTicker = 116,
    HatedInterestAdd = 117,
    HatedInterestRemove = 118,
    RoomSearch = 120,
    SendUploadSpeed = 121,
    GivePrivileges = 123,
    BranchLevel = 126,
    BranchRoot = 127,
    ChildDepth = 129,
    PrivateRoomUsers = 133,
    PrivateRoomAddUser = 134,
    PrivateRoomRemoveUser = 135,
    PrivateRoomDropMembership = 136,
    PrivateRoomDropOwnership = 137,
    PrivateRoomUnknown = 138,
    PrivateRoomAdded = 139,
    PrivateRoomRemoved = 140,
    PrivateRoomToggle = 141,
    NewPassword = 142,
    PrivateRoomAddOperator = 143,
    PrivateRoomRemoveOperator = 144,
    PrivateRoomOperatorAdded = 145,
    PrivateRoomOperatorRemoved = 146,
    RoomOperators = 148,
    MessageUsers = 149,
    AskPublicChat = 150,
    StopPublicChat = 151,
    PublicChatMessage = 152,
    CantConnectToPeer = 1001,
    CantCreateRoom = 1003,
    Unknown,
}

impl From<u32> for MessageCode {
    fn from(value: u32) -> Self {
        match value {
            1 => MessageCode::Login,
            2 => MessageCode::SetListenPort,
            3 => MessageCode::GetPeerAddress,
            5 => MessageCode::AddUser,
            6 => MessageCode::RemoveUser,
            7 => MessageCode::GetUserStatus,
            13 => MessageCode::SayInChatRoom,
            14 => MessageCode::JoinRoom,
            15 => MessageCode::LeaveRoom,
            16 => MessageCode::UserJoinedRoom,
            17 => MessageCode::UserLeftRoom,
            18 => MessageCode::ConnectToPeer,
            22 => MessageCode::PrivateMessages,
            23 => MessageCode::AcknowledgePrivateMessage,
            26 => MessageCode::FileSearch,
            28 => MessageCode::SetOnlineStatus,
            35 => MessageCode::SharedFoldersAndFiles,
            36 => MessageCode::GetUserStats,
            41 => MessageCode::KickedFromServer,
            42 => MessageCode::UserSearch,
            51 => MessageCode::InterestAdd,
            52 => MessageCode::InterestRemove,
            54 => MessageCode::GetRecommendations,
            56 => MessageCode::GetGlobalRecommendations,
            57 => MessageCode::GetUserInterests,
            58 => MessageCode::AdminCommand,
            64 => MessageCode::RoomList,
            65 => MessageCode::ExactFileSearch,
            66 => MessageCode::GlobalAdminMessage,
            69 => MessageCode::PrivilegedUsers,
            71 => MessageCode::HaveNoParents,
            73 => MessageCode::ParentsIp,
            83 => MessageCode::ParentMinSpeed,
            84 => MessageCode::ParentSpeedRatio,
            92 => MessageCode::CheckPrivileges,
            93 => MessageCode::EmbeddedMessage,
            100 => MessageCode::AcceptChildren,
            102 => MessageCode::PossibleParents,
            103 => MessageCode::WishlistSearch,
            104 => MessageCode::WishlistInterval,
            110 => MessageCode::GetSimilarUsers,
            111 => MessageCode::GetItemRecommendations,
            112 => MessageCode::GetItemSimilarUsers,
            113 => MessageCode::RoomTickers,
            114 => MessageCode::RoomTickerAdd,
            115 => MessageCode::RoomTickerRemove,
            116 => MessageCode::SetRoomTicker,
            117 => MessageCode::HatedInterestAdd,
            118 => MessageCode::HatedInterestRemove,
            120 => MessageCode::RoomSearch,
            121 => MessageCode::SendUploadSpeed,
            123 => MessageCode::GivePrivileges,
            126 => MessageCode::BranchLevel,
            127 => MessageCode::BranchRoot,
            129 => MessageCode::ChildDepth,
            133 => MessageCode::PrivateRoomUsers,
            134 => MessageCode::PrivateRoomAddUser,
            135 => MessageCode::PrivateRoomRemoveUser,
            136 => MessageCode::PrivateRoomDropMembership,
            137 => MessageCode::PrivateRoomDropOwnership,
            138 => MessageCode::PrivateRoomUnknown,
            139 => MessageCode::PrivateRoomAdded,
            140 => MessageCode::PrivateRoomRemoved,
            141 => MessageCode::PrivateRoomToggle,
            142 => MessageCode::NewPassword,
            143 => MessageCode::PrivateRoomAddOperator,
            144 => MessageCode::PrivateRoomRemoveOperator,
            145 => MessageCode::PrivateRoomOperatorAdded,
            146 => MessageCode::PrivateRoomOperatorRemoved,
            148 => MessageCode::RoomOperators,
            149 => MessageCode::MessageUsers,
            150 => MessageCode::AskPublicChat,
            151 => MessageCode::StopPublicChat,
            152 => MessageCode::PublicChatMessage,
            1001 => MessageCode::CantConnectToPeer,
            1002 => MessageCode::CantCreateRoom,
            _ => MessageCode::Unknown,
        }
    }
}
