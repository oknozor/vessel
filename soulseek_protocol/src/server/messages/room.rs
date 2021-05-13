use crate::frame::ParseBytes;
use crate::frame::{read_string, ToBytes};
use crate::server::messages::user::{Status, UserData};
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, BufWriter};

type Rooms = Vec<(String, u32)>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RoomList {
    rooms: Rooms,
    owned_private_rooms: Rooms,
    private_rooms: Rooms,
    operated_private_rooms: Vec<String>,
}

impl ParseBytes for RoomList {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
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
    pub room: String,
    pub username: String,
}

impl ParseBytes for UserRoomEvent {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let room_name = read_string(src)?;
        let username = read_string(src)?;

        Ok(Self {
            room: room_name,
            username,
        })
    }
}

#[async_trait]
impl ToBytes for UserRoomEvent {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomJoined {
    room_name: String,
    users: Vec<RoomUser>,
    owner: Option<String>,
    operators: Option<RoomOperators>,
}

impl RoomJoined {
    // For this message we need the lessage len to determine if we need to continue parsing operator
    // in private rooms
    pub(crate) fn parse(src: &mut Cursor<&[u8]>, msg_len: usize) -> std::io::Result<Self> {
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

        let is_public_room = src.position() == (msg_len + 8) as u64;

        let mut owner = None;
        let mut operators = None;

        if !is_public_room {
            // Unpack room owner and operators
            owner = Some(read_string(src)?);
            // if owner exists then we are on a private room : we can unpack operator
            let operator_nth = src.get_u32_le();
            let mut ops = Vec::with_capacity(operator_nth as usize);
            for _ in 0..operator_nth {
                ops.push(read_string(src)?);
            }
            operators = Some(ops);
        }

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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let room = read_string(src)?;
        let users_nth = src.get_u32_le();
        let mut users = Vec::with_capacity(users_nth as usize);

        for _ in 0..users_nth {
            users.push(read_string(src)?);
        }

        Ok(Self { room, users })
    }
}
