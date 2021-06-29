#[macro_use]
extern crate tokio;
#[macro_use]
extern crate tracing;

use futures::{Stream, StreamExt};
use soulseek_protocol::{
    peers::p2p::{download::DownloadProgress, response::PeerResponse},
    server::response::ServerResponse,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::{
    sync::mpsc::{self, Receiver, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use warp::{sse::Event, Filter};

struct Client(UnboundedReceiver<Event>);

const MAX_SEARCH_RESULT: u32 = 500;

#[derive(Default, Clone)]
struct Broadcaster {
    clients: Arc<Mutex<Vec<UnboundedSender<Event>>>>,
}

#[instrument(name = "sse_listener", level = "trace", skip(rx))]
pub async fn start_sse_listener(
    rx: Receiver<ServerResponse>,
    peer_rx: Receiver<PeerResponse>,
    download_progress_rx: Receiver<DownloadProgress>,
) {
    info!("Starting server sent event broadcast ...");
    let cors = warp::cors().allow_any_origin();

    let broadcaster = Broadcaster::default();

    // Dispatch soulseek messages to SSE clients
    let event_dispatcher = dispatch_soulseek_message(rx, broadcaster.clone());

    // Dispatch peer messages to SSE clients
    let peer_event_dispatcher = dispatch_peer_message(peer_rx, broadcaster.clone());

    // Dispatch download progress to SSE
    let download_progress = dispatch_download_progress(download_progress_rx, broadcaster.clone());

    let users = warp::any().map(move || broadcaster.clone());

    let sse_events = warp::path!("events")
        .and(warp::get())
        .and(users)
        .map(|broadcaster: Broadcaster| {
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
        download_progress,
    );
}

fn dispatch_peer_message(
    mut peer_rx: Receiver<PeerResponse>,
    broadcaster: Broadcaster,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
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
                broadcaster.clone().send_message_to_clients(event, &data)
            }
        }
    })
}

fn dispatch_soulseek_message(
    mut rx: Receiver<ServerResponse>,
    broadcaster: Broadcaster,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        info!("Starting to dispatch vessel message to SSE clients");
        while let Some(message) = rx.recv().await {
            debug!("SSE event : {:?}", message);
            let data = serde_json::to_string(&message).expect("Serialization error");
            let event = "server_message".to_string();
            broadcaster.clone().send_message_to_clients(&event, &data);
        }
    })
}

fn dispatch_download_progress(
    mut rx: Receiver<DownloadProgress>,
    broadcaster: Broadcaster,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        info!("Starting to dispatch vessel message to SSE clients");
        while let Some(progress) = rx.recv().await {
            let event = match &progress {
                DownloadProgress::Init { .. } => "download_started".to_string(),
                DownloadProgress::Progress { .. } => "download_progress".to_string(),
            };

            let data = serde_json::to_string(&progress).expect("Serialization error");

            broadcaster.clone().send_message_to_clients(&event, &data);
        }
    })
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
