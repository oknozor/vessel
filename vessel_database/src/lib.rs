#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tracing;

use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::Serialize;

use entity::{Entity, shared_dirs::get_shared_directories};
use soulseek_protocol::peers::p2p::{
    shared_directories::SharedDirectories,
};

pub mod entity;
pub mod settings;

#[derive(Clone, Debug)]
pub struct Database {
    inner: sled::Db,
}

lazy_static! {
    pub static ref SHARED_DIRS: Arc<Mutex<SharedDirectories>> =
        Arc::new(Mutex::new(get_shared_directories().unwrap()));
}

impl Default for Database {
    fn default() -> Self {
        Database {
            inner: sled::open("vessel_db").unwrap(),
        }
    }
}

impl Database {
    pub fn insert<T>(&self, entity: &T) -> sled::Result<()>
        where T: Sized + Entity + Serialize {
        let json = serde_json::to_string(entity).expect("Serialization error");

        debug!("Inserting entity {}:", json);

        self.inner
            .open_tree(T::COLLECTION)?
            .insert(entity.get_key(), json.as_bytes())
            .map(|_res| ())
    }

    pub fn get_all<T>(&self) -> Vec<T>
        where T: Entity + DeserializeOwned {
        self.inner
            .open_tree(T::COLLECTION)
            .unwrap()
            .iter()
            .map(|res| res.expect("database error"))
            .map(|(_k, v)| String::from_utf8(v.to_vec()).unwrap())
            .map(|entity_string| serde_json::from_str(entity_string.as_str()))
            .flat_map(Result::ok)
            .collect()
    }

    pub fn get_by_key<T>(&self, key: &str) -> Option<T> where T: Entity + DeserializeOwned {
        self.inner
            .open_tree(T::COLLECTION)
            .unwrap()
            .get(key)
            .ok()
            .flatten()
            .map(|data| data.to_vec())
            .map(String::from_utf8)
            .map(Result::unwrap)
            .map(|raw_data| serde_json::from_str(&raw_data))
            .map(Result::unwrap)
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::Database;
    use crate::entity::peer::PeerEntity;

    #[test]
    fn should_open_db() {
        let db = Database::default();
        assert!(db
            .insert(&PeerEntity {
                username: "toto".to_string(),
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 0,
            })
            .is_ok())
    }

    #[test]
    fn should_get_peer() {
        let db = Database::default();
        db.insert(&PeerEntity {
            username: "toto".to_string(),
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 0,
        })
            .unwrap();
        let peer = db.get_by_key::<PeerEntity>("toto").unwrap();

        assert_eq!(peer.username, "toto");
        assert_eq!(peer.get_address().to_string(), "127.0.0.1:0");
        assert_eq!(peer.port, 0);
    }

    #[test]
    fn should_get_all_peers() {
        let db = Database::default();
        db.insert(&PeerEntity {
            username: "alfred".to_string(),
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 2222,
        })
            .unwrap();
        let all_peers = db.get_all::<PeerEntity>();

        assert!(!all_peers.is_empty());
    }
}
