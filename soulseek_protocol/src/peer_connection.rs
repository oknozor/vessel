use std::io::BufWriter;
use tokio::net::TcpStream;
use bytes::BytesMut;

#[derive(Debug)]
pub struct PeerConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}