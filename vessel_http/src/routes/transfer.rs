use vessel_database::Database;
use warp::Filter;

pub fn get_download(
    database: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("downloads").map(move || {
        warp::reply::json(&database.all_downloads())
    })
}
