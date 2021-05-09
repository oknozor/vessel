use std::sync::{Arc, Mutex};

use entities::DownloadDbEntry;

use crate::database::entities::PeerEntity;
use crate::peers::messages::p2p::shared_directories::SharedDirectories;
use crate::peers::messages::p2p::transfer::TransferRequest;

pub mod entities;

#[derive(Clone, Debug)]
pub struct Database {
    inner: sled::Db,
}

lazy_static! {
    pub static ref SHARED_DIRS: Arc<Mutex<SharedDirectories>> =
        Arc::new(Mutex::new(SharedDirectories::from_config().unwrap()));
}

impl Default for Database {
    fn default() -> Self {
        Database {
            inner: sled::open("vessel_db").unwrap(),
        }
    }
}

impl Database {
    pub fn insert_peer(&self, peer: &PeerEntity) -> sled::Result<()> {
        debug!("Writing peer {:?} to db:", peer);

        let json = serde_json::to_string(&peer).expect("Serialization error");

        self.inner
            .open_tree("users")?
            .insert(peer.username.as_bytes(), json.as_bytes())
            .map(|_res| ())
    }

    pub fn insert_download(&self, request: &TransferRequest) -> sled::Result<()> {
        debug!("Writing download progress to db:");

        let entry = DownloadDbEntry {
            file_name: request.filename.clone(),
            ticket: request.ticket,
            file_size: request
                .file_size
                .expect("Accepted transfer request should have a file size"),
            progress: 0,
        };

        let json = serde_json::to_string(&entry).expect("Serialization error");

        self.inner
            .open_tree("downloads")?
            .insert(request.ticket.to_le_bytes(), json.as_bytes())
            .map(|_res| ())
    }

    pub fn get_download(&self, ticket: u32) -> Option<DownloadDbEntry> {
        self.inner
            .open_tree("downloads")
            .unwrap()
            .get(ticket.to_le_bytes())
            .ok()
            .flatten()
            .map(|data| data.to_vec())
            .map(String::from_utf8)
            .map(Result::unwrap)
            .map(|raw| serde_json::from_str(&raw))
            .map(Result::unwrap)
    }

    pub fn get_peer(&self, username: &str) -> Option<PeerEntity> {
        self.inner
            .open_tree("users")
            .unwrap()
            .get(&username)
            .ok()
            .flatten()
            .map(|data| data.to_vec())
            .map(String::from_utf8)
            .map(Result::unwrap)
            .map(|raw| serde_json::from_str(&raw))
            .map(Result::unwrap)
    }

    pub fn find_all(&self) -> Vec<(String, String)> {
        self.inner
            .open_tree("users")
            .unwrap()
            .iter()
            .map(|res| res.expect("database error"))
            .map(|(k, v)| {
                (
                    String::from_utf8(k.to_vec()).unwrap(),
                    String::from_utf8(v.to_vec()).unwrap(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::database::entities::PeerEntity;
    use crate::database::Database;

    #[test]
    fn should_open_db() {
        let db = Database::default();
        assert!(db
            .insert_peer(&PeerEntity {
                username: "toto".to_string(),
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 0,
            })
            .is_ok())
    }

    #[test]
    fn should_get_peer() {
        let db = Database::default();
        db.insert_peer(&PeerEntity {
            username: "toto".to_string(),
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 0,
        })
        .unwrap();
        let peer = db.get_peer("toto").unwrap();

        assert_eq!(peer.username, "toto");
        assert_eq!(peer.get_address().to_string(), "127.0.0.1:0");
        assert_eq!(peer.port, 0);
    }
}
