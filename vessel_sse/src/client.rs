use crate::broadcast::Broadcaster;
use futures::Stream;
use std::{
    convert::Infallible,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use warp::sse::Event;

pub type Clients = Arc<Mutex<Vec<UnboundedSender<Event>>>>;
pub(crate) struct Client(UnboundedReceiver<Event>);

impl Client {
    pub(crate) fn new(broadcaster: &Broadcaster) -> Client {
        let mut clients = broadcaster.clients.lock().unwrap();

        let (tx, rx) = mpsc::unbounded_channel();
        let event = Event::default().event("new_client").data("connected");
        tx.send(event).unwrap();

        info!("SSE client connected");

        clients.push(tx);
        Client(rx)
    }
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
