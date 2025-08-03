use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::bad_request;

use super::{Registry, RegistryError, TaskInfo};

pub fn router() -> Router<Arc<Mutex<Registry>>> {
    Router::new()
        .route("/api/tasks/{name}/stop", post(stop_task))
        .route("/api/tasks/{name}/start", post(start_task))
        .route("/api/tasks/{name}", get(get_task))
        .route("/api/tasks", get(list_tasks))
}

async fn get_task(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(name): Path<String>,
) -> Result<Json<TaskInfo>, impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Task name cannot be empty"));
    }

    let registry = match registry.lock() {
        Ok(registry) => registry,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock registry")),
    };

    match registry.get_task(&name) {
        Some(task) => Ok(Json(task.clone())),
        None => Err((StatusCode::NOT_FOUND, "Task not found")),
    }
}

async fn list_tasks(
    State(registry): State<Arc<Mutex<Registry>>>,
) -> Result<Json<Vec<TaskInfo>>, impl IntoResponse> {
    let registry = match registry.lock() {
        Ok(registry) => registry,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock registry")),
    };

    Ok(Json(registry.list_tasks()))
}

async fn stop_task(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(name): Path<String>,
) -> Result<(), impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Task name cannot be empty".into()));
    }

    let mut registry = match registry.lock() {
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
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(name): Path<String>,
) -> Result<(), impl IntoResponse> {
    if name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("Task name cannot be empty"),
        ));
    }

    let mut registry = match registry.lock() {
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
