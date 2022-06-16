use crate::sender::VesselSender;
use soulseek_protocol::peers::PeerRequestPacket;
use soulseek_protocol::server::request::ServerRequest;
use std::convert::Infallible;
use vessel_database::Database;
use warp::Filter;

pub(crate) mod chat;
pub(crate) mod peers;
pub(crate) mod rooms;
pub(crate) mod search;
pub(crate) mod transfer;
pub(crate) mod users;

pub fn routes(
    db: Database,
    sender: VesselSender<ServerRequest>,
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    rooms::routes(sender.clone())
        .or(peers::routes(peer_sender))
        .or(chat::routes(sender.clone()))
        .or(users::routes(sender.clone(), db.clone()))
        .or(search::routes(sender.clone()))
        .or(transfer::route(db))
        .or(rooms::routes(sender))
}

fn with_sender<T: Send>(
    sender: VesselSender<T>,
) -> impl Filter<Extract = (VesselSender<T>,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
