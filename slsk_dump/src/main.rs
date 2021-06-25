#[macro_use]
extern crate serde_derive;

use anyhow::Result;
use bytes::Buf;
use serde::Serialize;
use soulseek_protocol::server::MessageCode;
use std::io::{BufRead, BufReader, Cursor};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize)]
struct SoulseekMessageDump {
    code: String,
    raw_code: u32,
    len: u32,
    message: Vec<u8>,
}

fn main() -> Result<()> {
    let mut child = Command::new("tcpdump")
        .stdout(Stdio::piped())
        .arg("-x")
        .arg("tcp[tcpflags] & tcp-push != 0")
        .arg("and")
        .arg("dst")
        .arg("server.slsknet.org")
        .arg("and")
        .arg("src")
        .arg("192.168.0.17")
        .arg("and")
        .arg("port")
        .arg("2242")
        .spawn()?;

    let stdout = child.stdout.take().unwrap();

    let mut message = String::new();
    let mut slsk_unred: Vec<u8> = vec![];
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        // Filter out tcpdump header
        if !line.contains("slsknet.org") {
            let data: Vec<&str> = line.split(':').collect();
            let data = data[1];
            let data = data.replace(' ', "");
            message.push_str(&data);
        } else if !message.is_empty() {
            let message_bytes = message.as_str();
            // Get ip frame header len
            let header_len: u8 = str::parse::<u8>(&message_bytes[1..2])? * 8;

            // Advance to TCP Header length (str representation of 13 first byte = 26)
            let tcp_frame = &message_bytes[header_len as usize..];

            // Get Tcp header len
            let tcp_header_len: u8 = str::parse::<u8>(&tcp_frame[24..25])? * 4;

            // Advance to TCP push data
            let data = &tcp_frame[(tcp_header_len * 2) as usize..];

            let bytes = &hex::decode(&data)?;
            slsk_unred.extend_from_slice(bytes);
            let mut cursor = Cursor::new(slsk_unred.as_slice());

            println!("Converting tcpdump data to soulseek dump...");
            let remains = match bytes_to_json_dump(&mut cursor) {
                Ok((dumps, remains)) => {
                    for dump in dumps {
                        println!("{}", serde_json::to_string(&dump)?)
                    }
                    remains
                }
                Err(remains) => remains,
            };

            if remains.is_empty() {
                slsk_unred = vec![];
            } else {
                let bytes_red = cursor.position();
                slsk_unred.drain(0..bytes_red as usize);
            }

            message = String::new();
        }
    }

    child.wait()?;

    Ok(())
}

fn bytes_to_json_dump(
    cursor: &mut Cursor<&[u8]>,
) -> Result<(Vec<SoulseekMessageDump>, Vec<u8>), Vec<u8>> {
    let mut dumps = vec![];
    while cursor.remaining() >= 8 {
        let (message_code, len, message) = split_messages(cursor)?;

        let code = format!("{:?}", &message_code);
        let raw_code = message_code as u32;

        let dump = SoulseekMessageDump {
            code,
            raw_code,
            len,
            message,
        };

        dumps.push(dump);
    }

    Ok((dumps, cursor.chunk().to_vec()))
}

fn split_messages(cursor: &mut Cursor<&[u8]>) -> Result<(MessageCode, u32, Vec<u8>), Vec<u8>> {
    let len = cursor.get_u32_le();

    if len > cursor.remaining() as u32 {
        cursor.set_position(cursor.position() - 4);
        return Err(cursor.chunk().to_vec());
    }

    let code = MessageCode::from(cursor.get_u32_le());

    let len = if len >= 4 { len - 4 } else { 0 };

    let mut message = vec![];
    for _ in 0..len {
        message.push(cursor.get_u8());
    }

    Ok((code, len, message))
}
