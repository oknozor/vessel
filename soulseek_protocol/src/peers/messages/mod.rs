use std::io::Cursor;

use bytes::Buf;

use crate::peers::messages::connection::PeerConnectionMessage;
use crate::peers::request::PeerRequest;
use crate::peers::response::PeerResponse;

pub mod connection;
pub mod folder_content;
pub mod place_in_queue;
pub mod search;
pub mod shared_directories;
pub mod transfer;
pub mod user_info;
mod zlib;

#[derive(Debug)]
pub struct PeerMessageHeader {
    pub(crate) code: MessageCode,
    pub(crate) message_len: usize,
}

impl PeerMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u32_le();
        let code = MessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = (message_length) as usize;

        Ok(Self { message_len, code })
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum MessageCode {
    SharesRequest = 4,
    SharesReply = 5,
    SearchRequest = 8,
    SearchReply = 9,
    UserInfoRequest = 15,
    UserInfoReply = 16,
    FolderContentsRequest = 36,
    FolderContentsReply = 37,
    TransferRequest = 40,
    TransferReply = 41,
    UploadPlacehold = 42,
    QueueDownload = 43,
    PlaceInQueueReply = 44,
    UploadFailed = 46,
    QueueFailed = 50,
    PlaceInQueueRequest = 51,
    UploadQueueNotification = 52,
    Unknown,
}

impl From<u32> for MessageCode {
    fn from(code: u32) -> Self {
        match code {
            4 => MessageCode::SharesRequest,
            5 => MessageCode::SharesReply,
            8 => MessageCode::SearchRequest,
            9 => MessageCode::SearchReply,
            15 => MessageCode::UserInfoRequest,
            16 => MessageCode::UserInfoReply,
            36 => MessageCode::FolderContentsRequest,
            37 => MessageCode::FolderContentsReply,
            40 => MessageCode::TransferRequest,
            41 => MessageCode::TransferReply,
            42 => MessageCode::UploadPlacehold,
            43 => MessageCode::QueueDownload,
            44 => MessageCode::PlaceInQueueReply,
            46 => MessageCode::UploadFailed,
            50 => MessageCode::QueueFailed,
            51 => MessageCode::PlaceInQueueRequest,
            52 => MessageCode::UploadQueueNotification,
            _ => MessageCode::Unknown,
        }
    }
}

pub(crate) const PEER_MSG_HEADER_LEN: u32 = 8;

#[derive(Debug)]
pub enum PeerRequestPacket {
    Message(PeerRequest),
    ConnectionMessage(PeerConnectionMessage),
    // TODO :
    // DistributedMessage(PeerConnectionMessage),
    None,
}

#[derive(Debug)]
pub enum PeerResponsePacket {
    Message(PeerResponse),
    ConnectionMessage(PeerConnectionMessage),
    None,
}
