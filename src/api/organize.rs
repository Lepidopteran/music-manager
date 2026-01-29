use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use super::not_found;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
};

use sqlx::Connection;
use ts_rs::TS;

use crate::{
    api::internal_error,
    db::{Song, directories, songs},
    fs::Operation,
    metadata::{Metadata, item::ItemKey},
    organize,
    state::{AppState, FileOperationEvent},
};

#[derive(serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PathRenamePreviewResult {
    pub previous_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(serde::Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct PathRenameOptions {
    pub rename_original_files: bool,
    pub directory_id: Option<String>,
}

impl Default for PathRenameOptions {
    fn default() -> Self {
        Self {
            rename_original_files: true,
            directory_id: None,
        }
    }
}

// TODO: Add ability to use a custom templates for folder structure.

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/albums/{title}/organize",
        get(preview_organize_album_tracks).post(organize_album_tracks),
    )
}

#[axum::debug_handler]
async fn organize_album_tracks(
    Path(title): Path<String>,
    State(AppState {
        file_operation_manager: manager,
        db,
        ..
    }): State<AppState>,
    Query(options): Query<PathRenameOptions>,
) -> Result<()> {
    let mut connection = db.acquire().await.map_err(internal_error)?;

    let album = songs::get_album(&mut connection, title)
        .await
        .map_err(IntoResponse::into_response)?;

    let directories = directories::get_directories(&mut connection)
        .await
        .map_err(IntoResponse::into_response)?;

    let tracks = album
        .tracks
        .iter()
        .try_fold(HashMap::new(), |mut paths, song| {
            let directory_id = options
                .directory_id
                .as_deref()
                .unwrap_or(&song.directory_id);

            let directory: PathBuf = directories
                .iter()
                .find(|dir| dir.name == directory_id)
                .ok_or_else(|| {
                    not_found(format!("Directory {} not found", song.directory_id)).into_response()
                })?
                .path
                .clone()
                .into();

            let path = directory.join(
                organize::render_song_path(
                    &handlebars::Handlebars::new(),
                    organize::DEFAULT_TEMPLATE,
                    &map_organize(song),
                    options.rename_original_files,
                )
                .map_err(IntoResponse::into_response)?,
            );

            paths.insert(song.path.clone().into(), (path.clone(), song.id.clone()));

            Ok::<HashMap<PathBuf, (PathBuf, String)>, Response>(paths)
        })?;

    let operation_id = manager
        .queue_operation(Operation::Move {
            paths: tracks
                .iter()
                .map(|(from, (to, _))| (from.clone(), to.clone()))
                .collect(),
            overwrite: true,
            delete_empty_directories_after: true,
        })
        .await?;

    let mut finished_items = HashSet::new();

    let mut connection = db.begin().await.map_err(internal_error)?;
    while let Ok(item) = manager.events().recv().await {
        if !item.source() == operation_id {
            continue;
        }

        match item {
            FileOperationEvent::Completed { .. } => {
                tracing::debug!("Completed");
                break;
            }
            FileOperationEvent::Renamed { from, to, .. } => {
                let (_, song_id) = tracks.get(&from).expect("Path not found");

                songs::update_song_path(
                    &mut connection,
                    song_id,
                    to.to_str().expect("Path is not valid UTF-8"),
                )
                .await
                .map_err(IntoResponse::into_response)?;

                tracing::debug!(
                    "Renamed {} to {}",
                    from.to_str().expect("Path is not valid UTF-8"),
                    to.to_str().expect("Path is not valid UTF-8")
                );

                finished_items.insert(song_id);
                if finished_items.len() == tracks.len() {
                    break;
                }
            }
            FileOperationEvent::Moved { from, to, .. } => {
                let (_, song_id) = tracks.get(&from).expect("Path not found");

                songs::update_song_path(
                    &mut connection,
                    song_id,
                    to.to_str().expect("Path is not valid UTF-8"),
                )
                .await
                .map_err(IntoResponse::into_response)?;

                tracing::debug!(
                    "Renamed {} to {}",
                    from.to_str().expect("Path is not valid UTF-8"),
                    to.to_str().expect("Path is not valid UTF-8")
                );
                finished_items.insert(song_id);
                if finished_items.len() == tracks.len() {
                    break;
                }
            }
            FileOperationEvent::Failed { error, .. } => {
                return Err(internal_error(error).into());
            }
            _ => continue,
        }
    }

    connection.commit().await.map_err(internal_error)?;

    Ok(())
}

async fn preview_organize_album_tracks(
    State(pool): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(title): Path<String>,
    Query(options): Query<PathRenameOptions>,
) -> Result<Json<Vec<PathRenamePreviewResult>>> {
    let mut connection = pool.acquire().await.map_err(internal_error)?;

    let album = songs::get_album(&mut connection, title)
        .await
        .map_err(IntoResponse::into_response)?;

    let directories = directories::get_directories(&mut connection)
        .await
        .map_err(IntoResponse::into_response)?;

    let previews = album
        .tracks
        .iter()
        .map(|song| {
            let directory_id = options
                .directory_id
                .as_deref()
                .unwrap_or(&song.directory_id);

            let directory: PathBuf = directories
                .iter()
                .find(|dir| dir.name == directory_id)
                .ok_or_else(|| {
                    not_found(format!("Directory {} not found", song.directory_id)).into_response()
                })?
                .path
                .clone()
                .into();

            Ok(PathRenamePreviewResult {
                previous_path: PathBuf::from(&song.path),
                new_path: directory.join(
                    organize::render_song_path(
                        &handlebars::Handlebars::new(),
                        organize::DEFAULT_TEMPLATE,
                        &map_organize(song),
                        options.rename_original_files,
                    )
                    .map_err(IntoResponse::into_response)?,
                ),
            })
        })
        .collect::<Result<Vec<_>, Response>>()?;

    Ok(Json(previews))
}

fn map_organize(song: &Song) -> organize::Song {
    organize::Song {
        file_path: PathBuf::from(&song.path),
        metadata: Metadata::new(
            BTreeMap::from([
                (ItemKey::Artist, song.artist.clone().unwrap_or_default()),
                (ItemKey::Album, song.album.clone().unwrap_or_default()),
                (ItemKey::Genre, song.genre.clone().unwrap_or_default()),
                (ItemKey::Mood, song.mood.clone().unwrap_or_default()),
                (
                    ItemKey::AlbumArtist,
                    song.album_artist.clone().unwrap_or_default(),
                ),
                (ItemKey::Title, song.title.clone().unwrap_or_default()),
                (
                    ItemKey::TrackNumber,
                    song.track_number.clone().unwrap_or_default().to_string(),
                ),
                (
                    ItemKey::DiscNumber,
                    song.disc_number.clone().unwrap_or_default().to_string(),
                ),
                (
                    ItemKey::Year,
                    song.year.clone().unwrap_or_default().to_string(),
                ),
            ]),
            BTreeMap::new(),
        ),
    }
}
