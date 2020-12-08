use soulseek_protocol::connection::SlskConnection;
use soulseek_protocol::server_message::login::LoginRequest;
use soulseek_protocol::server_message::request::ServerRequest;
use soulseek_protocol::server_message::response::ServerResponse;
use std::sync::{Arc, Mutex};
use tokio::io::BufWriter;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> soulseek_protocol::Result<()> {
    let address = "server.slsknet.org:2242";
    let stream = TcpStream::connect(address).await?;

    let mut connection = Arc::new(Mutex::new(SlskConnection::new(stream)));
    let login = ServerRequest::Login(LoginRequest::new("popeye", "haricot"));

    connection.lock().unwrap().write_request(login).await?;

    let connection1 = connection.clone();
    tokio::spawn(async move {
        let mut vessel = connection1.lock().unwrap();
        loop {
            match vessel.read_response().await {
                Ok(Some(response)) => {
                    println!("{:?}", response);
                }
                Ok(None) => {
                    // Nothing to read
                }
                Err(e) => println!("{}", e),
            };
        }
    });

    connection
        .lock()
        .unwrap()
        .write_request(ServerRequest::JoinRoom("museek".to_string()))
        .await?;

    Ok(())
}
