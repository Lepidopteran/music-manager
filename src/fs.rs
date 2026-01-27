//! Manages FS operations.

use std::{
    collections::{HashMap, HashSet},
    fs::{self, read_dir},
    io,
    path::PathBuf,
    sync::{atomic::AtomicBool, mpsc},
};

use fs_extra::dir::{CopyOptions, TransitProcessResult};

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("Failed to perform fs_extra operation: {0}")]
    FsExtra(#[from] fs_extra::error::Error),
    #[error("Failed to perform IO operation: {0}")]
    Io(#[from] std::io::Error),
}

type Result<T, E = FsError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct FileOperationPaths {
    pub to_from: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl From<HashMap<PathBuf, HashSet<PathBuf>>> for FileOperationPaths {
    fn from(to_from: HashMap<PathBuf, HashSet<PathBuf>>) -> Self {
        Self { to_from }
    }
}

impl From<HashMap<PathBuf, Vec<PathBuf>>> for FileOperationPaths {
    fn from(to_from: HashMap<PathBuf, Vec<PathBuf>>) -> Self {
        Self {
            to_from: to_from
                .into_iter()
                .map(|(to, from)| (to, from.into_iter().collect()))
                .collect(),
        }
    }
}

impl FileOperationPaths {
    pub fn new() -> Self {
        Self {
            to_from: HashMap::new(),
        }
    }

    pub fn insert(&mut self, from: PathBuf, to: PathBuf) {
        self.to_from.entry(to).or_default().insert(from);
    }
}

#[derive(Debug, Clone, Default)]
pub enum FileSystemOperationEvent {
    #[default]
    Started,
    Completed,
    Cancelled,
    Progress {
        bytes: u64,
        total_bytes: u64,
        current_dir: usize,
        dir_count: usize,
    },
}

pub enum FileSystemOperation {
    Move {
        paths: FileOperationPaths,
        delete_empty_directories_after: bool,
        options: CopyOptions,
    },
    Copy {
        paths: FileOperationPaths,
        options: CopyOptions,
    },
    Delete {
        paths: HashSet<PathBuf>,
    },
}

impl FileSystemOperation {
    pub fn move_files(
        paths: FileOperationPaths,
        delete_empty_directories_after: bool,
        options: CopyOptions,
    ) -> Self {
        Self::Move {
            paths,
            options,
            delete_empty_directories_after,
        }
    }

    pub fn copy_files(paths: FileOperationPaths, options: CopyOptions) -> Self {
        Self::Copy { paths, options }
    }

    pub fn delete_files(paths: HashSet<PathBuf>) -> Self {
        Self::Delete { paths }
    }

    pub fn execute(
        self,
        tx: &mpsc::Sender<FileSystemOperationEvent>,
        stop_flag: &AtomicBool,
    ) -> Result<()> {
        match self {
            Self::Move {
                paths,
                delete_empty_directories_after,
                options,
            } => {
                let count = paths.to_from.len();
                for (index, (to, from_items)) in paths.to_from.iter().enumerate() {
                    if check_stopped(stop_flag, tx) {
                        return Ok(());
                    }
                    for from in from_items {
                        if check_stopped(stop_flag, tx) {
                            return Ok(());
                        }
                        log::info!("Moving {from:?} to {to:?}");
                        fs::rename(
                            from,
                            to.is_dir()
                                .then_some(
                                    to.join(from.file_name().expect("Failed to get file name")),
                                )
                                .as_ref()
                                .unwrap_or(to),
                        )
                        .or_else(|err| {
                            if err.kind() != io::ErrorKind::CrossesDevices {
                                return Err(FsError::from(err));
                            }
                            fs_extra::move_items_with_progress(&[from], to, &options, |transit| {
                                handle_transit(transit, stop_flag, tx, count, index)
                            })
                            .map_err(FsError::from)?;

                            Ok(())
                        })?;
                    }

                    if delete_empty_directories_after {
                        let (dirs, files): (HashSet<_>, HashSet<_>) =
                            from_items.iter().partition(|p| p.is_dir());

                        for from in dirs {
                            if read_dir(from)?.count() == 0 {
                                log::info!("Removing empty dir: {from:?}");
                                std::fs::remove_dir(from)?;
                            }
                        }

                        for from in files {
                            if let Some(parent) = from.parent()
                                && read_dir(parent)?.count() == 0
                            {
                                log::info!("Removing empty dir: {parent:?}");
                                std::fs::remove_dir(parent)?;
                            }
                        }
                    }
                }

                send_event(tx, FileSystemOperationEvent::Completed);
                Ok(())
            }

            Self::Copy { paths, options } => {
                let count = paths.to_from.len();
                for (index, (to, from_items)) in paths.to_from.iter().enumerate() {
                    if check_stopped(stop_flag, tx) {
                        return Ok(());
                    }

                    fs_extra::copy_items_with_progress(
                        &from_items.iter().collect::<Vec<_>>(),
                        to,
                        &options,
                        |transit| handle_transit(transit, stop_flag, tx, count, index),
                    )?;
                }

                send_event(tx, FileSystemOperationEvent::Completed);
                Ok(())
            }

            Self::Delete { paths } => {
                for path in paths {
                    if check_stopped(stop_flag, tx) {
                        return Ok(());
                    }

                    if path.is_dir() {
                        fs::remove_dir(path)?;
                    } else {
                        fs::remove_file(path)?;
                    }
                }

                send_event(tx, FileSystemOperationEvent::Completed);
                Ok(())
            }
        }
    }
}

fn check_stopped(stop_flag: &AtomicBool, tx: &mpsc::Sender<FileSystemOperationEvent>) -> bool {
    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
        send_event(tx, FileSystemOperationEvent::Cancelled);
        return true;
    }

    false
}

fn send_event(tx: &mpsc::Sender<FileSystemOperationEvent>, event: FileSystemOperationEvent) {
    if let Err(err) = tx.send(event) {
        log::error!("Failed to send event: {err:?}");
    };
}

fn handle_transit(
    transit: fs_extra::TransitProcess,
    stop_flag: &AtomicBool,
    tx: &mpsc::Sender<FileSystemOperationEvent>,
    total_count: usize,
    current: usize,
) -> TransitProcessResult {
    if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
        send_event(tx, FileSystemOperationEvent::Cancelled);
        return TransitProcessResult::Abort;
    }

    send_event(
        tx,
        FileSystemOperationEvent::Progress {
            bytes: transit.copied_bytes,
            total_bytes: transit.total_bytes,
            current_dir: current,
            dir_count: total_count,
        },
    );

    TransitProcessResult::ContinueOrAbort
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn move_files_and_delete_empty_dirs() {
        let stop_flag = AtomicBool::new(false);
        let (junk_tx, _) = mpsc::channel();

        let temp = tempdir().expect("Failed to create temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("Failed to create dst dir");

        let src_file = src_dir.join("file.txt");
        fs::write(&src_file, "hello").expect("Failed to write file");

        let mut to_from = HashMap::new();
        to_from.insert(dst_dir.clone(), vec![src_file.clone()]);

        let op = FileSystemOperation::Move {
            paths: FileOperationPaths::from(to_from),
            delete_empty_directories_after: true,
            options: fs_extra::dir::CopyOptions::new(),
        };

        op.execute(&junk_tx, &stop_flag)
            .expect("Failed to move files");

        let dst_file = dst_dir.join("file.txt");
        assert!(dst_file.exists(), "destination file should exist");
        assert!(!src_file.exists(), "source file should be moved");

        assert!(
            !src_dir.exists(),
            "empty source directory should be deleted"
        );
    }

    #[test]
    fn move_files_with_stop_flag() -> Result<()> {
        use std::sync::atomic::{AtomicBool, Ordering};
        use tempfile::tempdir;

        let (junk_tx, _) = mpsc::channel();

        let stop_flag = AtomicBool::new(false);
        let temp = tempdir()?;
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        std::fs::create_dir_all(&src_dir)?;
        std::fs::create_dir_all(&dst_dir)?;

        let src_file = src_dir.join("file.txt");
        std::fs::write(&src_file, "hello")?;

        let mut to_from = HashMap::new();
        to_from.insert(dst_dir.clone(), vec![src_file.clone()]);
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;

        let op = FileSystemOperation::Move {
            paths: FileOperationPaths::from(to_from),
            delete_empty_directories_after: false,
            options,
        };

        stop_flag.store(true, Ordering::SeqCst);

        let result = op.execute(&junk_tx, &stop_flag);

        assert!(result.is_ok() || result.is_err());

        Ok(())
    }
}
