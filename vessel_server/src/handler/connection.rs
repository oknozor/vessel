use std::fmt::Debug;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time;

const PEER_LISTENER_ADDRESS: &str = "0.0.0.0:2255";

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
#[derive(Debug)]
pub struct PeerConnectionListener {
    /// TCP listener supplied by the `run` caller.
    pub(crate) listener: TcpListener,
}

impl PeerConnectionListener {
    pub async fn new() -> Self {
        PeerConnectionListener {
            listener: TcpListener::bind(PEER_LISTENER_ADDRESS)
                .await
                .expect("Failed to start peer listener"),
        }
    }
    /// Accept an inbound connection.
    ///
    /// Errors are handled by backing off and retrying. An exponential backoff
    /// strategy is used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each subsequent
    /// failure doubles the wait time. If accepting fails on the 6th try after
    /// waiting for 64 seconds, then this function returns with an error.
    pub(crate) async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => {
                    if let Ok(_address) = socket.peer_addr() {
                        return Ok(socket);
                    };
                }
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}
