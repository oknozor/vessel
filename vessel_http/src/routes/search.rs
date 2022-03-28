use warp::{Filter, Rejection, Reply};
use warp::reply::json;

use soulseek_protocol::server::{request::ServerRequest, search::SearchRequest};

use crate::{
    model::{SearchQuery, SearchTicket},
    sender::VesselSender,
};
use crate::routes::with_sender;


pub fn routes(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("search")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(with_sender(sender))
        .and_then(search_handler)
}


async fn search_handler(
    query: SearchQuery,
    sender: VesselSender<ServerRequest>,
) -> Result<impl Reply, Rejection> {
    let ticket = rand::random();

    sender.send(ServerRequest::FileSearch(SearchRequest {
        ticket,
        query: query.term,
    })).await;

    Ok(json(&SearchTicket { ticket }))
}
