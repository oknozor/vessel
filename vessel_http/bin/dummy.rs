use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (rx, _) = mpsc::channel(32);
    vessel_http::start(rx).await;
}
