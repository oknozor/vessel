use std::io;
use std::net::Ipv4Addr;
use std::path::Path;

use crate::peers::messages::p2p::shared_directories::{Directory, File, SharedDirectories};
use crate::server::messages::peer::{Peer, PeerConnectionRequest};
use crate::settings::CONFIG;

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
    pub fn get_address_with_port(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
    pub fn get_address(&self) -> String {
        self.ip.to_string()
    }
}

impl SharedDirectories {
    pub fn from_config() -> io::Result<Self> {
        let paths = &CONFIG.shared_directories;
        let mut shared_directories = SharedDirectories { dirs: vec![] };

        for path in paths {
            visit_dir(path, &mut shared_directories.dirs)?;
        }

        Ok(shared_directories)
    }
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