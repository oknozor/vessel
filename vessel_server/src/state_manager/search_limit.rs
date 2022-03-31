use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct SearchLimit {
    limit: Arc<Mutex<u32>>,
    ticket: Arc<Mutex<u32>>,
}

impl SearchLimit {
    pub fn reset(&self, ticket: u32) {
        let mut lock = self.limit.lock().unwrap();
        *lock = 30;
        let mut old_ticket = self.ticket.lock().unwrap();
        *old_ticket = ticket;
    }

    pub fn new() -> Self {
        SearchLimit {
            limit: Arc::new(Mutex::new(30)),
            ticket: Arc::new(Mutex::new(0)),
        }
    }

    pub fn decrement(&self) {
        let mut lock = self.limit.lock().unwrap();
        *lock -= 1;
    }

    pub fn get(&self) -> u32 {
        *self.limit.lock().unwrap()
    }

    pub fn ticket(&self) -> u32 {
        *self.ticket.lock().unwrap()
    }
}
