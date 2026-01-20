use axum::{
    extract::{Path, State},
    response::{IntoResponse, Result},
    routing::get,
    Json, Router,
};

use crate::{
    app::AppState,
    db::{songs, Album},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/albums/{title}", get(get_album))
        .route("/api/albums/", get(get_albums))
}

async fn get_album(
    State(pool): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(title): Path<String>,
) -> Result<Json<Album>> {
    let album = songs::get_album(&pool, title)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(Json(album))
}

async fn get_albums(State(db): State<sqlx::Pool<sqlx::Sqlite>>) -> Result<Json<Vec<Album>>> {
    let albums = songs::get_albums(&db)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(Json(albums))
}
