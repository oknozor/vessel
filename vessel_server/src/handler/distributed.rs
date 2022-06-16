use crate::peers::handler::{connect_direct, PeerHandler};
use crate::{SearchLimit, SenderPool, ShutdownHelper, TryFutureExt};
use eyre::Result;
use soulseek_protocol::message_common::ConnectionType;
use soulseek_protocol::peers::p2p::response::PeerResponse;
use soulseek_protocol::server::peer::{Peer, RequestConnectionToPeer};
use soulseek_protocol::server::request::ServerRequest;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use rand::random;
use tokio::time::timeout;
use std::time::Duration;
use tokio::net::TcpStream;
use vessel_database::entity::peer::PeerEntity;
use vessel_database::Database;
use crate::peers::connection::PeerConnection;
use crate::peers::shutdown::Shutdown;

pub const MAX_PARENT: usize = 1;

pub struct DistributedConnectionManager {
    pub request_peer_connection_tx: mpsc::Sender<ServerRequest>,
    pub sse_tx: mpsc::Sender<PeerResponse>,
    pub ready_tx: mpsc::Sender<u32>,
    pub channels: SenderPool,
    pub shutdown_helper: ShutdownHelper,
    pub possible_parent_rx: Receiver<Vec<Peer>>,
    pub database: Database,
    pub search_limit: SearchLimit,
}

impl DistributedConnectionManager {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            while let Some(parents) = self.possible_parent_rx.recv().await {
                for parent in parents {
                    let parent = PeerEntity::from(parent);
                    let parent_count = self.channels.get_parent_count();
                    debug!("Connected to {}/{} parents", parent_count, MAX_PARENT);

                    if parent_count >= MAX_PARENT {
                        info!("Max parent count reached");
                        let server_request_sender = self.request_peer_connection_tx.clone();
                        server_request_sender
                            .send(ServerRequest::NoParents(false))
                            .await?;

                        return Ok(());
                    };

                    connect_to_peer_with_fallback(
                        self.request_peer_connection_tx.clone(),
                        self.sse_tx.clone(),
                        self.ready_tx.clone(),
                        self.channels.clone(),
                        self.shutdown_helper.clone(),
                        self.database.clone(),
                        &parent,
                        ConnectionType::DistributedNetwork,
                        self.search_limit.clone(),
                    )
                    .await?;
                }
            }
        }
    }
}

/// Try to connect directly to a peer and fallback to indirect connection
/// if direct connection fails.
pub(crate) async fn connect_to_peer_with_fallback(
    request_peer_connection_tx: Sender<ServerRequest>,
    sse_tx: Sender<PeerResponse>,
    ready_tx: Sender<u32>,
    channels: SenderPool,
    shutdown_helper: ShutdownHelper,
    database: Database,
    peer: &PeerEntity,
    conn_type: ConnectionType,
    search_limit: SearchLimit,
) -> Result<()> {
    shutdown_helper
        .limit_connections
        .acquire()
        .await
        .unwrap()
        .forget();

    debug!(
        "Available permit : {:?}",
        shutdown_helper.limit_connections.available_permits()
    );

    debug!("Trying direct {:?} connection to : {:?}", conn_type, peer);

    prepare_direct_connection_to_peer_with_fallback(
        channels.clone(),
        sse_tx.clone(),
        ready_tx.clone(),
        shutdown_helper.clone(),
        peer.clone(),
        database,
        search_limit.clone(),
    )
    .and_then(|handler| connect_direct(handler, conn_type))
    .or_else(|e| {
        warn!(
            "Direct connection to peer {:?} failed,  cause = {}",
            peer, e
        );

        request_indirect_connection(
            request_peer_connection_tx.clone(),
            channels.clone(),
            peer,
            conn_type,
        )
    })
    .await
}

// We were unable to connect to the peer, we are now asking the server
// to dispatch a connection request and expect a PierceFirewall message
// before upgrading the connection
async fn request_indirect_connection(
    request_peer_connection_tx: Sender<ServerRequest>,
    mut channels: SenderPool,
    peer: &PeerEntity,
    conn_type: ConnectionType,
) -> Result<()> {
    let token = random();

    // Save the channel state so we can later create the handler with the correct connection type
    channels.insert_indirect_connection_expected(&peer.username, conn_type, token);

    info!("Falling back to indirect connection with token {}", token);

    let server_request_sender = request_peer_connection_tx.clone();
    server_request_sender
        .send(ServerRequest::ConnectToPeer(RequestConnectionToPeer {
            token,
            username: peer.username.clone(),
            connection_type: conn_type,
        }))
        .await
        .map_err(|err| eyre!("Error sending connect to peer request {}", err))
}

async fn prepare_direct_connection_to_peer_with_fallback(
    channels: SenderPool,
    sse_tx: mpsc::Sender<PeerResponse>,
    ready_tx: mpsc::Sender<u32>,
    shutdown_helper: ShutdownHelper,
    peer: PeerEntity,
    db: Database,
    search_limit: SearchLimit,
) -> Result<PeerHandler> {
    let address = peer.get_address();
    let username = peer.username;

    match timeout(Duration::from_secs(1), TcpStream::connect(address)).await {
        Ok(Ok(socket)) => Ok(PeerHandler {
            peer_username: Some(username),
            connection: PeerConnection::new(socket),
            sse_tx: sse_tx.clone(),
            ready_tx,
            shutdown: Shutdown::new(shutdown_helper.notify_shutdown.subscribe()),
            limit_connections: shutdown_helper.limit_connections.clone(),
            limit_search: search_limit.clone(),
            _shutdown_complete: shutdown_helper.shutdown_complete_tx.clone(),
            connection_states: channels,
            db,
            address,
        }),
        Ok(Err(e)) => Err(eyre!(
            "Error in direct connection to peer {:?}, cause = {}",
            username,
            e
        )),
        Err(e) => Err(e.into()),
    }
}
