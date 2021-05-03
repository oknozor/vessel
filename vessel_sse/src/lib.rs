#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use futures::{Stream, StreamExt};
use soulseek_protocol::peers::messages::p2p::response::PeerResponse;
use soulseek_protocol::server::messages::response::ServerResponse;
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc::{self, Receiver, UnboundedReceiver, UnboundedSender};
use warp::sse::Event;
use warp::Filter;

struct Client(UnboundedReceiver<Event>);

const MAX_SEARCH_RESULT: u32 = 10;

#[derive(Default, Clone)]
struct Broadcaster {
    clients: Arc<Mutex<Vec<UnboundedSender<Event>>>>,
}

// see : https://github.com/seanmonstar/warp/blob/master/examples/sse_chat.rs
#[instrument(name = "sse_listener", level = "debug", skip(rx))]
pub async fn start_sse_listener(
    mut rx: Receiver<ServerResponse>,
    mut peer_rx: Receiver<PeerResponse>,
) {
    info!("Starting server sent event broadcast ...");

    let cors = warp::cors().allow_any_origin();

    let broadcaster = Broadcaster::default();
    let broadcaster_copy = broadcaster.clone();

    // Dispatch soulseek messages to SSE clients
    let event_dispatcher = tokio::task::spawn(async move {
        info!("Starting to dispatch vessel message to SSE clients");
        while let Some(message) = rx.recv().await {
            info!("SSE event : {:?}", message);
            let data = serde_json::to_string(&message).expect("Serialization error");
            let event = "server_message".to_string();
            broadcaster_copy
                .clone()
                .send_message_to_clients(&event, &data);
        }
    });

    let broadcaster_copy = broadcaster.clone();
    // Dispatch soulseek messages to SSE clients
    let peer_event_dispatcher = tokio::task::spawn(async move {
        info!("Starting to dispatch vessel message to SSE clients");
        let mut ticket_counts = HashMap::<u32, u32>::new();
        while let Some(message) = peer_rx.recv().await {
            let data = serde_json::to_string(&message).expect("Serialization error");

            let sse_event = match message {
                PeerResponse::SearchReply(reply) => {
                    let ticket_count = ticket_counts.get(&reply.ticket);

                    let count = if let Some(count) = ticket_count {
                        count + 1
                    } else {
                        0
                    };

                    ticket_counts.insert(reply.ticket, count);

                    if count < MAX_SEARCH_RESULT {
                        Some("search_reply")
                    } else {
                        None
                    }
                }
                _ => Some("unimplemented"),
            };

            if let Some(event) = sse_event {
                broadcaster_copy
                    .clone()
                    .send_message_to_clients(event, &data)
            }
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
        event_dispatcher,
        peer_event_dispatcher,
    );
}

impl Broadcaster {
    fn on_sse_event_received(
        &self,
    ) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
        let client = new_client(&self);
        client.map(|msg| msg)
    }

    fn send_message_to_clients(&self, event: &str, data: &str) {
        let mut clients = self.clients.lock().unwrap();

        // Update the client list, keeping only non errored channels
        let mut kept_client: Vec<UnboundedSender<Event>> = vec![];
        for client in clients.iter().cloned() {
            if let Ok(()) = client.send(Event::default().event(event).data(data)) {
                kept_client.push(client);
            };
        }

        *clients = kept_client;
    }
}

fn new_client(broadcaster: &Broadcaster) -> Client {
    let mut clients = broadcaster.clients.lock().unwrap();

    let (tx, rx) = mpsc::unbounded_channel();
    let event = Event::default().event("new_client").data("connected");
    tx.send(event).unwrap();

    info!("SSE client connected");

    clients.push(tx);
    Client(rx)
}

impl Stream for Client {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
