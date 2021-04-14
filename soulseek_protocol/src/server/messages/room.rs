use crate::frame::read_string;
use crate::frame::ParseBytes;
use crate::server::messages::user::{Status, UserData};
use bytes::Buf;
use std::io::Cursor;

type Rooms = Vec<(String, u32)>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RoomList {
    rooms: Rooms,
    owned_private_rooms: Rooms,
    private_rooms: Rooms,
    operated_private_rooms: Vec<String>,
}

impl ParseBytes for RoomList {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let rooms = RoomList::extract_room(src)?;
        let owned_private_rooms = RoomList::extract_room(src)?;
        let private_rooms = RoomList::extract_room(src)?;

        let number_of_rooms = src.get_u32_le();
        let mut operated_private_rooms = Vec::with_capacity(number_of_rooms as usize);

        for _ in 0..number_of_rooms {
            let room_name = read_string(src)?;
            operated_private_rooms.push(room_name)
        }

        Ok(Self {
            rooms,
            owned_private_rooms,
            private_rooms,
            operated_private_rooms,
        })
    }
}

impl RoomList {
    fn extract_room(src: &mut Cursor<&[u8]>) -> std::io::Result<Rooms> {
        let number_of_rooms = src.get_u32_le();
        let mut rooms_names = Vec::with_capacity(number_of_rooms as usize);

        for _ in 0..number_of_rooms {
            rooms_names.push(read_string(src)?)
        }

        let number_of_rooms = src.get_u32_le();
        let mut user_per_room = Vec::with_capacity(number_of_rooms as usize);

        for _ in 0..number_of_rooms {
            user_per_room.push(src.get_u32_le());
        }

        Ok(rooms_names
            .into_iter()
            .zip(user_per_room.into_iter())
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinedRoom {
    room: String,
    username: String,
    status: u32,
    avgspeed: u32,
    downloadnum: u64,
    files: u32,
    dirs: u32,
    slotsfree: u32,
    countrycode: String,
}

impl ParseBytes for UserJoinedRoom {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let room = read_string(src)?;
        let username = read_string(src)?;
        let status = src.get_u32_le();
        let avgspeed = src.get_u32_le();
        let downloadnum = src.get_u64_le();
        let files = src.get_u32_le();
        let dirs = src.get_u32_le();
        let slotsfree = src.get_u32_le();
        let countrycode = read_string(src)?;

        Ok(Self {
            room,
            username,
            status,
            avgspeed,
            downloadnum,
            files,
            dirs,
            slotsfree,
            countrycode,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoomEvent {
    room_name: String,
    username: String,
}

impl ParseBytes for UserRoomEvent {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let room_name = read_string(src)?;
        let username = read_string(src)?;

        Ok(Self {
            room_name,
            username,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomJoined {
    room_name: String,
    users: Vec<RoomUser>,
    owner: Option<String>,
    operators: Option<RoomOperators>,
}

impl ParseBytes for RoomJoined {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let room_name = read_string(src)?;

        let user_nth = src.get_u32_le();

        // Unpack user status
        let mut usernames = vec![];
        for _ in 0..user_nth {
            usernames.push(read_string(src)?);
        }

        // Unpack user status
        let user_status_nth = src.get_u32_le();
        let mut user_status = vec![];
        for _ in 0..user_status_nth {
            user_status.push(Status::from(src.get_u32_le()));
        }

        // Unpack user metadata
        let user_data_nth = src.get_u32_le();
        let mut user_data = vec![];
        for _ in 0..user_data_nth {
            user_data.push(UserData::parse(src)?);
        }

        // Unpack user free slots
        let free_slots_nth = src.get_u32_le();
        let mut free_slots = vec![];
        for _ in 0..free_slots_nth {
            free_slots.push(src.get_u32_le());
        }

        // Unpack user country codes
        let country_code_nth = src.get_u32_le();
        let mut country_codes = vec![];
        for _ in 0..country_code_nth {
            country_codes.push(read_string(src)?);
        }

        // Unpack room owner and operators
        let owner = read_string(src).ok();

        // if owner exists then we are on a private room : we can unpack operator
        let operators = if owner.is_some() {
            let operator_nth = src.get_u32_le();
            let mut operators = vec![];
            for _ in 0..operator_nth {
                operators.push(read_string(src)?);
            }
            Some(operators)
        } else {
            None
        };

        // Zip user info in a single struct
        let users = usernames
            .into_iter()
            .zip(user_data)
            .zip(user_status)
            .zip(country_codes)
            .map(|(((name, data), status), country)| RoomUser {
                name,
                status,
                data,
                country,
            })
            .collect();

        Ok(RoomJoined {
            room_name,
            users,
            owner,
            operators,
        })
    }
}

type RoomOperators = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomUser {
    name: String,
    status: Status,
    data: UserData,
    country: String,
}

impl ParseBytes for RoomUser {
    type Output = Self;

    fn parse(_src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        unimplemented!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomTickers {
    room: String,
    tickers: Vec<Ticker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomTicker {
    room: String,
    username: String,
    ticker: String,
}

impl ParseBytes for RoomTicker {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let room = read_string(src)?;
        let username = read_string(src)?;
        let ticker = read_string(src)?;

        Ok(Self {
            room,
            username,
            ticker,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    username: String,
    ticker: String,
}

impl ParseBytes for RoomTickers {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let room = read_string(src)?;
        let number_of_users = src.get_u32_le();

        let mut tickers = Vec::with_capacity(number_of_users as usize);

        for _ in 0..number_of_users {
            let username = read_string(src)?;
            let ticker = read_string(src)?;

            tickers.push(Ticker { username, ticker });
        }

        Ok(RoomTickers { room, tickers })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomUsers {
    room: String,
    users: Vec<String>,
}

impl ParseBytes for RoomUsers {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let room = read_string(src)?;
        let users_nth = src.get_u32_le();
        let mut users = Vec::with_capacity(users_nth as usize);

        for _ in 0..users_nth {
            users.push(read_string(src)?);
        }

        Ok(Self { room, users })
    }
}
