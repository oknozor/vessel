use std::net::Ipv4Addr;
use crate::server_message::ParseBytes;
use std::io::Cursor;
use bytes::Buf;
use crate::read_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStatus {
    username: String,
    status: Status,
    privileged: bool,
}

impl ParseBytes for UserStatus {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        unimplemented!()
    }
}

#[repr(u32)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Offline = 0,
    Online = 1,
    Away = 2,
    Unknown,
}

impl From<u32> for Status {
    fn from(value: u32) -> Self {
        match value {
            0 => Status::Offline,
            1 => Status::Online,
            2 => Status::Away,
            _ => Status::Unknown,
        }
    }
}

type Users = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserList(Users);

impl ParseBytes for UserList {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
            let number_of_users = src.get_u32_le();
            let mut users = Vec::with_capacity(number_of_users as usize);

            for _ in 0..number_of_users {
                let username = read_string(src)?;
                users.push(username)
            }

            Ok(Self(users))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerAddress {
    username: String,
    ip: Ipv4Addr,
    port: u32,
}

impl ParseBytes for PeerAddress {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserAdded {
    Ok {
        username: String,
        status: u32,
        average_speed: u32,
        download_number: u64,
        files: u32,
        dirs: u32,
        country_code: String,
    },
    NotFound {
        username: String,
    },
}

impl ParseBytes for UserAdded {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    username: String,
    average_speed: u32,
    download_number: u64,
    files: u32,
    dirs: u32,
    country_code: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    average_speed: i32,
    download_number: u32,
    files: u32,
    dirs: u32,
    privileged: u32,
}

impl ParseBytes for UserData {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let average_speed = src.get_i32_le();
        let download_number = src.get_u32_le();
        let files = src.get_u32_le();
        let dirs = src.get_u32_le();
        let privileged = src.get_u32_le();

        Ok(Self {
            average_speed,
            download_number,
            files,
            dirs,
            privileged,
        })
    }
}
