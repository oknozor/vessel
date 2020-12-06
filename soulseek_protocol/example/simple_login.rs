use soulseek_protocol::connection::SlskConnection;
use tokio::net::TcpStream;
use soulseek_protocol::server_message::request::ServerRequest;
use soulseek_protocol::server_message::login::LoginRequest;

#[tokio::main]
async fn main() -> soulseek_protocol::Result<()> {
    let address = "server.slsknet.org:2242";
    let stream = TcpStream::connect(address).await?;
    let mut connection = SlskConnection::new(stream);
    let login = ServerRequest::Login(LoginRequest::new("popeye", "haricot"));

    let write_result = connection.write_request(login).await;
    println!("WRITE LOGIN {:?}", write_result);

    loop {
        match connection.read_response().await {
            Ok(Some(response))  => println!("{:?}", response),
            Ok(None) => println!("non"),
            Err(e) => println!("{}", e),
        };
    }

    Ok(())
}
