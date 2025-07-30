use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use sqlx::{query_as, query_scalar};
use time::OffsetDateTime;
use tokio::task::spawn_blocking;

use crate::{
    app::AppState,
    db::Song,
    metadata::{SongFile, SongMetadata},
    paths::metadata_history_dir,
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

    let metadata_dir = metadata_history_dir().join(song_id.to_string());

    if !metadata_dir.exists() {
        std::fs::create_dir_all(&metadata_dir).map_err(internal_error)?;
    }

    std::fs::write(
        metadata_dir.join(format!(
            "{}.json",
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        )),
        serde_json::to_string_pretty(&original_metadata).map_err(internal_error)?,
    )
    .map_err(internal_error)?;

    tracing::info!("Original metadata: {original_metadata:#?}");

    *original_metadata = metadata.clone();
    tracing::info!("Updated metadata: {:#?}", file.metadata());

    let err = file.write();

    if let Err(err) = err {
        tracing::error!("Error writing metadata: {err}");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    Ok(StatusCode::OK)
}
