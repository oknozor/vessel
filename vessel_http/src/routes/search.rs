use warp::Filter;

use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::search::SearchRequest;

use crate::sender::VesselSender;

pub fn search(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("search" / String).map(move |query| {
        sender.send(ServerRequest::FileSearch(SearchRequest {
            ticket: rand::random(),
            query,
        }));
        "ok"
    })
}
