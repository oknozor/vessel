use std::io::Cursor;

use bytes::Buf;

use crate::frame::ParseBytes;
use crate::peers::messages::p2p::folder_content::FolderContentsRequest;
use crate::peers::messages::p2p::search::SearchReply;
use crate::peers::messages::p2p::shared_directories::SharedDirectories;
use crate::peers::messages::p2p::transfer::{
    PlaceInQueueReply, PlaceInQueueRequest, QueueDownload, QueueFailed, TransferReply,
    TransferRequest, UploadFailed,
};
use crate::peers::messages::p2p::user_info::UserInfo;
use crate::peers::messages::p2p::{MessageCode, PeerMessageHeader, PEER_MSG_HEADER_LEN};
use crate::SlskError;

#[derive(Debug)]
pub enum PeerResponse {
    SharesRequest,
    SharesReply(SharedDirectories),
    SearchReply(SearchReply),
    UserInfoRequest,
    UserInfoReply(UserInfo),
    FolderContentsRequest(FolderContentsRequest),
    FolderContentsReply(SharedDirectories),
    TransferRequest(TransferRequest),
    TransferReply(TransferReply),
    UploadPlaceholder,
    QueueDownload(QueueDownload),
    PlaceInQueueReply(PlaceInQueueReply),
    UploadFailed(UploadFailed),
    QueueFailed(QueueFailed),
    PlaceInQueueRequest(PlaceInQueueRequest),
    UploadQueueNotification,
    Unknown,
}

impl PeerResponse {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<PeerMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message header
        if src.remaining() < PEER_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }
        // Check if the buffer contains the full message already
        let header = PeerMessageHeader::read(src)?;

        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            Ok(header)
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerResponse::SharesRequest => "SharesRequest",
            PeerResponse::SharesReply(_) => "SharesReply",
            PeerResponse::SearchReply(_) => "SearchReply",
            PeerResponse::UserInfoRequest => "UserInfoRequest",
            PeerResponse::UserInfoReply(_) => "UserInfoReply",
            PeerResponse::FolderContentsRequest(_) => "FolderContentsRequest",
            PeerResponse::FolderContentsReply(_) => "FolderContentsReply",
            PeerResponse::TransferRequest(_) => "TransferRequest",
            PeerResponse::TransferReply(_) => "TransferReply",
            PeerResponse::UploadPlaceholder => "UploadPlaceholder",
            PeerResponse::QueueDownload(_) => "QueueDownload",
            PeerResponse::PlaceInQueueReply(_) => "PlaceInQueueReply",
            PeerResponse::UploadFailed(_) => "UploadFailed",
            PeerResponse::QueueFailed(_) => "QueueFailed",
            PeerResponse::PlaceInQueueRequest(_) => "PlaceInQueueRequest",
            PeerResponse::UploadQueueNotification => "UploadQueueNotification",
            PeerResponse::Unknown => "Unknown",
        }
    }

    #[instrument(level = "debug", skip(src))]
    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &PeerMessageHeader,
    ) -> std::io::Result<Self> {
        match header.code {
            MessageCode::SharesRequest => Ok(PeerResponse::SharesRequest),
            MessageCode::SharesReply => {
                SharedDirectories::parse(src).map(PeerResponse::SharesReply)
            }
            MessageCode::SearchRequest => todo!(),
            MessageCode::SearchReply => SearchReply::parse(src).map(PeerResponse::SearchReply),
            MessageCode::UserInfoRequest => Ok(PeerResponse::UserInfoRequest),
            MessageCode::UserInfoReply => UserInfo::parse(src).map(PeerResponse::UserInfoReply),
            MessageCode::FolderContentsRequest => {
                FolderContentsRequest::parse(src).map(PeerResponse::FolderContentsRequest)
            }
            MessageCode::FolderContentsReply => todo!(),
            MessageCode::TransferRequest => todo!(),
            MessageCode::TransferReply => todo!(),
            MessageCode::UploadPlacehold => todo!(),
            MessageCode::QueueDownload => todo!(),
            MessageCode::PlaceInQueueReply => todo!(),
            MessageCode::UploadFailed => todo!(),
            MessageCode::QueueFailed => todo!(),
            MessageCode::PlaceInQueueRequest => todo!(),
            MessageCode::UploadQueueNotification => todo!(),
            MessageCode::Unknown => Ok(PeerResponse::Unknown),
        }
    }
}
