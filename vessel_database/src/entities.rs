use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

use crate::settings::CONFIG;
use soulseek_protocol::peers::p2p::shared_directories::{Directory, File, SharedDirectories};
use soulseek_protocol::server::peer::{Peer, PeerAddress, PeerConnectionRequest};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerEntity {
    pub username: String,
    pub ip: Ipv4Addr,
    pub(crate) port: u32,
}

impl PeerEntity {
    pub fn new(username: &str, ip: Ipv4Addr, port: u32) -> Self {
        PeerEntity {
            username: username.to_string(),
            ip,
            port,
        }
    }
}

impl From<PeerConnectionRequest> for PeerEntity {
    fn from(request: PeerConnectionRequest) -> Self {
        PeerEntity {
            username: request.username,
            ip: request.ip,
            port: request.port,
        }
    }
}

impl From<PeerAddress> for PeerEntity {
    fn from(peer: PeerAddress) -> Self {
        PeerEntity {
            username: peer.username,
            ip: peer.ip,
            port: peer.port,
        }
    }
}

impl From<Peer> for PeerEntity {
    fn from(peer: Peer) -> Self {
        PeerEntity {
            username: peer.username,
            ip: peer.ip,
            port: peer.port,
        }
    }
}

impl PeerEntity {
    pub fn get_address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::from(self.ip), self.port as u16)
    }
}

pub fn get_shared_directories() -> io::Result<SharedDirectories> {
    let paths = &CONFIG.shared_directories;
    let mut shared_directories = SharedDirectories { dirs: vec![] };

    for path in paths {
        visit_dir(path, &mut shared_directories.dirs)?;
    }

    Ok(shared_directories)
}

fn visit_dir(path: &Path, dirs: &mut Vec<Directory>) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let mut dir = Directory {
                name: path.to_str().unwrap().to_string(),
                files: vec![],
            };

            if entry.path().is_dir() {
                visit_dir(&entry.path(), dirs)?;
            } else {
                let metadata = entry.metadata()?;
                dir.files.push(File {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    size: metadata.len(),
                    extension: entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .split('.')
                        .last()
                        .unwrap()
                        .to_string(),
                    // TODO
                    attributes: vec![],
                })
            }

            dirs.push(dir);
        }
    } else {
        error!("{:?} is not a directory", path)
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadDbEntry {
    pub file_name: String,
    pub ticket: u32,
    pub file_size: u64,
    pub progress: u64,
}
