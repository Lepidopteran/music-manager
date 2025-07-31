use std::{collections::HashMap, path::PathBuf};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use sqlx::{query_as, query_scalar};
use time::{OffsetDateTime, UtcDateTime};
use tokio::task::spawn_blocking;

use crate::{
    app::AppState,
    db::Song,
    metadata::{SongFile, SongMetadata},
    paths::metadata_history_dir,
    utils::*,
};

type SongId = i32;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/songs/", get(get_songs))
        .route("/api/songs/:id", get(get_song))
        .route("/api/songs/:id/refresh", post(refresh_song_details))
        .route("/api/songs/:id", put(edit_song))
        .route(
            "/api/songs/:id/metadata/restore/:timestamp",
            get(restore_metadata),
        )
        .route(
            "/api/songs/:id/metadata/history",
            get(get_song_metadata_history),
        )
}

async fn get_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
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
    Path(song_id): Path<SongId>,
) -> Result<Json<SongMetadata>, (StatusCode, String)> {
    let path = query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(PathBuf::from)
        .map_err(internal_error)?;

    let file = spawn_blocking(move || {
        SongFile::open(&path).map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
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

async fn get_song_metadata_history(
    Path(song_id): Path<SongId>,
) -> Result<Json<HashMap<UtcDateTime, SongMetadata>>, impl IntoResponse> {
    let metadata_dir = metadata_history_dir().join(song_id.to_string());

    if !metadata_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "No metadata found".to_string()));
    }

    let paths = std::fs::read_dir(metadata_dir)
        .map_err(internal_error)?
        .filter_map(|entry| {
            entry
                .ok()
                .filter(|entry| {
                    let path = entry.path();
                    path.extension() == Some("json".as_ref()) && path.is_file()
                })
                .map(|entry| entry.path())
        })
        .collect::<Vec<PathBuf>>();

    let metadata = paths
        .into_iter()
        .filter_map(|path| {
            let metadata: SongMetadata = serde_json::from_str(
                &std::fs::read_to_string(&path)
                    .map_err(internal_error)
                    .ok()?,
            )
            .ok()?;

            let timestamp = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| s.parse::<i128>().ok())
                .map(|ts| UtcDateTime::from_unix_timestamp_nanos(ts).unwrap())
                .unwrap();

            Some((timestamp, metadata))
        })
        .collect::<HashMap<_, _>>();

    Ok(Json(metadata))
}

async fn restore_metadata(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path((song_id, timestamp)): Path<(SongId, UtcDateTime)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let metadata_dir = metadata_history_dir().join(song_id.to_string());
    let path = metadata_dir.join(format!("{}.json", timestamp.unix_timestamp_nanos()));

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("No metadata found for timestamp \"{timestamp}\""),
        ));
    }

    let new_metadata: SongMetadata =
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(internal_error)?)
            .map_err(internal_error)?;

    let path = query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(PathBuf::from)
        .map_err(internal_error)?;

    let _ = spawn_blocking(move || update_metadata(song_id, &path, &new_metadata))
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}

async fn edit_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
    Json(metadata): Json<SongMetadata>,
) -> Result<StatusCode, (StatusCode, String)> {
    let path = query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(PathBuf::from)
        .map_err(internal_error)?;

    let _ = spawn_blocking(move || update_metadata(song_id, &path, &metadata))
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}

fn update_metadata(
    id: SongId,
    path: &std::path::Path,
    new_metadata: &SongMetadata,
) -> color_eyre::Result<()> {
    let mut song = SongFile::open(path)?;
    let original_metadata = song.metadata_mut();

    if original_metadata == new_metadata {
        return Ok(());
    }

    let metadata_dir = metadata_history_dir().join(id.to_string());

    if !metadata_dir.exists() {
        std::fs::create_dir_all(&metadata_dir)?;
    }

    std::fs::write(
        metadata_dir.join(format!(
            "{}.json",
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        )),
        serde_json::to_string_pretty(&original_metadata)?,
    )?;

    tracing::info!("Original metadata: {original_metadata:#?}");

    *original_metadata = new_metadata.clone();

    tracing::info!("Updated metadata: {:#?}", song.metadata());
    song.write()?;

    Ok(())
}
