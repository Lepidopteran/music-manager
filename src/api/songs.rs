use std::{collections::HashMap, path::PathBuf};

use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Result}, routing::{get, post, put}
};
use time::{OffsetDateTime, UtcDateTime};
use tokio::task::spawn_blocking;

use crate::{
    app::AppState,
    db::{songs, Song, UpdatedSong},
    metadata::{Metadata as SongMetadata, SongFile},
    paths::metadata_history_dir,
    utils::*,
};

type SongId = String;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/songs/", get(get_songs))
        .route("/api/songs/{id}", get(get_song))
        .route("/api/songs/{id}/file-info", post(get_song_file))
        .route("/api/songs/{id}/refresh", post(refresh_song_details))
        .route("/api/songs/{id}", put(edit_song))
        .route(
            "/api/songs/{id}/metadata/restore/{timestamp}",
            post(restore_metadata),
        )
        .route(
            "/api/songs/{id}/metadata/history",
            post(get_song_metadata_history),
        )
}

async fn get_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
) -> Result<Json<Song>> {
    let song = songs::get_song(&db, &song_id)
        .await
        .map(Json)
        .map_err(IntoResponse::into_response)?;

    Ok(song)
}

async fn get_songs(State(pool): State<sqlx::Pool<sqlx::Sqlite>>) -> Result<Json<Vec<Song>>> {
    let songs = songs::get_songs(&pool)
        .await
        .map(Json)
        .map_err(IntoResponse::into_response)?;

    Ok(songs)
}

async fn get_song_file(
    State(pool): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
) -> Result<Json<SongFile>> {
    let path = songs::get_song_path(&pool, &song_id)
        .await
        .map_err(IntoResponse::into_response)?;

    let file = read_song_file(path).await?;

    Ok(Json(file))
}

async fn refresh_song_details(
    State(pool): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
) -> Result<Json<Option<SongMetadata>>> {
    let path = songs::get_song_path(&pool, &song_id)
        .await
        .map_err(IntoResponse::into_response)?;

    let file = read_song_file(path).await?;
    let metadata = file.metadata().clone();

    songs::update_song(&pool, &song_id, UpdatedSong::from(file))
        .await
        .map_err(internal_error)?;

    Ok(Json(metadata))
}

async fn get_song_metadata_history(
    Path(song_id): Path<SongId>,
) -> Result<Json<HashMap<UtcDateTime, SongMetadata>>, impl IntoResponse> {
    let metadata_dir = metadata_history_dir().join(&song_id);

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
    State(pool): State<sqlx::Pool<sqlx::Sqlite>>,
    Path((song_id, timestamp)): Path<(SongId, UtcDateTime)>,
) -> Result<StatusCode> {
    let metadata_dir = metadata_history_dir().join(&song_id);
    let path = metadata_dir.join(format!("{}.json", timestamp.unix_timestamp_nanos()));

    if !path.exists() {
        return Err(not_found(format!("No metadata found for timestamp \"{timestamp}\"")).into());
    }

    let new_metadata: SongMetadata =
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(internal_error)?)
            .map_err(internal_error)?;

    let path = songs::get_song_path(&pool, &song_id)
        .await
        .map_err(IntoResponse::into_response)?;

    let _ = spawn_blocking(move || update_metadata(song_id, &path, &new_metadata))
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}

async fn read_song_file(path: PathBuf) -> Result<SongFile> {
    let file = spawn_blocking(move || SongFile::open(&path))
        .await
        .expect("Failed to join thread")
        .map_err(internal_error)?;

    Ok(file)
}

async fn edit_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<SongId>,
    Json(metadata): Json<SongMetadata>,
) -> Result<StatusCode> {
    let path = songs::get_song_path(&db, &song_id)
        .await
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

    if original_metadata.as_ref() == Some(new_metadata) {
        return Ok(());
    }

    let metadata_dir = metadata_history_dir().join(&id);

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
    song.set_metadata(new_metadata.clone());

    tracing::info!("Updated metadata: {:#?}", song.metadata());
    song.write()?;

    Ok(())
}
