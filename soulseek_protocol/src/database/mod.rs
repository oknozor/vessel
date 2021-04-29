use crate::peers::messages::p2p::shared_directories::{Directory, File, SharedDirectories};
use crate::peers::messages::p2p::transfer::TransferRequest;
use crate::settings::CONFIG;
use std::io;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Database {
    inner: sled::Db,
}

lazy_static! {
    pub static ref SHARED_DIRS: Arc<Mutex<SharedDirectories>> =
        Arc::new(Mutex::new(SharedDirectories::from_config().unwrap()));
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

impl Default for Database {
    fn default() -> Self {
        Database {
            inner: sled::open("vessel_db").unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadDbEntry {
    pub file_name: String,
    pub ticket: u32,
    pub file_size: u64,
    pub progress: u64,
}

impl Database {
    pub fn insert_peer(&self, username: &str, address: Ipv4Addr) -> sled::Result<()> {
        debug!("Writing peer {}@{} to db:", username, address.to_string());
        self.inner
            .open_tree("users")?
            .insert(username.as_bytes(), address.to_string().as_bytes())
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
            .map(|res| {
                let json = String::from_utf8(res.to_vec()).expect("UTF-8 parsing error");
                serde_json::from_str(&json).expect("Serde error")
            })
    }

    pub fn get_peer_by_name(&self, username: &str) -> Option<String> {
        self.inner
            .open_tree("users")
            .unwrap()
            .get(username)
            .ok()
            .flatten()
            .map(|res| String::from_utf8(res.to_vec()).expect("UTF-8 parsing error"))
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
    use crate::database::{Database, DownloadDbEntry};
    use std::net::Ipv4Addr;

    #[test]
    fn should_open_db() {
        let db = Database::default();
        assert!(db.insert_peer("toto", Ipv4Addr::new(127, 0, 0, 1)).is_ok())
    }

    #[test]
    fn should_get_peer() {
        let db = Database::default();
        db.insert_peer("toto", Ipv4Addr::new(127, 0, 0, 1)).unwrap();
        let address = db.get_peer_by_name("toto").unwrap();
        assert_eq!(address.to_string(), "127.0.0.1");
    }

    #[test]
    fn should_convert_download_entry() {
        let entry = DownloadDbEntry {
            file_name: "test".to_string(),
            ticket: 0,
            file_size: 3,
            progress: 2,
        };

        let bytes = entry.as_bytes();
    }
}
