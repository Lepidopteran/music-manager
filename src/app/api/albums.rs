use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sqlx::query_as;

use crate::{metadata::{Album, Song}, utils::*};

pub fn router() -> Router<sqlx::Pool<sqlx::Sqlite>> {
    Router::new()
        .route("/api/albums/:title", get(get_album))
        .route("/api/albums/", get(get_albums))
}

async fn get_album(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(title): Path<String>,
) -> Result<Json<Album>, impl IntoResponse> {
    let tracks = match query_as!(Song, "SELECT * FROM songs WHERE album = ?", title)
        .fetch_all(&db)
        .await
    {
        Ok(tracks) => tracks,
        Err(err) => {
            tracing::error!("{}", err);
            return Err(internal_error(err));
        }
    };

    if tracks.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Album not found".to_string()));
    }

    let album = match Album::try_from(tracks) {
        Ok(album) => album,
        Err(err) => {
            tracing::error!("{}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
        }
    };

    Ok(Json(album))
}

#[derive(serde::Serialize)]
struct AlbumMetadata {
    title: Option<String>,
    artist: Option<String>,
}

async fn get_albums(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
) -> Result<Json<Vec<AlbumMetadata>>, impl IntoResponse> {
    match query_as!(
        AlbumMetadata,
        "SELECT album AS title, album_artist AS artist FROM songs WHERE album IS NOT NULL GROUP by album"
    )
    .fetch_all(&db)
    .await
    {
        Ok(albums) => Ok(Json(albums)),
        Err(err) => {
            tracing::error!("{}", err);
            Err(internal_error(err))
        }
    }
}
