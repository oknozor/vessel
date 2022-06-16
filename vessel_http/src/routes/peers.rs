use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::routes::with_sender;
use crate::{model::QueueRequest, sender::VesselSender};
use soulseek_protocol::peers::p2p::transfer::QueueUpload;
use soulseek_protocol::peers::{p2p::request::PeerRequest, PeerRequestPacket};

pub fn routes(
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let send_share_request = warp::path!("peers" / String / "shares")
        .and(warp::post())
        .and(with_sender(peer_sender.clone()))
        .and_then(send_share_handler);

    let queue_upload = warp::path!("peers" / String / "queue")
        .and(warp::post())
        .and(warp::body::json::<QueueRequest>())
        .and(with_sender(peer_sender.clone()))
        .and_then(queue_upload_handler);

    let send_user_info = warp::path!("users" / String / "info")
        .and(warp::post())
        .and(with_sender(peer_sender))
        .and_then(send_user_info_handler);

    send_share_request.or(queue_upload).or(send_user_info)
}

async fn send_share_handler(
    peer_name: String,
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> Result<impl Reply, Rejection> {
    peer_sender
        .send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::SharesRequest),
        ))
        .await;

    Ok(StatusCode::NO_CONTENT)
}

async fn queue_upload_handler(
    peer_name: String,
    request: QueueRequest,
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> Result<impl Reply, Rejection> {
    peer_sender
        .send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::QueueUpload(QueueUpload {
                file_name: request.file_name,
            })),
        ))
        .await;

    Ok(StatusCode::NO_CONTENT)
}

async fn send_user_info_handler(
    peer_name: String,
    peer_sender_copy: VesselSender<(String, PeerRequestPacket)>,
) -> Result<impl Reply, Rejection> {
    peer_sender_copy
        .send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::UserInfoRequest),
        ))
        .await;

    Ok(StatusCode::NO_CONTENT)
}
