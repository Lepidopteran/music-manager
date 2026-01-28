use std::convert::Infallible;

use axum::{
    Router,
    extract::State,
    response::{
        Sse,
        sse::{Event as SseEvent, KeepAlive},
    },
    routing::get,
};
use futures::Stream;
use serde::Serialize;
use time::OffsetDateTime;
use tokio::sync::broadcast::Sender;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use ts_rs::TS;

use crate::{
    AppState,
    tasks::{TaskEvent as TaskReport, TaskEventType},
};

#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
pub struct FileSystemEvent {
    #[serde(flatten)]
    pub inner: super::state::FileOperationEvent,
    #[serde(with = "time::serde::rfc3339")]
    #[ts(type = "Date")]
    pub timestamp: OffsetDateTime,

}

impl From<super::state::FileOperationEvent> for FileSystemEvent {
    fn from(event: super::state::FileOperationEvent) -> Self {
        Self {
            inner: event,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl From<FileSystemEvent> for SseEvent {
    fn from(event: FileSystemEvent) -> Self {
        SseEvent::default()
            .event("fs-event")
            .json_data(event)
            .expect("Failed to serialize event")
    }
}

#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export)]
pub struct TaskEvent {
    pub source: String,
    pub kind: TaskEventType,
    pub message: String,
    pub current: Option<u64>,
    pub total: Option<u64>,
    pub step: Option<u8>,
    #[serde(with = "time::serde::rfc3339")]
    #[ts(type = "Date")]
    pub timestamp: OffsetDateTime,
}

impl TaskEvent {
    pub fn new(source: &str, event: TaskReport) -> Self {
        Self {
            source: source.to_string(),
            kind: event.kind,
            message: event.message,
            current: event.current,
            total: event.total,
            step: event.step,
            timestamp: event.timestamp,
        }
    }
}

impl From<TaskEvent> for SseEvent {
    fn from(event: TaskEvent) -> Self {
        SseEvent::default()
            .event("task-event")
            .json_data(event)
            .expect("Failed to serialize event")
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum AppEventKind {
    Initialized,
    Shutdown,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppEvent {
    pub kind: AppEventKind,
    pub message: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl Default for AppEvent {
    fn default() -> Self {
        Self {
            kind: AppEventKind::Initialized,
            message: String::from("Muusik event channel initialized"),
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl From<AppEvent> for SseEvent {
    fn from(event: AppEvent) -> Self {
        SseEvent::default()
            .event("app-event")
            .json_data(event)
            .expect("Failed to serialize event")
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/events", get(handler))
}

async fn handler(
    State(tx): State<Sender<SseEvent>>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|event| event.ok().map(Ok));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
