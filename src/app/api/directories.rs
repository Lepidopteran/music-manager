use std::path::PathBuf;

use fs_extra::dir::get_size;
use serde::Serialize;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use sqlx::error::ErrorKind;
use sysinfo::Disks;

use crate::{
    app::{AppState, Database},
    db::Directory as DirectoryDB,
    utils::*,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Directory {
    name: String,
    path: String,
    path_size: Option<u64>,
    free_space: Option<u64>,
    total_space: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/directories/", get(get_directories))
        .route(
            "/api/directories/filesystem/*path",
            get(get_directory_folders),
        )
        .route("/api/directories/", post(add_directory))
        .route("/api/directories/:name", delete(remove_directory))
}

async fn add_directory(
    State(db): State<Database>,
    Json(directory): Json<DirectoryDB>,
) -> Result<Json<Directory>, impl IntoResponse> {
    if directory.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if directory.path.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Path cannot be empty".to_string()));
    }

    if !directory.path.starts_with('/') {
        return Err((StatusCode::BAD_REQUEST, "Path must be absolute".to_string()));
    }

    let path = std::path::Path::new(&directory.path);

    if !path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Path \"{}\" does not exist", directory.path),
        ));
    }

    if !path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Path \"{}\" is not a directory", directory.path),
        ));
    }

    let directories = sqlx::query_scalar!("SELECT path FROM directories")
        .fetch_all(&db)
        .await
        .map_err(internal_error)?;

    for entry in directories {
        if path.starts_with(entry) {
            return Err((
                StatusCode::CONFLICT,
                format!(
                    "Path \"{}\" is a subdirectory of an existing directory",
                    directory.path
                ),
            ));
        }
    }

    let result = match sqlx::query!(
        "INSERT INTO directories (name, path) VALUES (?, ?)",
        directory.name,
        directory.path
    )
    .execute(&db)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => directory,
        Ok(_) => directory,
        Err(err) => match err {
            sqlx::Error::Database(err) if err.kind() == ErrorKind::UniqueViolation => {
                tracing::error!("\"{}\" already exists", directory.name);
                return Err((
                    StatusCode::CONFLICT,
                    format!("\"{}\" already exists", directory.name),
                ));
            }
            _ => return Err(internal_error(err)),
        },
    };

    let disks = Disks::new_with_refreshed_list();
    let disk = disks.iter().find(|disk| {
        result
            .path
            .contains(&disk.mount_point().to_string_lossy().to_string())
    });

    Ok(Json(Directory {
        path_size: get_size(&result.path).ok(),
        free_space: disk.map(|disk| disk.available_space()),
        total_space: disk.map(|disk| disk.total_space()),
        name: result.name,
        path: result.path,
    }))
}

async fn remove_directory(
    State(db): State<Database>,
    Path(name): Path<String>,
) -> Result<StatusCode, impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    match sqlx::query!("DELETE FROM directories WHERE name = ?", name)
        .execute(&db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(StatusCode::OK)
            } else {
                Err((StatusCode::NOT_FOUND, format!("\"{name}\" not found")))
            }
        }
        Err(err) => Err(internal_error(err)),
    }
}

async fn get_directories(
    State(db): State<Database>,
) -> Result<Json<Vec<Directory>>, impl IntoResponse> {
    let disks = Disks::new_with_refreshed_list();
    let directories = match sqlx::query_as!(DirectoryDB, "SELECT * FROM directories")
        .fetch_all(&db)
        .await
    {
        Ok(directories) => directories,
        Err(err) => {
            tracing::error!("{}", err);
            return Err(internal_error(err));
        }
    };

    let directories_with_space: Vec<Directory> = directories
        .into_iter()
        .filter_map(|directory| {
            let disk = disks.iter().find(|disk| {
                directory
                    .path
                    .contains(&disk.mount_point().to_string_lossy().to_string())
            });

            disk.map(|disk| Directory {
                path_size: get_size(&directory.path).ok(),
                free_space: Some(disk.available_space()),
                total_space: Some(disk.total_space()),
                name: directory.name,
                path: directory.path,
            })
        })
        .collect();

    Ok(Json(directories_with_space))
}

async fn get_directory_folders(
    path: Path<String>,
) -> Result<Json<Vec<PathBuf>>, impl IntoResponse> {
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
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            return Some(entry.path());
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
