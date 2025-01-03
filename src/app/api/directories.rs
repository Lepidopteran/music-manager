use serde::{Deserialize, Serialize};
use sqlx::{error::ErrorKind, FromRow};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

#[derive(Deserialize, Serialize, FromRow)]
pub struct Directory {
    pub name: String,
    pub path: String,
}

use crate::{app::AppState, utils::*};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/directories/", get(get_directories))
        .route("/api/directories/", post(add_directory))
        .route("/api/directories/:name", get(get_directory))
        .route("/api/directories/:name", delete(remove_directory))
}

async fn add_directory(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Json(directory): Json<Directory>,
) -> Result<Json<Directory>, impl IntoResponse> {
    match sqlx::query!(
        "INSERT INTO directories (name, path) VALUES (?, ?)",
        directory.name,
        directory.path
    )
    .execute(&db)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => Ok(Json(directory)),
        Ok(_) => Ok(Json(directory)),
        Err(err) => match err {
            sqlx::Error::Database(err) if err.kind() == ErrorKind::UniqueViolation => {
                tracing::error!("\"{}\" already exists", directory.name);
                Err((
                    StatusCode::CONFLICT,
                    format!("\"{}\" already exists", directory.name),
                ))
            }
            _ => Err(internal_error(err)),
        },
    }
}

async fn remove_directory(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(name): Path<String>,
) -> Result<StatusCode, impl IntoResponse> {
    if name.trim().is_empty() {
        return Ok(StatusCode::BAD_REQUEST);
    }

    match sqlx::query!("DELETE FROM directories WHERE name = ?", name)
        .execute(&db)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => Ok(StatusCode::OK),
        Ok(_) => Ok(StatusCode::NOT_FOUND),
        Err(err) => Err(internal_error(err)),
    }
}

async fn get_directory(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(name): Path<String>,
) -> Result<Json<Directory>, impl IntoResponse> {
    sqlx::query_as!(Directory, "SELECT * FROM directories WHERE name = ?", name)
        .fetch_one(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}

async fn get_directories(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
) -> Result<axum::Json<Vec<Directory>>, impl IntoResponse> {
    sqlx::query_as!(Directory, "SELECT * FROM directories")
        .fetch_all(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}
