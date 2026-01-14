use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    app::{AppState, TaskRegistry},
    bad_request,
    task::{RegistryError, TaskReport},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks/{name}/stop", get(stop_task))
        .route("/api/tasks/{name}/start", get(start_task))
        .route("/api/tasks", get(list_tasks))
}

async fn list_tasks(
    State(tasks): State<TaskRegistry>,
) -> Result<Json<Vec<TaskReport>>, impl IntoResponse> {
    let registry = match tasks.lock() {
        Ok(registry) => registry,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock registry")),
    };

    Ok(Json(registry.tasks()))
}

async fn stop_task(
    State(tasks): State<TaskRegistry>,
    Path(name): Path<String>,
) -> Result<(), impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Task name cannot be empty".into()));
    }

    let mut registry = match tasks.lock() {
        Ok(registry) => registry,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to lock registry".into(),
            ))
        }
    };

    match registry.stop_task(&name) {
        Ok(_) => Ok(()),
        Err(err) => Err(match err {
            RegistryError::NotFound => (StatusCode::NOT_FOUND, String::from("Task not found")),
            RegistryError::StateError(err) => bad_request(err),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Failed to stop task"),
            ),
        }),
    }
}

async fn start_task(
    State(tasks): State<TaskRegistry>,
    Path(name): Path<String>,
) -> Result<(), impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("Task name cannot be empty"),
        ));
    }

    let mut registry = match tasks.lock() {
        Ok(registry) => registry,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Failed to lock registry"),
            ))
        }
    };

    match registry.start_task(&name) {
        Ok(_) => Ok(()),
        Err(err) => Err(match err {
            RegistryError::NotFound => (StatusCode::NOT_FOUND, String::from("Task not found")),
            RegistryError::StateError(err) => bad_request(err),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Failed to start task"),
            ),
        }),
    }
}
