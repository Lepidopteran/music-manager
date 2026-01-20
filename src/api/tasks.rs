use axum::{
    Json, Router,
    extract::{Path, State},
    response::Result,
    routing::{get, post},
};

use crate::{
    Error,
    state::{AppState, TaskRegistry},
    tasks::TaskReport,
};

use super::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks/{name}/stop", post(stop_task))
        .route("/api/tasks/{name}/start", post(start_task))
        .route("/api/tasks", get(list_tasks))
}

async fn list_tasks(State(tasks): State<TaskRegistry>) -> Result<Json<Vec<TaskReport>>> {
    let registry = tasks.lock().map_err(internal_error)?;
    Ok(Json(registry.tasks()))
}

async fn stop_task(State(tasks): State<TaskRegistry>, Path(name): Path<String>) -> Result<()> {
    if name.trim().is_empty() {
        return Err(bad_request("Task name cannot be empty").into());
    }

    let mut registry = tasks.lock().map_err(internal_error)?;

    Ok(registry.stop_task(&name).map_err(Error::from)?)
}

async fn start_task(State(tasks): State<TaskRegistry>, Path(name): Path<String>) -> Result<()> {
    if name.trim().is_empty() {
        return Err(bad_request("Task name cannot be empty").into());
    }

    let mut registry = tasks.lock().map_err(internal_error)?;

    Ok(registry.start_task(&name).map_err(Error::from)?)
}
