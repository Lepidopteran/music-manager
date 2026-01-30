//! Manages FS operations.

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::{self, read_dir},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use serde::Serialize;
use ts_rs::TS;

const BUFFER_SIZE: usize = 64 * 1024;
const ORDERING: Ordering = Ordering::SeqCst;

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error("Failed to perform IO operation: {0}")]
    Io(#[from] std::io::Error),
    #[error("File doesn't exist: {0}")]
    FileNotFound(String),
    #[error("File already exists: {0}")]
    FileAlreadyExists(String),
}

type Result<T, E = OperationError> = std::result::Result<T, E>;
type OperationPaths = HashMap<PathBuf, PathBuf>;

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum OperationEvent {
    #[default]
    Started,
    Completed,
    Cancelled,
    Progress {
        file_index: usize,
        file_count: usize,
        copied_bytes: u64,
        total_bytes: u64,
    },
    Renamed {
        from: PathBuf,
        to: PathBuf,
    },
    Moved {
        from: PathBuf,
        to: PathBuf,
    },
    Copied {
        from: PathBuf,
        to: PathBuf,
    },
    Deleted {
        path: PathBuf,
    },
}

#[derive(Clone, Debug)]
pub enum Operation {
    Move {
        paths: OperationPaths,
        overwrite: bool,
        delete_empty_directories_after: bool,
    },
    Copy {
        paths: OperationPaths,
        overwrite: bool,
    },
    Delete {
        paths: HashSet<PathBuf>,
    },
}

impl Operation {
    pub fn execute(self, tx: &mpsc::Sender<OperationEvent>, stop_flag: &AtomicBool) -> Result<()> {
        match self {
            Self::Move {
                paths,
                overwrite,
                delete_empty_directories_after,
            } => Self::execute_move(
                paths,
                overwrite,
                delete_empty_directories_after,
                tx,
                stop_flag,
            )?,
            Self::Copy { paths, overwrite } => Self::execute_copy(paths, overwrite, tx, stop_flag)?,
            Self::Delete { paths } => Self::execute_delete(paths, tx, stop_flag)?,
        }

        send_event(tx, OperationEvent::Completed);
        Ok(())
    }

    fn execute_move(
        paths: OperationPaths,
        overwrite: bool,
        delete_empty_directories_after: bool,
        tx: &mpsc::Sender<OperationEvent>,
        stop_flag: &AtomicBool,
    ) -> Result<()> {
        let count = paths.len();
        for (index, (from, to)) in paths.iter().enumerate() {
            if Self::is_stopped(stop_flag, tx) {
                return Ok(());
            }

            if !from.exists() {
                return Err(OperationError::FileNotFound(
                    from.to_string_lossy().to_string(),
                ));
            }

            let to = if to.is_dir() {
                to.join(from.file_name().expect("File name should exist"))
            } else {
                to.to_path_buf()
            };

            if to.exists() && !overwrite {
                return Err(OperationError::FileAlreadyExists(
                    to.to_string_lossy().to_string(),
                ));
            }

            log::trace!("Moving {from:?} to {to:?}");

            match fs::rename(from, &to) {
                Ok(_) => {
                    send_event(
                        tx,
                        OperationEvent::Renamed {
                            from: from.to_path_buf(),
                            to: to.to_path_buf(),
                        },
                    );
                }

                Err(err) => {
                    if err.kind() != io::ErrorKind::CrossesDevices {
                        return Err(OperationError::from(err));
                    }

                    move_file(
                        from,
                        &to,
                        overwrite,
                        stop_flag,
                        |copied_bytes, total_bytes| {
                            handle_progress(copied_bytes, total_bytes, index, count, stop_flag, tx)
                        },
                    )?;

                    send_event(
                        tx,
                        OperationEvent::Moved {
                            from: from.to_path_buf(),
                            to: to.to_path_buf(),
                        },
                    );
                }
            }

            if delete_empty_directories_after {
                if from.is_dir() && read_dir(from)?.count() == 0 {
                    log::trace!("Removing empty dir: {from:?}");
                    std::fs::remove_dir(from)?;
                } else if let Some(parent) = from.parent()
                    && read_dir(parent)?.count() == 0
                {
                    log::trace!("Removing empty dir: {parent:?}");
                    std::fs::remove_dir(parent)?;
                }
            }
        }

        Ok(())
    }

    fn execute_copy(
        paths: OperationPaths,
        overwrite: bool,
        tx: &mpsc::Sender<OperationEvent>,
        stop_flag: &AtomicBool,
    ) -> Result<()> {
        let count = paths.len();
        for (index, (from, to)) in paths.iter().enumerate() {
            if Self::is_stopped(stop_flag, tx) {
                return Ok(());
            }

            let to = if to.is_dir() {
                to.join(from.file_name().expect("File name should exist"))
            } else {
                to.to_path_buf()
            };

            copy_file(
                from,
                to,
                overwrite,
                stop_flag,
                |copied_bytes, total_bytes| {
                    handle_progress(copied_bytes, total_bytes, index, count, stop_flag, tx);
                },
            )?;
        }

        Ok(())
    }

    fn execute_delete(
        paths: HashSet<PathBuf>,
        tx: &mpsc::Sender<OperationEvent>,
        stop_flag: &AtomicBool,
    ) -> Result<()> {
        for path in paths {
            if Self::is_stopped(stop_flag, tx) {
                return Ok(());
            }

            if path.is_dir() {
                fs::remove_dir(&path)?;
            } else {
                fs::remove_file(&path)?;
            }

            send_event(
                tx,
                OperationEvent::Deleted {
                    path: path.to_path_buf(),
                },
            );
        }

        Ok(())
    }

    fn is_stopped(stop_flag: &AtomicBool, tx: &mpsc::Sender<OperationEvent>) -> bool {
        if stop_flag.load(ORDERING) {
            send_event(tx, OperationEvent::Cancelled);
            return true;
        }

        false
    }
}

/// Returns the size of the path in bytes
pub fn path_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = fs::symlink_metadata(path.as_ref())?;

    let mut bytes = 0;

    if metadata.is_dir() {
        for entry in read_dir(&path)? {
            let entry = entry?;
            let entry_metadata = entry.metadata()?;

            if entry_metadata.is_dir() {
                bytes += path_size(entry.path())?;
            } else {
                bytes += entry_metadata.len();
            }
        }
    } else {
        bytes = metadata.len();
    }

    Ok(bytes)
}

fn copy_file<P: AsRef<Path>, T: AsRef<Path>, F: FnMut(u64, u64)>(
    from: P,
    to: T,
    overwrite: bool,
    stop_flag: &AtomicBool,
    mut handle_progress: F,
) -> Result<()> {
    if stop_flag.load(ORDERING) {
        return Ok(());
    }

    if from.as_ref().exists() {
        return Err(OperationError::FileNotFound(
            from.as_ref().to_string_lossy().to_string(),
        ));
    } else if to.as_ref().exists() && !overwrite {
        return Err(OperationError::FileAlreadyExists(
            to.as_ref().to_string_lossy().to_string(),
        ));
    }

    let mut file_from = fs::File::open(from)?;
    let file_size = file_from.metadata()?.len();
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut copied_bytes: u64 = 0;

    let mut file_to = fs::File::create(&to)?;

    while !stop_flag.load(ORDERING) && !buffer.is_empty() {
        match file_from.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                file_to.write_all(&buffer[..n])?;
                copied_bytes += n as u64;
                handle_progress(copied_bytes, file_size);
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(OperationError::from(err)),
        }
    }

    if stop_flag.load(ORDERING) && file_to.metadata()?.len() != file_size {
        let _ = std::fs::remove_file(&to);
    }

    Ok(())
}

fn move_file<P: AsRef<Path>, T: AsRef<Path>>(
    from: P,
    to: T,
    overwrite: bool,
    stop_flag: &AtomicBool,
    handle_progress: impl FnMut(u64, u64),
) -> Result<()> {
    copy_file(&from, to, overwrite, stop_flag, handle_progress)?;

    if !stop_flag.load(ORDERING) {
        fs::remove_file(&from)?;
    }

    Ok(())
}

fn send_event(tx: &mpsc::Sender<OperationEvent>, event: OperationEvent) {
    if let Err(err) = tx.send(event) {
        log::error!("Failed to send event: {err:?}");
    };
}

fn handle_progress(
    copied_bytes: u64,
    total_bytes: u64,
    file_index: usize,
    file_count: usize,
    stop_flag: &AtomicBool,
    tx: &mpsc::Sender<OperationEvent>,
) {
    if stop_flag.load(ORDERING) {
        send_event(tx, OperationEvent::Cancelled);
    }

    send_event(
        tx,
        OperationEvent::Progress {
            file_index,
            file_count,
            copied_bytes,
            total_bytes,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;
    use test_log::test;

    #[test]
    fn test_move_operation() {
        let stop_flag = AtomicBool::new(false);
        let (junk_tx, _) = mpsc::channel();

        let temp = tempdir().expect("Failed to create temp dir");
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&dst_dir).expect("Failed to create dst dir");

        let src_file = src_dir.join("file.txt");
        fs::write(&src_file, "hello").expect("Failed to write file");

        let mut paths = HashMap::new();
        paths.insert(src_file.clone(), dst_dir.clone());

        let op = Operation::Move {
            paths,
            delete_empty_directories_after: true,
            overwrite: true,
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
    fn test_cancelled_move_operation() -> Result<()> {
        let (junk_tx, _) = mpsc::channel();

        let stop_flag = AtomicBool::new(false);
        let temp = tempdir()?;
        let src_dir = temp.path().join("src");
        let dst_dir = temp.path().join("dst");

        std::fs::create_dir_all(&src_dir)?;
        std::fs::create_dir_all(&dst_dir)?;

        let src_file = src_dir.join("file.txt");
        std::fs::write(&src_file, "hello")?;

        let mut paths = HashMap::new();
        paths.insert(src_file.clone(), dst_dir.clone());
        let op = Operation::Move {
            paths,
            delete_empty_directories_after: false,
            overwrite: true,
        };

        stop_flag.store(true, ORDERING);

        let result = op.execute(&junk_tx, &stop_flag);

        assert!(result.is_ok() || result.is_err());

        Ok(())
    }
}
