use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
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
        .route("/api/songs/:id", put(edit_song))
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

async fn edit_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<i32>,
    Json(metadata): Json<SongMetadata>,
) -> Result<StatusCode, (StatusCode, String)> {
    let path = query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(PathBuf::from)
        .map_err(internal_error)?;

    let mut file = spawn_blocking(move || {
        SongFile::open(&path).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
    })
    .await
    .map_err(internal_error)??;

    let original_metadata = file.metadata_mut();

    tracing::info!("Original metadata: {original_metadata:#?}");

    original_metadata.title = metadata.title.clone();
    original_metadata.artist = metadata.artist.clone();
    original_metadata.album = metadata.album.clone();
    original_metadata.album_artist = metadata.album_artist.clone();
    original_metadata.genre = metadata.genre.clone();
    original_metadata.track_number = metadata.track_number.clone();
    original_metadata.disc_number = metadata.disc_number.clone();
    original_metadata.year = metadata.year.clone();

    tracing::info!("Updated metadata: {:#?}", file.metadata());

    let err = file.write();

    if let Err(err) = err {
        tracing::error!("Error writing metadata: {err}");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    Ok(StatusCode::OK)
}
