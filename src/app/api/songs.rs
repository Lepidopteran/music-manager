use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::{query_as, query_scalar};
use tokio::task::spawn_blocking;

use crate::{
    app::AppState,
    db::Song,
    metadata::{SongFile, SongMetadata},
    utils::*,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/songs/", get(get_songs))
        .route("/api/songs/:id", get(get_song))
        .route("/api/songs/:id/refresh", post(refresh_song_details))
}

async fn get_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<i32>,
) -> Result<Json<Song>, impl IntoResponse> {
    query_as("SELECT * FROM songs WHERE id = ?")
        .bind(song_id)
        .fetch_one(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}

async fn refresh_song_details(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<i32>,
) -> Result<Json<SongMetadata>, (StatusCode, String)> {
    let path = query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(PathBuf::from)
        .map_err(internal_error)?;

    let file = spawn_blocking(move || {
        SongFile::open(&path).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
    })
    .await
    .map_err(internal_error)??;

    let metadata = file.metadata().clone();

    sqlx::query("UPDATE songs SET title = ?, artist = ?, album = ?, album_artist = ?, genre = ?, track_number = ?, disc_number = ?, year = ? WHERE id = ?")
        .bind(metadata.title.clone())
        .bind(metadata.artist.clone())
        .bind(metadata.album.clone())
        .bind(metadata.album_artist.clone())
        .bind(metadata.genre.clone())
        .bind(metadata.track_number.clone())
        .bind(metadata.disc_number.clone())
        .bind(metadata.year.clone())
        .bind(song_id)
        .execute(&db)
        .await
        .map_err(internal_error)?;

    Ok(Json(metadata))
}

async fn get_songs(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
) -> Result<axum::Json<Vec<Song>>, impl IntoResponse> {
    query_as("SELECT * FROM songs")
        .fetch_all(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}
