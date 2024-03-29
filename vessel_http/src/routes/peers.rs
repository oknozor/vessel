use warp::Filter;

use crate::{model::QueueRequest, sender::VesselSender};
use soulseek_protocol::peers::p2p::transfer::QueueUpload;
use soulseek_protocol::peers::{p2p::request::PeerRequest, PeerRequestPacket};

pub fn queue_upload_request(
    peer_sender: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("peers" / String / "queue"))
        .and(warp::body::json())
        .map(move |peer_name, request: QueueRequest| {
            peer_sender.send((
                peer_name,
                PeerRequestPacket::Message(PeerRequest::QueueUpload(QueueUpload {
                    file_name: request.file_name,
                })),
            ));
            "ok"
        })
}

pub fn send_user_info_request(
    peer_sender_copy: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / String / "info").map(move |peer_name| {
        peer_sender_copy.send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::UserInfoRequest),
        ));
        "ok"
    })
}

pub fn send_share_resquest(
    peer_sender_copy: VesselSender<(String, PeerRequestPacket)>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("peers" / String / "shares").map(move |peer_name| {
        peer_sender_copy.send((
            peer_name,
            PeerRequestPacket::Message(PeerRequest::SharesRequest),
        ));
        "ok"
    })
}
