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
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::debug;
use ts_rs::TS;

use crate::fs::{FileSystemOperation, FileSystemOperationEvent};

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "bindings.ts")]
pub enum FileOperationEvent {
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
    Progress {
        source: i128,
        bytes: u64,
        total_bytes: u64,
        current_dir: usize,
        dir_count: usize,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum FileOperationManagerError {
    #[error("Failed to add operation: {0}")]
    FailedToAddOperation(
        #[from] mpsc::error::SendError<(i128, FileSystemOperation, Arc<AtomicBool>)>,
    ),

    #[error("Couldn't find operation")]
    NotFound,

    #[error("Cannot remove in progress operation from queue")]
    InProgress,
}

type Result<T, E = FileOperationManagerError> = std::result::Result<T, E>;

#[derive(Clone, Debug, serde::Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[ts(export, export_to = "bindings.ts")]
pub enum FileOperationState {
    Move {
        to_from: HashMap<PathBuf, HashSet<PathBuf>>,
        status: FileSystemOperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
    Copy {
        to_from: HashMap<PathBuf, HashSet<PathBuf>>,
        status: FileSystemOperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
    Delete {
        paths: HashSet<PathBuf>,
        status: FileSystemOperationStatus,
        #[serde(skip)]
        stop_flag: Arc<AtomicBool>,
    },
}

impl FileOperationState {
    pub fn status(&self) -> &FileSystemOperationStatus {
        match self {
            FileOperationState::Move { status, .. } => status,
            FileOperationState::Copy { status, .. } => status,
            FileOperationState::Delete { status, .. } => status,
        }
    }

    pub fn stop_flag(&self) -> &Arc<AtomicBool> {
        match self {
            FileOperationState::Move { stop_flag, .. } => stop_flag,
            FileOperationState::Copy { stop_flag, .. } => stop_flag,
            FileOperationState::Delete { stop_flag, .. } => stop_flag,
        }
    }

    pub fn set_status(&mut self, status: FileSystemOperationStatus) {
        match self {
            FileOperationState::Move {
                status: previous_status,
                ..
            } => *previous_status = status,

            FileOperationState::Copy {
                status: previous_status,
                ..
            } => *previous_status = status,

            FileOperationState::Delete {
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

impl From<&FileSystemOperation> for FileOperationState {
    fn from(op: &FileSystemOperation) -> Self {
        match op {
            FileSystemOperation::Move { paths, .. } => FileOperationState::Move {
                to_from: paths.to_from().clone(),
                status: FileSystemOperationStatus::Pending,
                stop_flag: Default::default(),
            },
            FileSystemOperation::Copy { paths, .. } => FileOperationState::Copy {
                to_from: paths.to_from().clone(),
                status: FileSystemOperationStatus::Pending,
                stop_flag: Default::default(),
            },
            FileSystemOperation::Delete { paths, .. } => FileOperationState::Delete {
                paths: paths.clone(),
                status: FileSystemOperationStatus::Pending,
                stop_flag: Default::default(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "bindings.ts")]
pub enum FileSystemOperationStatus {
    #[default]
    Pending,
    InProgress,
}

type QueueItem = (i128, FileSystemOperation, Arc<AtomicBool>);

pub struct FileOperationManager {
    queue: tokio::sync::mpsc::Sender<QueueItem>,
    events: broadcast::Sender<FileOperationEvent>,
    state: Arc<Mutex<BTreeMap<i128, FileOperationState>>>,
}

impl FileOperationManager {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<QueueItem>(256);
        let (events, _) = broadcast::channel::<FileOperationEvent>(256);
        let state: Arc<Mutex<BTreeMap<i128, FileOperationState>>> =
            Arc::new(Mutex::new(BTreeMap::new()));

        let state_clone = state.clone();
        let events_clone = events.clone();
        tokio::spawn(async move {
            while let Some((id, operation, flag)) = rx.recv().await {
                let (tx, rx) = std::sync::mpsc::channel();
                let (bridged_tx, mut bridged_rx) = mpsc::channel(256);

                tokio::task::spawn_blocking(move || {
                    while let Ok(item) = rx.recv() {
                        if bridged_tx.blocking_send(item).is_err() {
                            break;
                        }
                    }
                });

                let state = state_clone.clone();
                let events = events_clone.clone();
                tokio::spawn(async move {
                    while let Some(item) = bridged_rx.recv().await {
                        match item {
                            FileSystemOperationEvent::Started => {
                                send_event(&events, FileOperationEvent::Started { source: id });

                                let mut state = state.lock().await;
                                if let Some(op) = state.get_mut(&id) {
                                    op.set_status(FileSystemOperationStatus::InProgress);
                                }
                            }
                            FileSystemOperationEvent::Completed => {
                                send_event(&events, FileOperationEvent::Completed { source: id });

                                let mut state = state.lock().await;
                                state.remove(&id);
                            }
                            FileSystemOperationEvent::Cancelled => {
                                send_event(&events, FileOperationEvent::Cancelled { source: id });

                                let mut state = state.lock().await;
                                state.remove(&id);
                            }
                            FileSystemOperationEvent::Progress {
                                bytes,
                                total_bytes,
                                current_dir,
                                dir_count,
                            } => {
                                send_event(
                                    &events,
                                    FileOperationEvent::Progress {
                                        source: id,
                                        bytes,
                                        total_bytes,
                                        current_dir,
                                        dir_count,
                                    },
                                );
                            }
                        }
                    }
                });

                if let Err(e) = tokio::task::spawn_blocking(move || operation.execute(&tx, &flag))
                    .await
                    .expect("Failed to execute operation")
                {
                    tracing::error!("Failed to execute operation: {e}");
                    events_clone
                        .send(FileOperationEvent::Failed {
                            source: id,
                            error: e.to_string(),
                        })
                        .expect("Failed to send event");
                }
            }
        });

        Self {
            queue: tx,
            events,
            state,
        }
    }

    pub async fn queue_operation(&self, operation: FileSystemOperation) -> Result<i128> {
        let id = OffsetDateTime::now_utc().unix_timestamp_nanos();
        let operation_state = FileOperationState::from(&operation);
        let flag = operation_state.stop_flag().clone();

        self.state.lock().await.insert(id, operation_state);

        self.queue.send((id, operation, flag)).await?;

        Ok(id)
    }

    pub async fn stop_operation(&self, id: i128) -> Result<()> {
        let mut state = self.state.lock().await;
        let state = state
            .get_mut(&id)
            .ok_or(FileOperationManagerError::NotFound)?;

        state.set_stop_flag(true);

        Ok(())
    }

    pub fn events(&self) -> broadcast::Receiver<FileOperationEvent> {
        self.events.subscribe()
    }
}

fn send_event(tx: &broadcast::Sender<FileOperationEvent>, event: FileOperationEvent) {
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
    use crate::fs::FileOperationPaths;

    #[test(tokio::test)]
    async fn test() {
        let temp = tempdir().expect("Failed to create temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("Failed to create dst dir");

        let src_file = src_dir.join("file.txt");
        fs::write(&src_file, "hello").expect("Failed to write file");

        let mut to_from = HashMap::new();
        to_from.insert(dst_dir.clone(), vec![src_file.clone()]);

        let manager = FileOperationManager::new();
        let mut events = manager.events();

        let event_task = tokio::spawn(async move {
            while let Ok(item) = events.recv().await {
                tracing::info!("Event: {item:?}");

                if let FileOperationEvent::Completed { .. } = item {
                    break;
                }
            }
        });

        let _ = manager
            .queue_operation(FileSystemOperation::move_files(
                FileOperationPaths::from(to_from),
                true,
                fs_extra::dir::CopyOptions::new(),
            ))
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

        let mut to_from = HashMap::new();
        to_from.insert(dst_dir.clone(), vec![src_dir.join("file2.txt")]);

        let manager = FileOperationManager::new();
        let mut events = manager.events();

        let event_task = tokio::spawn(async move {
            while let Ok(item) = events.recv().await {
                tracing::info!("Event: {item:?}");

                if let FileOperationEvent::Failed { .. } = item {
                    break;
                }
            }
        });

        let _ = manager
            .queue_operation(FileSystemOperation::move_files(
                FileOperationPaths::from(to_from),
                true,
                fs_extra::dir::CopyOptions::new(),
            ))
            .await
            .expect("Failed to add operation");

        event_task.await.expect("Failed to join event task");
    }
}
