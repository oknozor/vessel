use std::sync::{Arc, Mutex};

pub mod channels;
pub mod connection;
pub mod dispatcher;
pub mod handler;
pub mod listener;
pub mod shutdown;

#[derive(Clone, Debug)]
pub struct SearchLimit {
    inner: Arc<Mutex<u32>>,
}

impl SearchLimit {
    fn decrement(&self) {
        let mut lock = self.inner.lock().unwrap();
        *lock -= 1;
    }

    fn get(&self) -> u32 {
        *self.inner.lock().unwrap()
    }
    pub(crate) fn new(limit: u32) -> Self {
        SearchLimit { inner: Arc::new(Mutex::new(limit)) }
    }
}

