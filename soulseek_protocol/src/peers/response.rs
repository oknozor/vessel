use crate::frame::read_string;
use crate::peers::messages::{
    FolderContentsReply, MessageCode, PeerMessageHeader, PlaceInQueueReply, PlaceInQueueRequest,
    QueueDownload, QueueFailed, SearchReply, SearchRequest, TransferReply, TransferRequest,
    UploadFailed, UserInfo, PEER_MSG_HEADER_LEN,
};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug)]
pub enum PeerResponse {
    SharesRequest,
    SharesReply,
    SearchRequest(SearchRequest),
    SearchReply(SearchReply),
    UserInfoRequest,
    UserInfoReply(UserInfo),
    FolderContentsRequest(FolderContentsReply),
    FolderContentsReply(FolderContentsReply),
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
            // discard header data
            src.set_position(0);
            src.advance(PEER_MSG_HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerResponse::SharesRequest => "SharesRequest",
            PeerResponse::SharesReply => "SharesReply",
            PeerResponse::SearchRequest(_) => "SearchRequest",
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

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &PeerMessageHeader,
    ) -> std::io::Result<Self> {
        let message = match header.code {
            MessageCode::SharesRequest => todo!(),
            MessageCode::SharesReply => todo!(),
            MessageCode::SearchRequest => todo!(),
            MessageCode::SearchReply => todo!(),
            MessageCode::UserInfoRequest => todo!(),
            MessageCode::UserInfoReply => {
                println!("{:?}", src);
                let description = read_string(src)?;
                println!("{}", description);
                let has_picture = src.get_u8() != 0;
                println!("{}", has_picture);
                let picture = if has_picture {
                    read_string(src).ok()
                } else {
                    None
                };
                let total_upload = src.get_u32_le();
                println!("{}", total_upload);
                let queue_size = src.get_u32_le();
                println!("{}", queue_size);
                let slots_free = src.get_u8() == 1;
                println!("{}", slots_free);

                PeerResponse::UserInfoReply(UserInfo {
                    description,
                    picture,
                    total_upload,
                    queue_size,
                    slots_free,
                })
            }
            MessageCode::FolderContentsRequest => todo!(),
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
            MessageCode::Unknown => PeerResponse::Unknown,
        };

        Ok(message)
    }
}
