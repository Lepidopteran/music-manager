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

use crate::AppState;

#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
pub struct FileOperationManagerEvent {
    #[serde(flatten)]
    pub inner: super::state::OperationManagerEvent,
    #[serde(with = "time::serde::rfc3339")]
    #[ts(type = "Date")]
    pub timestamp: OffsetDateTime,
}

impl From<super::state::OperationManagerEvent> for FileOperationManagerEvent {
    fn from(event: super::state::OperationManagerEvent) -> Self {
        Self {
            inner: event,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl From<FileOperationManagerEvent> for SseEvent {
    fn from(event: FileOperationManagerEvent) -> Self {
        SseEvent::default()
            .event("fs-event")
            .json_data(event)
            .expect("Failed to serialize event")
    }
}

#[derive(Debug, Clone, serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "bindings.ts")]
pub struct JobManagerEvent {
    #[serde(flatten)]
    pub inner: super::state::JobManagerEvent,
    #[serde(with = "time::serde::rfc3339")]
    #[ts(type = "Date")]
    pub timestamp: OffsetDateTime,
}

impl From<super::state::JobManagerEvent> for JobManagerEvent {
    fn from(event: super::state::JobManagerEvent) -> Self {
        Self {
            inner: event,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl From<JobManagerEvent> for SseEvent {
    fn from(event: JobManagerEvent) -> Self {
        SseEvent::default()
            .event("job-event")
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
