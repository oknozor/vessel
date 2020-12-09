#[macro_use]
extern crate log;
#[macro_use]
extern crate tokio;

use soulseek_protocol::server_message::response::ServerResponse;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use warp::http::Method;
use warp::Filter;

struct Client(Receiver<String>);

struct Broadcaster {
    clients: Vec<Sender<String>>,
}

type Cache = Arc<Mutex<Vec<ServerResponse>>>;

// see : https://github.com/seanmonstar/warp/blob/master/examples/sse_chat.rs
pub async fn start_sse_listener(mut rx: Receiver<ServerResponse>) {
    info!("Starting to server sent event broadcast ...");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(&[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ]);

    // Soulseek send user list and room list once on connect, we cache this to pass them
    // to new sse client
    let cache = Arc::new(Mutex::new(Vec::default()));

    let broadcaster = Arc::new(Mutex::new(Broadcaster { clients: vec![] }));
    let broadcaster_copy = broadcaster.clone();

    // Cache room list and user list to give them to future web clients
    while let Some(message) = rx.recv().await {
        match message {
            ServerResponse::RoomList(rooms) => {
                let mut cache = cache.lock().unwrap();
                cache.push(ServerResponse::RoomList(rooms));
            }
            ServerResponse::PrivilegedUsers(users) => {
                let mut cache = cache.lock().unwrap();
                cache.push(ServerResponse::PrivilegedUsers(users));

                // Once we get user list we can get out and proceed to the main event loop
                break;
            }
            other => debug!("Hanshake message : {:?}", other),
        }
    }

    // Dispatch soulseek messages to SSE clients
    let event_dispatcher = tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            debug!("SSE event : {}", message.kind());
            let message = serde_json::to_string(&message).expect("Serialization error");
            let broadcaster = broadcaster_copy.clone();
            let mut broadcaster = broadcaster.lock().unwrap();

            for client in broadcaster.clients.iter_mut() {
                client
                    .try_send(message.clone())
                    .expect("failed to send message to sse client");
            }
        }
    });

    let sse_events = warp::path!("events")
        .and(warp::get())
        .map(move || {
            // Stream incoming server event to sse
            let stream = broadcaster
                .lock()
                .unwrap()
                .new_client(cache.clone())
                .map(|msg| msg.map(warp::sse::json));

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
    fn new_client(&mut self, cache: Cache) -> Client {
        info!("sse client connected");

        let (tx, rx) = mpsc::channel(100);

        tx.clone()
            .try_send("data: connected\n\n".to_string())
            .unwrap();

        // get every cached data and push them as a welcome pack to the client
        let cache = cache.lock().unwrap();
        for item in cache.iter() {
            let json = serde_json::to_string(item).unwrap();
            tx.clone().try_send(json).unwrap();
        }

        self.clients.push(tx);
        Client(rx)
    }
}

impl Stream for Client {
    type Item = Result<String, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
