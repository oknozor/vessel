use vessel_database::Database;
use warp::Filter;
use vessel_database::entity::download::DownloadEntity;

pub fn get_download(
    database: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("downloads").map(move || {
        warp::reply::json(&database.get_all::<DownloadEntity>())
    })
}
