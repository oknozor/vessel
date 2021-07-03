use crate::client::{Client, Clients};
use crate::MAX_SEARCH_RESULT;
use futures::{Stream, StreamExt};
use soulseek_protocol::{
    peers::p2p::{download::DownloadProgress, response::PeerResponse},
    server::response::ServerResponse,
};
use std::{collections::HashMap, convert::Infallible};
use tokio::sync::mpsc::UnboundedSender;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use warp::sse::Event;

#[derive(Default, Clone)]
pub(crate) struct Broadcaster {
    pub(crate) clients: Clients,
}

impl Broadcaster {
    pub(crate) fn on_sse_event_received(
        &self,
    ) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
        let client = Client::new(&self);
        client.map(|msg| msg)
    }

    fn send_message_to_clients(&self, event: &str, data: &str) {
        let mut clients = self.clients.lock().unwrap();

        // Update the client list, keeping only non errored channels
        let mut kept_client: Vec<UnboundedSender<Event>> = vec![];
        for client in clients.iter().cloned() {
            if let Ok(()) = client.send(Event::default().event(event).data(data)) {
                kept_client.push(client);
            };
        }

        *clients = kept_client;
    }

    pub(crate) fn dispatch_soulseek_message(
        &self,
        mut rx: Receiver<ServerResponse>,
    ) -> JoinHandle<()> {
        let broadcaster = self.clone();
        tokio::task::spawn(async move {
            info!("Starting to dispatch vessel message to SSE clients");
            while let Some(message) = rx.recv().await {
                debug!("SSE event : {:?}", message);
                let event = match message {
                    ServerResponse::LoginResponse(_) => "login_response",
                    ServerResponse::ListenPort(_) => "listen_port",
                    ServerResponse::PeerAddress(_) => "peer_address",
                    ServerResponse::UserAdded(_) => "user_added",
                    ServerResponse::UserRemoved(_) => "user_removed",
                    ServerResponse::UserStatus(_) => "user_status",
                    ServerResponse::ChatMessage(_) => "chat_message",
                    ServerResponse::RoomJoined(_) => "room_joined",
                    ServerResponse::RoomLeft(_) => "room_left",
                    ServerResponse::PrivateMessage(_) => "private_message",
                    ServerResponse::UserJoinedRoom(_) => "user_joined_room",
                    ServerResponse::UserLeftRoom(_) => "user_left_room",
                    ServerResponse::UserStats(_) => "user_stats",
                    ServerResponse::KickedFromServer => "kicked",
                    ServerResponse::Recommendations(_) => "recommendations",
                    ServerResponse::GlobalRecommendations(_) => "global_recommendations",
                    ServerResponse::UserInterests(_) => "user_interests",
                    ServerResponse::RoomList(_) => "room_lists",
                    ServerResponse::AdminMessage(_) => "admin_message",
                    ServerResponse::PrivilegedUsers(_) => "privileged_users",
                    ServerResponse::EmbeddedMessage(_) => "embedded_message",
                    ServerResponse::SimilarUsers(_) => "similar_users",
                    ServerResponse::ItemRecommendations(_) => "item_recommendations",
                    ServerResponse::ItemSimilarUsers(_) => "item_similar_users",
                    ServerResponse::RoomTickers(_) => "room_tickers",
                    ServerResponse::RoomTickersAdded(_) => "room_tickers_added",
                    ServerResponse::RoomTickersRemoved(_) => "room_tickers_removed",
                    ServerResponse::PrivateRoomUsers(_) => "private_room_users",
                    ServerResponse::PrivateRoomUserAdded(_) => "private_room_users_added",
                    ServerResponse::PrivateRoomUserRemoved(_) => "private_room_users_removed",
                    ServerResponse::PrivateRoomAdded(_) => "private_room_added",
                    ServerResponse::PrivateRoomRemoved(_) => "private_room_removed",
                    ServerResponse::PrivateRoomInvitationEnabled(_) => {
                        "private_room_invitation_enabled"
                    }
                    ServerResponse::PublicChatMessage(_) => "public_chat_message",
                    ServerResponse::CantConnectToPeer(_) => "cant_connect_to_peer",
                    ServerResponse::CantCreateRoom(_) => "cant_create_room",
                    _ => {
                        continue;
                    }
                };

                let data = serde_json::to_string(&message).expect("Serialization error");
                broadcaster.send_message_to_clients(&event, &data);
            }
        })
    }

    pub(crate) fn dispatch_peer_message(
        &self,
        mut peer_rx: Receiver<PeerResponse>,
    ) -> JoinHandle<()> {
        let broadcaster = self.clone();
        tokio::task::spawn(async move {
            info!("Starting to dispatch vessel message to SSE clients");
            let mut ticket_counts = HashMap::<u32, u32>::new();
            while let Some(message) = peer_rx.recv().await {
                let data = serde_json::to_string(&message).expect("Serialization error");

                let sse_event = match message {
                    PeerResponse::SearchReply(reply) => {
                        let ticket_count = ticket_counts.get(&reply.ticket);

                        let count = if let Some(count) = ticket_count {
                            count + 1
                        } else {
                            0
                        };

                        ticket_counts.insert(reply.ticket, count);

                        if count < MAX_SEARCH_RESULT {
                            Some("search_reply")
                        } else {
                            None
                        }
                    }
                    _ => Some("unimplemented"),
                };

                if let Some(event) = sse_event {
                    broadcaster.send_message_to_clients(event, &data)
                }
            }
        })
    }

    pub(crate) fn dispatch_download_progress(
        &self,
        mut rx: Receiver<DownloadProgress>,
    ) -> JoinHandle<()> {
        let broadcaster = self.clone();
        tokio::task::spawn(async move {
            info!("Starting to dispatch vessel message to SSE clients");
            while let Some(progress) = rx.recv().await {
                let event = match &progress {
                    DownloadProgress::Init { .. } => "download_started".to_string(),
                    DownloadProgress::Progress { .. } => "download_progress".to_string(),
                };

                let data = serde_json::to_string(&progress).expect("Serialization error");

                broadcaster.send_message_to_clients(&event, &data);
            }
        })
    }
}
