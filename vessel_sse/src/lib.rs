#[macro_use]
extern crate log;

use futures::StreamExt;
use soulseek_protocol::server_message::response::ServerResponse;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::interval;
use warp::http::Method;
use warp::{sse::ServerSentEvent, Filter, Rejection, Reply};

pub async fn start_sse_listener(tx: Receiver<ServerResponse>) {
    info!("Starting to server sent event broadcast ...");
    warp::serve(router()).run(([127, 0, 0, 1], 3031)).await;
}

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("ticks").and(warp::get()).map(|| {
        let mut counter: u64 = 0;
        // create server event source
        let event_stream = interval(Duration::from_secs(1)).map(move |_| {
            counter += 1;
            sse_counter(counter)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    })
}

pub fn router() -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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

    routes().with(cors).with(warp::log("api"))
}

// create server-sent event
fn sse_counter(counter: u64) -> Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::data(counter))
}
