use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::ToBytes;
use crate::peers::messages::{MessageCode};
use crate::peers::messages::shared_directories::SharedDirectories;
use crate::peers::messages::search::{SearchRequest, SearchReply};
use crate::peers::messages::user_info::UserInfo;
use crate::peers::messages::transfer::*;

/// TODO
#[derive(Debug)]
pub enum PeerRequest {
    SharesRequest,
    SharesReply(SharedDirectories),
    SearchRequest(SearchRequest),
    SearchReply(SearchReply),
    UserInfoRequest,
    UserInfoReply(UserInfo),
    FolderContentsRequest(SharedDirectories),
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

impl PeerRequest {
    pub fn kind(&self) -> &str {
        match self {
            PeerRequest::SharesRequest => "SharesRequest",
            PeerRequest::SharesReply(_) => "SharesReply",
            PeerRequest::SearchRequest(_) => "SearchRequest",
            PeerRequest::SearchReply(_) => "SearchReply",
            PeerRequest::UserInfoRequest => "UserInfoRequest",
            PeerRequest::UserInfoReply(_) => "UserInfoReply",
            PeerRequest::FolderContentsRequest(_) => "FolderContentsRequest",
            PeerRequest::FolderContentsReply(_) => "FolderContentsReply",
            PeerRequest::TransferRequest(_) => "TransferRequest",
            PeerRequest::TransferReply(_) => "TransferReply",
            PeerRequest::UploadPlaceholder => "UploadPlaceholder",
            PeerRequest::QueueDownload(_) => "QueueDownload",
            PeerRequest::PlaceInQueueReply(_) => "PlaceInQueueReply",
            PeerRequest::UploadFailed(_) => "UploadFailed",
            PeerRequest::QueueFailed(_) => "QueueFailed",
            PeerRequest::PlaceInQueueRequest(_) => "PlaceInQueueRequest",
            PeerRequest::UploadQueueNotification => "UploadQueueNotification",
            PeerRequest::Unknown => "Unknown",
        }
    }
}

#[async_trait]
impl ToBytes for PeerRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            PeerRequest::SharesRequest => {
                buffer.write_u32_le(4).await?;
                buffer
                    .write_u32_le(MessageCode::SharesRequest as u32)
                    .await?;
            }
            PeerRequest::SharesReply(shared_dirs) => {
                shared_dirs.write_to_buf(buffer).await?;
            }
            PeerRequest::SearchRequest(search_request) => {
                search_request.write_to_buf(buffer).await?
            }
            PeerRequest::SearchReply(search_reply) => search_reply.write_to_buf(buffer).await?,
            PeerRequest::UserInfoRequest => {
                buffer.write_u32_le(4).await?;
                buffer
                    .write_u32_le(MessageCode::UserInfoRequest as u32)
                    .await?;
            }
            PeerRequest::UserInfoReply(user_info) => user_info.write_to_buf(buffer).await?,
            PeerRequest::FolderContentsRequest(folder_content_request) => {
                folder_content_request.write_to_buf(buffer).await?
            }
            PeerRequest::FolderContentsReply(folder_content_reply) => {
                folder_content_reply.write_to_buf(buffer).await?
            }
            PeerRequest::TransferRequest(transfer_request) => {
                transfer_request.write_to_buf(buffer).await?
            }
            PeerRequest::TransferReply(transfer_reply) => {
                transfer_reply.write_to_buf(buffer).await?
            }
            PeerRequest::UploadPlaceholder => {}
            PeerRequest::QueueDownload(queue_download) => {
                queue_download.write_to_buf(buffer).await?
            }
            PeerRequest::PlaceInQueueReply(place_in_queue_reply) => {
                place_in_queue_reply.write_to_buf(buffer).await?
            }
            PeerRequest::UploadFailed(upload_failed) => upload_failed.write_to_buf(buffer).await?,
            PeerRequest::QueueFailed(queue_failed) => queue_failed.write_to_buf(buffer).await?,
            PeerRequest::PlaceInQueueRequest(place_in_queue_request) => {
                place_in_queue_request.write_to_buf(buffer).await?
            }
            PeerRequest::UploadQueueNotification => {}
            PeerRequest::Unknown => {}
        }

        Ok(())
    }
}
