use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Result},
    routing::get,
};

use crate::{
    AppState,
    api::internal_error,
    db::{
        albums::{self, Album},
        songs,
    },
    state::Pool,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/albums/{id}", get(get_album))
        .route("/api/albums/", get(get_albums))
}

async fn get_album(State(pool): State<Pool>, Path(id): Path<String>) -> Result<Json<Album>> {
    let mut connection = pool.acquire().await.map_err(internal_error)?;
    let album = albums::get_album(&mut connection, &id)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(Json(album))
}

async fn get_albums(State(pool): State<Pool>) -> Result<Json<Vec<Album>>> {
    let mut connection = pool.acquire().await.map_err(internal_error)?;
    let albums = albums::get_albums(&mut connection)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(Json(albums))
}
