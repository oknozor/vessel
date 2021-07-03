use std::io::Cursor;

use crate::{
    frame::ParseBytes,
    peers::p2p::{
        folder_content::FolderContentsRequest,
        search::SearchReply,
        shared_directories::SharedDirectories,
        transfer::{
            PlaceInQueueReply, PlaceInQueueRequest, QueueFailed, TransferReply, TransferRequest,
            UploadFailed,
        },
        user_info::UserInfo,
        PeerMessageCode, PeerMessageHeader,
    },
    ProtocolMessage,
};

#[derive(Debug, Serialize)]
#[serde(untagged)]
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

impl ProtocolMessage for PeerResponse {
    type Header = PeerMessageHeader;

    fn parse(src: &mut Cursor<&[u8]>, header: &Self::Header) -> std::io::Result<Self> {
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
            PeerMessageCode::QueueUpload => todo!(),
            PeerMessageCode::PlaceInQueueReply => todo!(),
            PeerMessageCode::UploadFailed => {
                UploadFailed::parse(src).map(PeerResponse::UploadFailed)
            }
            PeerMessageCode::QueueFailed => QueueFailed::parse(src).map(PeerResponse::QueueFailed),
            PeerMessageCode::PlaceInQueueRequest => todo!(),
            PeerMessageCode::UploadQueueNotification => todo!(),
            PeerMessageCode::Unknown => {
                warn!("Unknown message from peer : \n{:?}", src);
                Ok(PeerResponse::Unknown)
            }
        }
    }
}
