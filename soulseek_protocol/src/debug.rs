use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn spawn_debug<F>(f: F, name: String) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    tokio::spawn(BlockDebug { name, f })
}

struct BlockDebug<F> {
    name: String,
    f: F,
}

impl<F: Future> Future for BlockDebug<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        println!("start {}", self.name);
        let inner = unsafe { Pin::map_unchecked_mut(self.as_mut(), |this| &mut this.f) };
        let res = inner.poll(cx);
        println!("stop {}", self.name);
        res
    }
}
