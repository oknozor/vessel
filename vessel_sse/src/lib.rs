#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use futures::{Stream, StreamExt};
use soulseek_protocol::server::messages::response::ServerResponse;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc::{self, Receiver, UnboundedReceiver, UnboundedSender};
use warp::sse::Event;
use warp::Filter;

struct Client(UnboundedReceiver<String>);

#[derive(Default, Clone)]
struct Broadcaster {
    clients: Arc<Mutex<Vec<UnboundedSender<String>>>>,
}

// see : https://github.com/seanmonstar/warp/blob/master/examples/sse_chat.rs
#[instrument(name = "sse_listener", level = "debug", skip(rx))]
pub async fn start_sse_listener(mut rx: Receiver<ServerResponse>) {
    info!("Starting server sent event broadcast ...");

    let cors = warp::cors().allow_any_origin();

    let broadcaster = Broadcaster::default();
    let broadcaster_copy = broadcaster.clone();

    // Dispatch soulseek messages to SSE clients
    let event_dispatcher = tokio::task::spawn(async move {
        info!("Starting to dispatch vessel message to SSE clients");
        while let Some(message) = rx.recv().await {
            info!("SSE event : {:?}", message);
            let message = serde_json::to_string(&message).expect("Serialization error");
            broadcaster_copy.clone().send_message_to_clients(message);
        }
    });

    let users = warp::any().map(move || broadcaster.clone());

    let sse_events = warp::path!("events")
        .and(warp::get())
        .and(users)
        .map(|broadcaster: Broadcaster| {
            info!("SSE EVENT");
            // Stream incoming server event to sse
            let stream = broadcaster.on_sse_event_received();
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        })
        .with(cors)
        .with(warp::log("api"));

    let _ = join!(
        warp::serve(sse_events).run(([127, 0, 0, 1], 3031)),
        event_dispatcher
    );
}


impl Broadcaster {
    fn on_sse_event_received(&self) -> impl Stream<Item=Result<Event, serde_json::Error>> + Send + 'static {
        let client = new_client(&self);
        client.map(|msg| warp::sse::Event::default().json_data(&msg.unwrap()))
    }

    fn send_message_to_clients(&self, message: String) {
        let mut clients = self.clients.lock().unwrap();

        // Update the client list, keeping only non errored channels
        let mut kept_client: Vec<UnboundedSender<String>> = vec![];
        for client in clients.iter().cloned() {
            if let Ok(()) = client.send(message.clone()) {
                kept_client.push(client);
            };
        }

        *clients = kept_client;
    }
}

fn new_client(broadcaster: &Broadcaster) -> Client {
    let mut clients = broadcaster.clients.lock().unwrap();

    let (tx, rx) = mpsc::unbounded_channel();
    tx.send("data: connected\n\n".to_string()).unwrap();
    info!("SSE client connected");

    clients.push(tx);
    Client(rx)
}

impl Stream for Client {
    type Item = Result<String, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
