use crate::{
    frame::{write_string, ToBytes, STR_LENGTH_PREFIX},
    server::MessageCode,
};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

// FIXME : what does this message mean ?
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCommand {
    string1: String,
    string2: Vec<String>,
}

#[async_trait]
impl ToBytes for AdminCommand {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let str2_len: u32 = self
            .string2
            .iter()
            .map(|s| s.bytes().len() as u32 + STR_LENGTH_PREFIX)
            .sum();
        let len = STR_LENGTH_PREFIX
            + self.string1.bytes().len() as u32
            + self.string2.len() as u32
            + str2_len;

        buffer.write_u32_le(len).await?;
        buffer
            .write_u32_le(MessageCode::AdminCommand as u32)
            .await?;
        write_string(&self.string1, buffer).await?;
        buffer.write_u32_le(self.string2.len() as u32).await?;
        for s in &self.string2 {
            write_string(&s, buffer).await?
        }
        Ok(())
    }
}
