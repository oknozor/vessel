use std::io::Cursor;
use std::net::{SocketAddr};
use std::path::Path;

use bytes::{Buf, BytesMut};
use eyre::Result;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc::Sender;
use tokio::{io::AsyncReadExt, net::TcpStream};

use soulseek_protocol::frame::ToBytes;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::download::DownloadProgress;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::{ProtocolHeader, ProtocolMessage, SlskError};
use vessel_database::Database;

#[derive(Debug)]
pub struct PeerConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub(crate) connection_type: ConnectionType,
}

impl PeerConnection {
    pub(crate) fn new(socket: TcpStream) -> PeerConnection {
        PeerConnection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
            connection_type: ConnectionType::HandShake,
        }
    }

    pub(crate) async fn read_message<T: ProtocolMessage>(
        &mut self,
    ) -> soulseek_protocol::Result<T> {
        loop {
            match self.parse_message::<T>() {
                Ok(Some(message)) => {
                    return Ok(message);
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.stream.buffer().is_empty() {
                    // No message
                } else {
                    return Err(SlskError::ConnectionResetByPeer);
                }
            }
        }
    }

    /// Send a [`PeerMessage`] the soulseek server, using `[ToBytes]` to write to the buffer.
    #[instrument(level = "trace", skip(self))]
    pub(crate) async fn write_request(
        &mut self,
        message: PeerRequestPacket,
    ) -> tokio::io::Result<()> {
        match message {
            PeerRequestPacket::Message(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Request sent to peer {:?}", message);
            }
            PeerRequestPacket::ConnectionMessage(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Connection request sent to peer {:?}", message);
            }
            _ => unreachable!(),
        }

        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// Parse the connection message with a 5 bytes prefix if we are trying to establish connection to a peer,
    /// otherwise consume the peer message plus the 8 bytes header prefix.
    fn consume(&mut self, message_len: usize) {
        match self.connection_type {
            ConnectionType::HandShake | ConnectionType::DistributedNetwork => {
                self.buffer.advance(message_len + 5)
            }
            ConnectionType::PeerToPeer => self.buffer.advance(message_len + 8),
            ConnectionType::FileTransfer => {
                unreachable!("Attempt to parse bytes from File Transfert connection ")
            }
        }
    }

    fn parse_message<T: ProtocolMessage>(&mut self) -> soulseek_protocol::Result<Option<T>> {
        use soulseek_protocol::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match T::check(&mut buf) {
            Ok(header) => {
                buf.set_position(T::Header::LEN as u64);
                let connection_message = T::parse(&mut buf, &header)?;

                // consume the message bytes
                self.consume(header.message_len());
                Ok(Some(connection_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_peer_address_with_port(&self) -> Result<SocketAddr, std::io::Error> {
        match &self.stream.get_ref().peer_addr() {
            Ok(address) => Ok(address.to_owned()),
            Err(err) => Err(std::io::Error::from(err.kind())),
        }
    }

    pub(crate) async fn download(
        &mut self,
        db: &Database,
        progress_sender: Sender<DownloadProgress>,
        user_name: String,
    ) -> Result<()> {
        let address = self.get_peer_address_with_port()?.to_string();

        // We need to parse the ticket from the upload connection
        if self.buffer.remaining() < 4 && self.try_read_buffer().await? == 0 {
            return Err(eyre!("Empty buffer on download init"));
        }

        let mut cursor = Cursor::new(&mut self.buffer);
        info!("Got incoming upload connection from {}", address);
        let ticket = cursor.get_u32_le();
        let download_entry = db.get_download(ticket);

        if let Some(entry) = download_entry {
            let file_name = &entry.file_name;
            let path = Path::new(file_name).file_name().expect("File name error");

            progress_sender
                .send(DownloadProgress::Init {
                    file_name: file_name.clone(),
                    user_name,
                    ticket,
                })
                .await?;

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(path)
                .await?;

            self.stream.write_u32_le(0).await?;
            self.stream.write_u32_le(0).await?;
            self.stream.write_u32_le(0).await?;
            self.stream.flush().await?;

            info!("Starting to download {}", file_name);
            let mut progress = 0;
            let mut percent_progress = 0;
            let file_size = entry.file_size as usize;

            loop {
                let byte_red = file.write(self.buffer.chunk()).await?;

                let percent = 100 * progress / file_size;

                // Avoid to reprint percent every time the task yield
                if percent > percent_progress {
                    percent_progress = percent;
                    progress_sender
                        .send(DownloadProgress::Progress { ticket, percent })
                        .await?;
                    info!("{}% of {}", percent, file_name);
                }

                self.buffer.advance(byte_red as usize);
                progress += byte_red;

                if progress >= file_size {
                    info!("100% of {}", file_name);
                    file.sync_data().await?;
                    return Ok(());
                }

                if 0 == self.try_read_buffer().await? {
                    info!("Download finished");
                    file.sync_data().await?;
                    return Ok(());
                }
            }
        } else {
            Err(eyre!(SlskError::ConnectionResetByPeer))
        }
    }

    async fn try_read_buffer(&mut self) -> Result<usize> {
        let bytes_red = self.stream.read_buf(&mut self.buffer).await?;
        if 0 == bytes_red {
            if self.buffer.is_empty() {
                info!("Download finished");
                Ok(bytes_red)
            } else {
                Err(eyre!("connection reset by peer"))
            }
        } else {
            Ok(bytes_red)
        }
    }
}

impl Drop for PeerConnection {
    fn drop(&mut self) {
        debug!("Dropping peer connection")
    }
}
