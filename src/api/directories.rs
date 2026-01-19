use fs_extra::dir::get_size;
use serde::Serialize;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};

use sysinfo::Disks;
use ts_rs::TS;

use crate::{
    app::{AppState, Database},
    db::{directories, Directory as DirectoryDB, NewDirectory},
    utils::*,
};

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "Directory", export)]
struct DirectoryResponse {
    /// The name of the directory.
    name: String,
    /// The path of the directory.
    path: String,
    /// The display name of the directory, only used in the UI.
    display_name: Option<String>,
    /// The size of the directory takes up in bytes.
    path_size: Option<u64>,
    /// The free space of the hard drive the directory is stored on.
    free_space: Option<u64>,
    /// The total space of the hard drive the directory is stored on.
    total_space: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/directories/", get(get_directories))
        .route(
            "/api/directories/filesystem/{*path}",
            get(get_directory_folders),
        )
        .route("/api/directories/", post(add_directory))
        .route("/api/directories/{name}", delete(remove_directory))
}

async fn add_directory(
    State(pool): State<Database>,
    Json(new_directory): Json<NewDirectory>,
) -> Result<Json<DirectoryResponse>, Response> {
    let DirectoryDB {
        name,
        path,
        display_name,
    } = directories::add_directory(pool, new_directory)
        .await
        .map_err(IntoResponse::into_response)?;

    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .iter()
        .find(|disk| path.contains(&disk.mount_point().to_string_lossy().to_string()));

    Ok(Json(DirectoryResponse {
        free_space: disk.map(|disk| disk.available_space()),
        total_space: disk.map(|disk| disk.total_space()),
        path_size: get_size(&path).ok(),
        display_name,
        path,
        name,
    }))
}

async fn remove_directory(
    State(pool): State<Database>,
    Path(name): Path<String>,
) -> Result<StatusCode, Response> {
    directories::remove_directory(pool, name)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok(StatusCode::OK)
}

async fn get_directories(
    State(pool): State<Database>,
) -> Result<Json<Vec<DirectoryResponse>>, Response> {
    let disks = Disks::new_with_refreshed_list();
    let directories = directories::get_directories(&pool)
        .await
        .map_err(|err| err.into_response())?;

    let directories_with_space: Vec<DirectoryResponse> = directories
        .into_iter()
        .filter_map(|directory| {
            let disk = disks.iter().find(|disk| {
                directory
                    .path
                    .contains(&disk.mount_point().to_string_lossy().to_string())
            });

            disk.map(|disk| DirectoryResponse {
                name: directory.name,
                path_size: get_size(&directory.path).ok(),
                free_space: Some(disk.available_space()),
                total_space: Some(disk.total_space()),
                display_name: directory.display_name,
                path: directory.path,
            })
        })
        .collect();

    Ok(Json(directories_with_space))
}

async fn get_directory_folders(path: Path<String>) -> Result<Json<Vec<String>>, impl IntoResponse> {
    if path.to_string().trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Path cannot be empty".to_string()));
    }

    let path = std::path::PathBuf::from(&path.to_string());

    if !path.exists() {
        return Err((StatusCode::BAD_REQUEST, "Path does not exist".to_string()));
    }

    if !path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Path is not a directory".to_string(),
        ));
    }

    if path.is_relative() {
        return Err((StatusCode::BAD_REQUEST, "Path must be absolute".to_string()));
    }

    let directories = match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = std::fs::metadata(entry.path()) {
                        if metadata.is_dir() {
                            return Some(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                None
            })
            .collect(),
        Err(err) => return Err(internal_error(err)),
    };

    Ok(Json(directories))
}
