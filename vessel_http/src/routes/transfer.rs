use crate::routes::with_db;
use vessel_database::entity::download::DownloadEntity;
use vessel_database::entity::upload::UploadEntity;
use vessel_database::Database;
use warp::reply::json;
use warp::{Filter, Rejection, Reply};

pub fn route(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get_downloads = warp::path!("downloads")
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_downloads_handler);

    let get_uploads = warp::path!("uploads")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_uploads_handler);

    get_downloads.or(get_uploads)
}

async fn get_downloads_handler(database: Database) -> Result<impl Reply, Rejection> {
    Ok(json(&database.get_all::<DownloadEntity>()))
}

async fn get_uploads_handler(database: Database) -> Result<impl Reply, Rejection> {
    Ok(json(&database.get_all::<UploadEntity>()))
}
