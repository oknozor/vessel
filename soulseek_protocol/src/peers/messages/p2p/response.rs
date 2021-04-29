use std::io::Cursor;

use bytes::Buf;

use crate::frame::ParseBytes;
use crate::peers::messages::p2p::folder_content::FolderContentsRequest;
use crate::peers::messages::p2p::search::SearchReply;
use crate::peers::messages::p2p::shared_directories::SharedDirectories;
use crate::peers::messages::p2p::transfer::{
    PlaceInQueueReply, PlaceInQueueRequest, QueueFailed, TransferReply, TransferRequest,
    UploadFailed,
};
use crate::peers::messages::p2p::user_info::UserInfo;
use crate::peers::messages::p2p::{PeerMessageCode, PeerMessageHeader, PEER_MSG_HEADER_LEN};
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
    QueueDownload { filename: String },
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
            PeerResponse::QueueDownload { .. } => "QueueDownload",
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
            PeerMessageCode::SharesRequest => Ok(PeerResponse::SharesRequest),
            PeerMessageCode::SharesReply => {
                SharedDirectories::parse(src).map(PeerResponse::SharesReply)
            }
            PeerMessageCode::SearchRequest => todo!(),
            PeerMessageCode::SearchReply => SearchReply::parse(src).map(PeerResponse::SearchReply),
            PeerMessageCode::UserInfoRequest => Ok(PeerResponse::UserInfoRequest),
            PeerMessageCode::UserInfoReply => UserInfo::parse(src).map(PeerResponse::UserInfoReply),
            PeerMessageCode::FolderContentsRequest => {
                FolderContentsRequest::parse(src).map(PeerResponse::FolderContentsRequest)
            }
            PeerMessageCode::FolderContentsReply => todo!(),
            PeerMessageCode::TransferRequest => {
                TransferRequest::parse(src).map(PeerResponse::TransferRequest)
            }
            PeerMessageCode::TransferReply => todo!(),
            PeerMessageCode::UploadPlacehold => todo!(),
            PeerMessageCode::QueueUpload => {
                error!("QUEUE UPLOAD NOT IMPLEMENTED");
                Ok(PeerResponse::UserInfoRequest)
            }
            PeerMessageCode::PlaceInQueueReply => todo!(),
            PeerMessageCode::UploadFailed => todo!(),
            PeerMessageCode::QueueFailed => QueueFailed::parse(src).map(PeerResponse::QueueFailed),
            PeerMessageCode::PlaceInQueueRequest => {
                error!("Place in queue request parsing not implemented");
                Ok(PeerResponse::UserInfoRequest)
            }
            PeerMessageCode::UploadQueueNotification => todo!(),
            PeerMessageCode::Unknown => {
                warn!("Unknown message from peer : \n{:?}", src);
                Ok(PeerResponse::Unknown)
            }
        }
    }
}
