use std::net::IpAddr;

pub mod download;
pub mod peer;
pub mod shared_dirs;
pub mod upload;

/// A generic insertable entity
pub trait Entity {
    fn get_key(&self) -> Vec<u8>;
    const COLLECTION: &'static str;
}
