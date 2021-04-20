use self::search::SearchRequest;
use crate::frame::{read_string, ParseBytes, ToBytes};
use crate::peers::messages::distributed::DistributedMessageCode::BranchLevel;
use crate::SlskError;
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

pub(crate) const DISTRIBUTED_MSG_HEADER_LEN: u32 = 5;

mod branch;
mod search;

#[derive(Debug)]
pub struct DistributedMessageHeader {
    pub(crate) code: DistributedMessageCode,
    pub(crate) message_len: usize,
}

impl DistributedMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u8();
        let code = DistributedMessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 1;

        Ok(Self { message_len, code })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum DistributedMessageCode {
    Ping = 0,
    SearchRequest = 3,
    BranchLevel = 4,
    BranchRoot = 5,
    ChildDepth = 7,
    ServerSearchRequest = 93,
}

impl From<u8> for DistributedMessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => DistributedMessageCode::Ping,
            3 => DistributedMessageCode::SearchRequest,
            4 => DistributedMessageCode::BranchLevel,
            5 => DistributedMessageCode::BranchRoot,
            7 => DistributedMessageCode::ChildDepth,
            93 => DistributedMessageCode::ServerSearchRequest,
            code => unreachable!("Unexpected distributed message code: {}", code),
        }
    }
}

#[derive(Debug)]
pub enum DistributedMessage {
    Ping,
    SearchRequest(SearchRequest),
    BranchLevel(u32),
    BranchRoot(String),
    ChildDepth(u32),
    ServerSearchRequest,
}

#[async_trait]
impl ToBytes for DistributedMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            DistributedMessage::Ping => {
                write_empty_distributed_msg(DistributedMessageCode::Ping, buffer).await?
            }
            DistributedMessage::SearchRequest(_) => todo!(),
            DistributedMessage::BranchLevel(_) => todo!(),
            DistributedMessage::BranchRoot(_) => todo!(),
            DistributedMessage::ChildDepth(_) => todo!(),
            DistributedMessage::ServerSearchRequest => todo!(),
        }

        Ok(())
    }
}

impl DistributedMessage {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<DistributedMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < DISTRIBUTED_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = DistributedMessageHeader::read(src)?;

        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            Ok(header)
        }
    }

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &DistributedMessageHeader,
    ) -> std::io::Result<Self> {
        match header.code {
            DistributedMessageCode::Ping => Ok(DistributedMessage::Ping),
            DistributedMessageCode::SearchRequest => {
                SearchRequest::parse(src).map(DistributedMessage::SearchRequest)
            }
            DistributedMessageCode::BranchLevel => {
                Ok(DistributedMessage::BranchLevel(src.get_u32_le()))
            }
            DistributedMessageCode::BranchRoot => {
                Ok(DistributedMessage::BranchRoot(read_string(src)?))
            }
            DistributedMessageCode::ChildDepth => {
                Ok(DistributedMessage::ChildDepth(src.get_u32_le()))
            }
            DistributedMessageCode::ServerSearchRequest => {
                Ok(DistributedMessage::ServerSearchRequest)
            }
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            DistributedMessage::Ping => "Ping",
            DistributedMessage::SearchRequest(_) => "SearchRequest",
            DistributedMessage::BranchLevel(_) => "BranchLevel",
            DistributedMessage::BranchRoot(_) => "BranchRoot",
            DistributedMessage::ChildDepth(_) => "ChildDepth",
            DistributedMessage::ServerSearchRequest => "ServerSearchRequest",
        }
    }
}

async fn write_empty_distributed_msg(
    code: DistributedMessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    buffer.write_u32_le(1).await?;
    buffer.write_u8(code as u8).await?;
    Ok(())
}
