use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::Serialize;
use time::OffsetDateTime;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use ts_rs::TS;

use crate::fs::{Operation, OperationError, OperationEvent};

type Result<T, E = OperationManagerError> = std::result::Result<T, E>;
type OperationResult = std::result::Result<(), OperationError>;

type QueueItem = (
    i128,
    Operation,
    Arc<AtomicBool>,
    mpsc::Sender<OperationEvent>,
    oneshot::Sender<OperationResult>,
);

#[derive(Debug, thiserror::Error)]
pub enum OperationManagerError {
    #[error("Failed to queue operation: {0}")]
    FailedToQueueOperation(#[from] mpsc::error::SendError<QueueItem>),

    #[error("Couldn't find operation")]
    NotFound,
}

#[derive(Debug)]
pub struct OperationHandle {
    id: i128,
    events: mpsc::Receiver<OperationEvent>,
    result: oneshot::Receiver<OperationResult>,
}

impl OperationHandle {
    pub fn id(&self) -> i128 {
        self.id
    }

    pub fn events(&mut self) -> &mut mpsc::Receiver<OperationEvent> {
        &mut self.events
    }

    pub fn result(&mut self) -> &mut oneshot::Receiver<OperationResult> {
        &mut self.result
    }
}

#[derive(Clone, Debug, serde::Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[ts(export, export_to = "bindings.ts", rename = "FileOperationState")]
pub enum OperationState {
    Move {
        paths: HashMap<PathBuf, PathBuf>,
        status: OperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
    Copy {
        paths: HashMap<PathBuf, PathBuf>,
        status: OperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
    Delete {
        paths: HashSet<PathBuf>,
        status: OperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
}

impl OperationState {
    pub fn status(&self) -> &OperationStatus {
        match self {
            OperationState::Move { status, .. } => status,
            OperationState::Copy { status, .. } => status,
            OperationState::Delete { status, .. } => status,
        }
    }

    pub fn stop_flag(&self) -> &Arc<AtomicBool> {
        match self {
            OperationState::Move { stop_flag, .. } => stop_flag,
            OperationState::Copy { stop_flag, .. } => stop_flag,
            OperationState::Delete { stop_flag, .. } => stop_flag,
        }
    }

    pub fn set_status(&mut self, status: OperationStatus) {
        match self {
            OperationState::Move {
                status: previous_status,
                ..
            } => *previous_status = status,

            OperationState::Copy {
                status: previous_status,
                ..
            } => *previous_status = status,

            OperationState::Delete {
                status: previous_status,
                ..
            } => *previous_status = status,
        }
    }

    pub fn set_stop_flag(&self, stop: bool) {
        let flag = self.stop_flag();

        flag.store(stop, Ordering::SeqCst);
    }
}

impl From<&Operation> for OperationState {
    fn from(op: &Operation) -> Self {
        match op {
            Operation::Move { paths, .. } => OperationState::Move {
                paths: paths.clone(),
                status: OperationStatus::Pending,
                stop_flag: Default::default(),
            },
            Operation::Copy { paths, .. } => OperationState::Copy {
                paths: paths.clone(),
                status: OperationStatus::Pending,
                stop_flag: Default::default(),
            },
            Operation::Delete { paths, .. } => OperationState::Delete {
                paths: paths.clone(),
                status: OperationStatus::Pending,
                stop_flag: Default::default(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "bindings.ts", rename ="FileOperationStatus")]
pub enum OperationStatus {
    #[default]
    Pending,
    InProgress,
}

#[derive(Clone, Debug)]
pub struct OperationManager {
    queue: tokio::sync::mpsc::Sender<QueueItem>,
    events: broadcast::Sender<OperationManagerEvent>,
    state: Arc<Mutex<BTreeMap<i128, OperationState>>>,
}

impl OperationManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<QueueItem>(256);
        let (events, _) = broadcast::channel::<OperationManagerEvent>(256);
        let state: Arc<Mutex<BTreeMap<i128, OperationState>>> =
            Arc::new(Mutex::new(BTreeMap::new()));

        let state_clone = state.clone();
        let events_clone = events.clone();
        tokio::spawn(async move {
            while let Some((id, operation, flag, operation_tx, result)) = rx.recv().await {
                let (tx, rx) = std::sync::mpsc::channel::<OperationEvent>();
                let (bridged_tx, mut bridged_rx) = mpsc::channel(256);

                tokio::task::spawn_blocking(move || {
                    while let Ok(item) = rx.recv() {
                        log::debug!("Sending event: {item:?}");
                        if operation_tx.blocking_send(item.clone()).is_err()
                            && bridged_tx.blocking_send(item).is_err()
                        {
                            break;
                        }
                    }
                });

                let state = state_clone.clone();
                let events = events_clone.clone();

                tokio::spawn(async move {
                    while let Some(item) = bridged_rx.recv().await {
                        match item {
                            OperationEvent::Started => {
                                send_event(&events, OperationManagerEvent::Started { source: id });

                                let mut state = state.lock().await;
                                if let Some(op) = state.get_mut(&id) {
                                    op.set_status(OperationStatus::InProgress);
                                }
                            }
                            OperationEvent::Completed => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Completed { source: id },
                                );

                                let mut state = state.lock().await;
                                state.remove(&id);
                            }
                            OperationEvent::Cancelled => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Cancelled { source: id },
                                );

                                let mut state = state.lock().await;
                                state.remove(&id);
                            }
                            OperationEvent::Progress {
                                copied_bytes,
                                total_bytes,
                                file_count,
                                file_index,
                            } => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Progress {
                                        source: id,
                                        copied_bytes,
                                        total_bytes,
                                        file_count,
                                        file_index,
                                    },
                                );
                            }
                            OperationEvent::Renamed { from, to } => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Renamed {
                                        source: id,
                                        from,
                                        to,
                                    },
                                );
                            }
                            OperationEvent::Moved { from, to } => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Moved {
                                        source: id,
                                        from,
                                        to,
                                    },
                                );
                            }
                            OperationEvent::Copied { from, to } => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Copied {
                                        source: id,
                                        from,
                                        to,
                                    },
                                );
                            }
                            OperationEvent::Deleted { path } => {
                                send_event(
                                    &events,
                                    OperationManagerEvent::Deleted { source: id, path },
                                );
                            }
                        }
                    }
                });

                let operation = tokio::task::spawn_blocking(move || operation.execute(&tx, &flag))
                    .await
                    .expect("Failed to execute operation");

                if let Err(e) = &operation {
                    tracing::error!("Failed to execute operation: {e}");
                    events_clone
                        .send(OperationManagerEvent::Failed {
                            source: id,
                            error: e.to_string(),
                        })
                        .expect("Failed to send event");
                }

                let _ = result.send(operation);
            }
        });

        Self {
            queue: tx,
            events,
            state,
        }
    }

    pub async fn queue_operation(&self, operation: Operation) -> Result<OperationHandle> {
        let id = OffsetDateTime::now_utc().unix_timestamp_nanos();
        let operation_state = OperationState::from(&operation);
        let flag = operation_state.stop_flag().clone();
        let (events_tx, events) = mpsc::channel(256);
        let (result_tx, result) = oneshot::channel();

        self.state.lock().await.insert(id, operation_state);
        self.queue
            .send((id, operation, flag, events_tx, result_tx))
            .await?;

        Ok(OperationHandle { id, events, result })
    }

    pub async fn stop_operation(&self, id: i128) -> Result<()> {
        let mut state = self.state.lock().await;
        let state = state.get_mut(&id).ok_or(OperationManagerError::NotFound)?;

        state.set_stop_flag(true);

        Ok(())
    }

    pub fn events(&self) -> broadcast::Receiver<OperationManagerEvent> {
        self.events.subscribe()
    }
}

impl Default for OperationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OperationManagerEvent {
    Failed {
        source: i128,
        error: String,
    },
    Started {
        source: i128,
    },
    Completed {
        source: i128,
    },
    Cancelled {
        source: i128,
    },
    Moved {
        source: i128,
        from: PathBuf,
        to: PathBuf,
    },
    Renamed {
        source: i128,
        from: PathBuf,
        to: PathBuf,
    },
    Copied {
        source: i128,
        from: PathBuf,
        to: PathBuf,
    },
    Deleted {
        source: i128,
        path: PathBuf,
    },
    Progress {
        source: i128,
        copied_bytes: u64,
        total_bytes: u64,
        file_index: usize,
        file_count: usize,
    },
}

impl OperationManagerEvent {
    pub fn source(&self) -> i128 {
        match self {
            OperationManagerEvent::Failed { source, .. } => *source,
            OperationManagerEvent::Started { source } => *source,
            OperationManagerEvent::Completed { source } => *source,
            OperationManagerEvent::Cancelled { source } => *source,
            OperationManagerEvent::Progress { source, .. } => *source,
            OperationManagerEvent::Moved { source, .. } => *source,
            OperationManagerEvent::Renamed { source, .. } => *source,
            OperationManagerEvent::Copied { source, .. } => *source,
            OperationManagerEvent::Deleted { source, .. } => *source,
        }
    }
}

fn send_event(tx: &broadcast::Sender<OperationManagerEvent>, event: OperationManagerEvent) {
    if tx.send(event).is_err() {
        tracing::error!("Failed to send event");
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use test_log::test;

    use super::*;

    #[test(tokio::test)]
    async fn test() {
        let temp = tempdir().expect("Failed to create temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("Failed to create dst dir");

        let src_file = src_dir.join("file.txt");
        fs::write(&src_file, "hello").expect("Failed to write file");

        let mut paths = HashMap::new();
        paths.insert(src_file.clone(), dst_dir.clone());

        let manager = OperationManager::new();
        let mut events = manager.events();

        let event_task = tokio::spawn(async move {
            while let Ok(item) = events.recv().await {
                tracing::info!("Event: {item:?}");

                if let OperationManagerEvent::Completed { .. } = item {
                    break;
                }
            }
        });

        let _ = manager
            .queue_operation(Operation::Move {
                paths,
                overwrite: true,
                delete_empty_directories_after: true,
            })
            .await
            .expect("Failed to add operation");

        event_task.await.expect("Failed to join event task");

        let dst_file = dst_dir.join("file.txt");

        assert!(dst_file.exists(), "destination file should exist");
        assert!(!src_file.exists(), "source file should be moved");

        assert!(
            !src_dir.exists(),
            "empty source directory should be deleted"
        );
    }

    #[test(tokio::test)]
    async fn test_fail() {
        let temp = tempdir().expect("Failed to create temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("Failed to create dst dir");

        let mut paths = HashMap::new();
        paths.insert(src_dir.join("file2.txt"), dst_dir.clone());

        let manager = OperationManager::new();
        let mut events = manager.events();

        let event_task = tokio::spawn(async move {
            while let Ok(item) = events.recv().await {
                tracing::info!("Event: {item:?}");

                if let OperationManagerEvent::Failed { .. } = item {
                    break;
                }
            }
        });

        let _ = manager
            .queue_operation(Operation::Move {
                paths,
                overwrite: true,
                delete_empty_directories_after: true,
            })
            .await
            .expect("Failed to add operation");

        event_task.await.expect("Failed to join event task");
    }
}
