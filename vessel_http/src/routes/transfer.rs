use vessel_database::entity::download::DownloadEntity;
use vessel_database::entity::upload::UploadEntity;
use vessel_database::Database;
use warp::Filter;

pub fn get_downloads(
    database: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("downloads").map(move || warp::reply::json(&database.get_all::<DownloadEntity>()))
}

pub fn get_uploads(
    database: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("uploads").map(move || warp::reply::json(&database.get_all::<UploadEntity>()))
}
