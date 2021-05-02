use warp::Filter;

use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::search::SearchRequest;

use crate::model;
use crate::model::{SearchQuery, SearchTicket};
use crate::sender::VesselSender;

pub fn search(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let opt_query = warp::query::<SearchQuery>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<SearchQuery>,), std::convert::Infallible>((None,)) });

    warp::path!("search")
        .and(opt_query)
        .map(move |query: Option<SearchQuery>| match query {
            Some(query) => {
                let ticket = rand::random();
                sender.send(ServerRequest::FileSearch(SearchRequest {
                    ticket,
                    query: query.term,
                }));
                warp::reply::json(&SearchTicket { ticket })
            }
            None => warp::reply::json(&model::Error {
                cause: "Failed to decode query param.".to_string(),
            }),
        })
}
