use std::convert::Infallible;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::get,
    Json, Router,
};
use futures::Stream;
use time::OffsetDateTime;
use tokio_stream::{wrappers::WatchStream, StreamExt};

use crate::{
    app::{AppState, TaskRegistry},
    bad_request,
    task::{TaskEventType, TaskReport},
};

use super::RegistryError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskEvent {
    pub source: String,
    pub kind: TaskEventType,
    pub message: String,
    pub current: Option<u64>,
    pub total: Option<u64>,
    pub step: Option<u8>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks/{name}/stop", get(stop_task))
        .route("/api/tasks/{name}/start", get(start_task))
        .route("/api/tasks/events", get(events))
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

async fn events(
    State(tasks): State<TaskRegistry>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let registry = tasks.lock().unwrap();
    let tasks = registry.list();
    let streams = tasks.iter().filter_map(|task| {
        let task_name = task.clone();
        registry.get_event_channel(task).map(move |channel| {
            WatchStream::new(channel).map(move |event| {
                let crate::task::TaskEvent {
                    kind,
                    message,
                    current,
                    total,
                    step,
                    timestamp,
                } = event;

                Ok(Event::default()
                    .event("task-event")
                    .json_data(TaskEvent {
                        source: task_name.to_string(),
                        kind,
                        message,
                        current,
                        total,
                        step,
                        timestamp,
                    })
                    .expect("Failed to serialize event"))
            })
        })
    });

    let stream = futures::stream::select_all(streams);
    Sse::new(stream).keep_alive(KeepAlive::default())
}
