use warp::Filter;

use soulseek_protocol::server::messages::request::ServerRequest;
use soulseek_protocol::server::messages::search::{SearchRequest};

use crate::sender::VesselSender;
use crate::model::SearchQuery;
use warp::http::StatusCode;
use warp::http::Response;
use std::collections::HashMap;

pub fn search(
    sender: VesselSender<ServerRequest>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let opt_query = warp::query::<SearchQuery>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<SearchQuery>,), std::convert::Infallible>((None,)) });

    warp::path!("search")
        .and(opt_query)
        .map(move |query: Option<SearchQuery>| {
        match query {
            Some(query) => {
                sender.send(ServerRequest::FileSearch(SearchRequest {
                    ticket: rand::random(),
                    query: query.term,
                }));
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body("ok")
            }
            None => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Failed to decode query param."),
        }
    })
}
