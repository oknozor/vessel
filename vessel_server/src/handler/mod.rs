use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Semaphore};

/// TODO : Make this value configurable
pub const MAX_CONNECTIONS: usize = 10_000;

pub mod connection;
pub mod distributed;
pub mod indirect_connection;
pub mod peers;

#[derive(Debug, Clone)]
pub struct ShutdownHelper {
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown_complete_tx: mpsc::Sender<()>,
    pub limit_connections: Arc<Semaphore>,
}
