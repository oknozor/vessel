use std::net::Ipv4Addr;
use crate::settings::CONFIG;
use std::path::PathBuf;
use std::io;
use std::sync::{Arc, Mutex};
use crate::peers::messages::shared_directories::{SharedDirectories, File, Directory};

#[derive(Clone)]
pub struct Database {
    inner: sled::Db,
}

lazy_static! {
    pub static ref SHARED_DIRS: Arc<Mutex<SharedDirectories>> = Arc::new(Mutex::new(SharedDirectories::from_config().unwrap()));
}

impl SharedDirectories {
    pub fn from_config() -> io::Result<Self> {
        let paths = &CONFIG.shared_directories;
        let mut shared_directories = SharedDirectories {
            dirs: vec![]
        };

        for path in paths {
            visit_dir(path, &mut shared_directories.dirs)?;
        }

        Ok(shared_directories)
    }
}

fn visit_dir(path: &PathBuf, dirs: &mut Vec<Directory>) -> io::Result<()> {
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
                    extension: entry.file_name().to_str().unwrap().split('.').last().unwrap().to_string(),
                    // TODO
                    attributes: vec![]
                })
            }

            dirs.push(dir);
        }
    } else {
        error!("{:?} is not a directory", path)
    }

    Ok(())
}


impl Database {
    pub fn new() -> Database {
        Database {
            inner: sled::open("vessel_db").unwrap(),
        }
    }

    pub fn insert_peer(&self, username: &str, address: Ipv4Addr) -> sled::Result<()> {
        debug!("Writing peer {}@{} to db:", username, address.to_string());
        self.inner.open_tree("users")?
            .insert(username.as_bytes(), address.to_string().as_bytes())
            .map(|res| ())
    }

    pub fn get_peer_by_name(&self, username: &str) -> Option<String> {
        self.inner.open_tree("users").unwrap()
            .get(username)
            .ok()
            .flatten()
            .map(|res| String::from_utf8(res.to_vec()).expect("UTF-8 parsing error"))
    }

    pub fn find_all(&self) -> Vec<(String, String)> {
        self.inner.open_tree("users").unwrap()
            .iter()
            .map(|res| res.expect("database error"))
            .map(|(k, v)| (String::from_utf8(k.to_vec()).unwrap(),String::from_utf8(v.to_vec()).unwrap()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::database::Database;
    use std::net::Ipv4Addr;

    #[test]
    fn should_open_db() {
        let db = Database::new();
        assert!(db.insert_peer("toto", Ipv4Addr::new(127, 0, 0, 1)).is_ok())
    }

    #[test]
    fn should_get_peer() {
        let db = Database::new();
        db.insert_peer("toto", Ipv4Addr::new(127, 0, 0, 1)).unwrap();
        let address = db.get_peer_by_name("toto").unwrap();
        assert_eq!(address.to_string(), "127.0.0.1");
    }
}
