use std::io::Cursor;

use bytes::Buf;

use crate::frame::{read_string, ParseBytes};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStatus {
    username: String,
    status: Status,
    privileged: bool,
}

impl ParseBytes for UserStatus {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let status = Status::from(src.get_u32_le());
        let privileged = src.get_u8() == 1;

        Ok(UserStatus {
            username,
            status,
            privileged,
        })
    }
}

#[repr(u32)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Offline = 0,
    Away = 1,
    Online = 2,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserList(Users);

impl ParseBytes for UserList {
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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let code = src.get_u8();
        let username = read_string(src)?;
        match code {
            0 => Ok(UserAdded::NotFound { username }),
            1 => {
                let status = src.get_u32_le();
                let average_speed = src.get_u32_le();
                let download_number = src.get_u64_le();
                let files = src.get_u32_le();
                let dirs = src.get_u32_le();
                let country_code = read_string(src)?;

                Ok(UserAdded::Ok {
                    username,
                    status,
                    average_speed,
                    download_number,
                    files,
                    dirs,
                    country_code,
                })
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    username: String,
    average_speed: u32,
    download_number: u64,
    files: u32,
    dirs: u32,
}

impl ParseBytes for UserStats {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let average_speed = src.get_u32_le();
        let download_number = src.get_u64_le();
        let files = src.get_u32_le();
        let dirs = src.get_u32_le();

        Ok(Self {
            username,
            average_speed,
            download_number,
            files,
            dirs,
        })
    }
}

// FIXME : changed, must break
#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    average_speed: u32,
    download_number: u64,
    files: u32,
    dirs: u32,
}

impl ParseBytes for UserData {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let average_speed = src.get_u32_le();
        let download_number = src.get_u64_le();
        let files = src.get_u32_le();
        let dirs = src.get_u32_le();

        Ok(Self {
            average_speed,
            download_number,
            files,
            dirs,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersWithStatus {
    users: Vec<UserWithStatus>,
}

impl ParseBytes for UsersWithStatus {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let users_nth = src.get_u32_le();
        let mut users = vec![];
        for _ in 0..users_nth {
            users.push(UserWithStatus::parse(src)?)
        }

        Ok(Self { users })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithStatus {
    username: String,
    status: Status,
}

impl ParseBytes for UserWithStatus {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;
        let status = Status::from(src.get_u32_le());

        Ok(Self { username, status })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemSimilarUsers {
    item: String,
    users: Vec<UserWithStatus>,
}

impl ParseBytes for ItemSimilarUsers {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let item = read_string(src)?;
        let users_nth = src.get_u32_le();
        let mut users = Vec::with_capacity(users_nth as usize);

        for _ in 0..users_nth {
            users.push(UserWithStatus::parse(src)?);
        }

        Ok(Self { item, users })
    }
}
