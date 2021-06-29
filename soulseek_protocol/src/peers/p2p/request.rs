use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::{
    frame::{write_string, ToBytes},
    peers::p2p::{
        folder_content::FolderContentsRequest, search::SearchReply,
        shared_directories::SharedDirectories, transfer::*, user_info::UserInfo, PeerMessageCode,
    },
};

/// TODO
#[derive(Debug)]
pub enum PeerRequest {
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
    QueueUpload { filename: String },
    PlaceInQueueReply(PlaceInQueueReply),
    UploadFailed(UploadFailed),
    QueueFailed(QueueFailed),
    PlaceInQueueRequest(PlaceInQueueRequest),
    UploadQueueNotification,
    Unknown,
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
                    .write_u32_le(PeerMessageCode::SharesRequest as u32)
                    .await?;
            }
            PeerRequest::SharesReply(shared_dirs) => {
                shared_dirs.write_to_buf(buffer).await?;
            }
            PeerRequest::SearchReply(search_reply) => search_reply.write_to_buf(buffer).await?,
            PeerRequest::UserInfoRequest => {
                buffer.write_u32_le(4).await?;
                buffer
                    .write_u32_le(PeerMessageCode::UserInfoRequest as u32)
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
            PeerRequest::QueueUpload { filename } => {
                write_str_msg(filename, PeerMessageCode::QueueUpload, buffer).await?
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

pub(crate) async fn write_str_msg(
    src: &str,
    code: PeerMessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    let bytes = src.as_bytes();
    let message_len = bytes.len() as u32 + 8;
    buffer.write_u32_le(message_len).await?;
    buffer.write_u32_le(code as u32).await?;
    write_string(src, buffer).await?;
    Ok(())
}
