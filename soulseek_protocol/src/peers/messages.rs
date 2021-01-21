use crate::frame::{read_string, write_string, ToBytes};
use crate::message_common::ConnectionType;
use crate::peers::messages::InitMessageCode::{PeerInit, PierceFireWall, Unknown};
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

pub(crate) const CONNECTION_MSG_HEADER_LEN: u32 = 5;
pub(crate) const PEER_MSG_HEADER_LEN: u32 = 8;

#[derive(Debug)]
pub struct ConnectionMessageHeader {
    pub(crate) code: InitMessageCode,
    pub(crate) message_len: usize,
}

#[derive(Debug)]
pub struct PeerMessageHeader {
    pub(crate) code: MessageCode,
    pub(crate) message_len: usize,
}

#[derive(Debug)]
pub enum PeerPacket {
    Message(PeerMessage),
    ConnectionMessage(PeerConnectionMessage),
    None,
}

impl ConnectionMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u8();
        let code = InitMessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 1;

        Ok(Self { message_len, code })
    }
}

impl PeerMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u32_le();
        let code = MessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 4;

        Ok(Self { message_len, code })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum InitMessageCode {
    PierceFireWall = 0,
    PeerInit = 1,
    Unknown,
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

impl From<u8> for InitMessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => PierceFireWall,
            1 => PeerInit,
            _ => Unknown,
        }
    }
}

#[derive(Debug)]
pub enum PeerConnectionMessage {
    PierceFirewall(u32),
    PeerInit {
        username: String,
        connection_type: ConnectionType,
        token: u32,
    },
}

/// TODO
#[derive(Debug)]
pub enum PeerMessage {
    SharesRequest,
    SharesReply,
    SearchRequest,
    SearchReply,
    UserInfoRequest,
    UserInfoReply,
    FolderContentsRequest,
    FolderContentsReply,
    TransferRequest,
    TransferReply,
    UploadPlaceholder,
    QueueDownload,
    PlaceInQueueReply,
    UploadFailed,
    QueueFailed,
    PlaceInQueueRequest,
    UploadQueueNotification,
    Unknown,
}

#[async_trait]
impl ToBytes for PeerConnectionMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            PeerConnectionMessage::PierceFirewall(token) => {
                buffer.write_u32_le(CONNECTION_MSG_HEADER_LEN).await?;
                buffer.write_u8(InitMessageCode::PierceFireWall as u8).await?;
                buffer.write_u32_le(*token).await?;
            }
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                let username_len = username.bytes().len() as u32 + 4;
                let connection_type_len = connection_type.bytes().len() as u32 + 4;

                buffer.write_u32_le(1 + username_len + connection_type_len + 4).await?;
                buffer.write_u8(InitMessageCode::PeerInit as u8).await?;
                write_string(&username, buffer).await?;
                write_string(connection_type.as_ref(), buffer).await?;

                buffer.write_u32_le(*token).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ToBytes for PeerMessage {
    async fn write_to_buf(&self, buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>) -> tokio::io::Result<()> {
        match self {
            PeerMessage::SharesRequest => {}
            PeerMessage::SharesReply => {}
            PeerMessage::SearchRequest => {}
            PeerMessage::SearchReply => {}
            PeerMessage::UserInfoRequest => {
                buffer.write_u32_le(4).await?;
                buffer.write_u32_le(MessageCode::UserInfoRequest as u32).await?;
            }
            PeerMessage::UserInfoReply => {}
            PeerMessage::FolderContentsRequest => {}
            PeerMessage::FolderContentsReply => {}
            PeerMessage::TransferRequest => {}
            PeerMessage::TransferReply => {}
            PeerMessage::UploadPlaceholder => {}
            PeerMessage::QueueDownload => {}
            PeerMessage::PlaceInQueueReply => {}
            PeerMessage::UploadFailed => {}
            PeerMessage::QueueFailed => {}
            PeerMessage::PlaceInQueueRequest => {}
            PeerMessage::UploadQueueNotification => {}
            PeerMessage::Unknown => {}
        }

        Ok(())
    }
}

impl PeerMessage {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<PeerMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < PEER_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = PeerMessageHeader::read(src)?;
        debug!("HEADER : {:?}", header);
        println!("remaning < msg len : {}", src.remaining() < header.message_len);

        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(PEER_MSG_HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &PeerMessageHeader,
    ) -> std::io::Result<Self> {
        info!("PARSING INCOMING PEER MESSAGE ");
        let message = match header.code {
            MessageCode::SharesRequest => PeerMessage::SharesRequest,
            MessageCode::SharesReply => PeerMessage::SharesReply,
            MessageCode::SearchRequest => PeerMessage::SearchRequest,
            MessageCode::SearchReply => PeerMessage::SearchReply,
            MessageCode::UserInfoRequest => PeerMessage::UserInfoRequest,
            MessageCode::UserInfoReply => PeerMessage::UserInfoReply,
            MessageCode::FolderContentsRequest => PeerMessage::FolderContentsRequest,
            MessageCode::FolderContentsReply => PeerMessage::FolderContentsReply,
            MessageCode::TransferRequest => PeerMessage::TransferRequest,
            MessageCode::TransferReply => PeerMessage::TransferReply,
            MessageCode::UploadPlacehold => PeerMessage::UploadPlaceholder,
            MessageCode::QueueDownload => PeerMessage::QueueDownload,
            MessageCode::PlaceInQueueReply => PeerMessage::PlaceInQueueReply,
            MessageCode::UploadFailed => PeerMessage::UploadFailed,
            MessageCode::QueueFailed => PeerMessage::QueueFailed,
            MessageCode::PlaceInQueueRequest => PeerMessage::PlaceInQueueRequest,
            MessageCode::UploadQueueNotification => PeerMessage::UploadQueueNotification,
            MessageCode::Unknown => PeerMessage::Unknown,
        };

        Ok(message)
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerMessage::SharesRequest => "SharesRequest",
            PeerMessage::SharesReply => "SharesReply",
            PeerMessage::SearchRequest => "SearchRequest",
            PeerMessage::SearchReply => "SearchReply",
            PeerMessage::UserInfoRequest => "UserInfoRequest",
            PeerMessage::UserInfoReply => "UserInfoReply",
            PeerMessage::FolderContentsRequest => "FolderContentsRequest",
            PeerMessage::FolderContentsReply => "FolderContentsReply",
            PeerMessage::TransferRequest => "TransferRequest",
            PeerMessage::TransferReply => "TransferReply",
            PeerMessage::UploadPlaceholder => "UploadPlaceholder",
            PeerMessage::QueueDownload => "QueueDownload",
            PeerMessage::PlaceInQueueReply => "PlaceInQueueReply",
            PeerMessage::UploadFailed => "UploadFailed",
            PeerMessage::QueueFailed => "QueueFailed",
            PeerMessage::PlaceInQueueRequest => "PlaceInQueueRequest",
            PeerMessage::UploadQueueNotification => "UploadQueueNotification",
            PeerMessage::Unknown => "Unknown",
        }
    }
}

impl PeerConnectionMessage {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<ConnectionMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < CONNECTION_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = ConnectionMessageHeader::read(src)?;
        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(CONNECTION_MSG_HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &ConnectionMessageHeader,
    ) -> std::io::Result<Self> {
        let message = match header.code {
            InitMessageCode::PierceFireWall => {
                PeerConnectionMessage::PierceFirewall(src.get_u32_le())
            }
            InitMessageCode::PeerInit => PeerConnectionMessage::PeerInit {
                username: read_string(src)?,
                connection_type: ConnectionType::from(read_string(src)?),
                token: src.get_u32_le(),
            },
            InitMessageCode::Unknown => panic!("Unkown message kind, code"),
        };

        Ok(message)
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerConnectionMessage::PierceFirewall(_) => "PierceFirewall",
            PeerConnectionMessage::PeerInit { .. } => "PeerInit",
        }
    }
}
