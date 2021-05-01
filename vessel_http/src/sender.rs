use std::fmt::Debug;

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct VesselSender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> VesselSender<T>
where
    T: Debug,
{
    pub(crate) fn new(sender: mpsc::Sender<T>) -> Self {
        Self { inner: sender }
    }

    pub(crate) fn send(&self, t: T) {
        self.inner.try_send(t).unwrap();
    }
}

impl<T> Clone for VesselSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
