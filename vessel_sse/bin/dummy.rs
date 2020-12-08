use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (_, tx) = mpsc::channel(32);
    vessel_sse::start_sse_listener(tx).await;
}
