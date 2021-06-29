use std::io::Cursor;

use crate::{MessageCode, ProtocolHeader};
use bytes::Buf;

pub mod download;
pub mod folder_content;
pub mod place_in_queue;
pub mod request;
pub mod response;
pub mod search;
pub mod shared_directories;
pub mod transfer;
pub mod user_info;
mod zlib;

#[derive(Debug)]
pub struct PeerMessageHeader {
    pub(crate) code: PeerMessageCode,
    pub(crate) message_len: usize,
}

impl ProtocolHeader for PeerMessageHeader {
    const LEN: usize = 8;
    type Code = PeerMessageCode;

    fn message_len(&self) -> usize {
        self.message_len
    }

    fn new(message_len: usize, code: Self::Code) -> Self {
        Self { code, message_len }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum PeerMessageCode {
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
    QueueUpload = 43,
    PlaceInQueueReply = 44,
    UploadFailed = 46,
    QueueFailed = 50,
    PlaceInQueueRequest = 51,
    UploadQueueNotification = 52,
    Unknown,
}

impl MessageCode for PeerMessageCode {
    const LEN: usize = 4;

    fn read(src: &mut Cursor<&[u8]>) -> Self {
        let code = src.get_u32_le();
        Self::from(code)
    }
}

impl From<u32> for PeerMessageCode {
    fn from(code: u32) -> Self {
        match code {
            4 => PeerMessageCode::SharesRequest,
            5 => PeerMessageCode::SharesReply,
            8 => PeerMessageCode::SearchRequest,
            9 => PeerMessageCode::SearchReply,
            15 => PeerMessageCode::UserInfoRequest,
            16 => PeerMessageCode::UserInfoReply,
            36 => PeerMessageCode::FolderContentsRequest,
            37 => PeerMessageCode::FolderContentsReply,
            40 => PeerMessageCode::TransferRequest,
            41 => PeerMessageCode::TransferReply,
            42 => PeerMessageCode::UploadPlacehold,
            43 => PeerMessageCode::QueueUpload,
            44 => PeerMessageCode::PlaceInQueueReply,
            46 => PeerMessageCode::UploadFailed,
            50 => PeerMessageCode::QueueFailed,
            51 => PeerMessageCode::PlaceInQueueRequest,
            52 => PeerMessageCode::UploadQueueNotification,
            _ => PeerMessageCode::Unknown,
        }
    }
}
