use std::io::Cursor;
use tokio::io::BufWriter;
use tokio::net::TcpStream;

pub trait ParseBytes {
    type Output;
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output>;
}

#[async_trait]
pub trait ToBytes {
    async fn write_to_buf(&self, buffer: &mut BufWriter<TcpStream>) -> tokio::io::Result<()>;
}
