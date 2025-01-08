use color_eyre::eyre::Result;
use sqlx::{query, sqlite::SqliteQueryResult};
use walkdir::{DirEntry, WalkDir};

use crate::metadata::Song;
use futures::stream::{iter, StreamExt};

use super::*;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

// TODO: add progress information

pub struct ScanSongs {
    info: TaskInfo,
    status: Arc<AtomicU8>,
    db: sqlx::Pool<sqlx::Sqlite>,
}

impl ScanSongs {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self {
            db: pool,
            status: Arc::new(AtomicU8::from(TaskStatus::default() as u8)),
            info: TaskInfo {
                id: "scan-songs".to_string(),
                name: "Scan Songs".to_string(),
                description: "Scans directories for songs".to_string(),
            },
        }
    }
}

impl Task for ScanSongs {
    fn start(&mut self) -> Result<(), TaskError> {
        let status = self.status.clone();
        if TaskStatus::is_running(status.load(Ordering::Relaxed)) {
            return Err(TaskError::Running);
        }

        let db = self.db.clone();

        status.store(TaskStatus::Running.into(), Ordering::Relaxed);
        tokio::spawn(async move {
            let directories: Vec<String> = query!("SELECT path FROM directories")
                .fetch_all(&db)
                .await
                .map(|result| result.into_iter().map(|row| row.path).collect())
                .map_err(|err| {
                    tracing::error!("Song scan error: {}", err);
                    drop(err)
                })
                .unwrap_or_default();

            if directories.is_empty() {
                tracing::warn!("No directories found, cancelling scan");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let mut song_paths = scan_song_paths(directories, status.clone());

            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            if song_paths.is_empty() {
                tracing::warn!("No songs found");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let existing_songs: Vec<String> = query!("SELECT path FROM songs")
                .fetch_all(&db)
                .await
                .map(|result| result.into_iter().map(|row| row.path).collect())
                .unwrap_or_default();

            song_paths.retain(|path| !existing_songs.contains(&path.to_string_lossy().to_string()));

            if song_paths.is_empty() {
                tracing::warn!("No new songs found");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let tasks: Vec<_> = song_paths
                .into_iter()
                .map(|path| {
                    let db = db.clone();
                    let path = path.clone();
                    async move {
                        add_song(db, path).await.map(|_| ())
                    }
                })
                .collect();

            // PERF: Increase the speed of adding data to the database.
            let results: Vec<_> = iter(tasks).buffer_unordered(1).collect().await;

            for result in results {
                if let Err(err) = result {
                    tracing::error!("Failed to insert song: {}", err);
                }
            }

            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                tracing::info!("Song scan cancelled");
                return;
            }

            status.store(TaskStatus::Idle.into(), Ordering::Relaxed);

            tracing::info!("Song scan complete");
        });

        Ok(())
    }

    fn stop(&mut self) -> Result<(), TaskError> {
        let status = self.status.clone();

        if !TaskStatus::is_running(status.load(Ordering::Relaxed)) {
            return Err(TaskError::Stop);
        }

        status.store(TaskStatus::Stopped.into(), Ordering::Relaxed);
        tracing::info!("Cancel request received, cancelling scan");
        Ok(())
    }

    fn status(&self) -> TaskStatus {
        TaskStatus::from(self.status.clone().load(Ordering::Relaxed))
    }

    fn info(&self) -> &TaskInfo {
        &self.info
    }
}

async fn add_song(
    pool: sqlx::Pool<sqlx::Sqlite>,
    path: PathBuf,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let metadata = match Song::from_path(&path) {
        Ok(song) => Some(song),
        Err(err) => {
            tracing::warn!("Failed to read tags: {}", err);
            None
        }
    };

    tracing::info!(
        "Adding song: {}, {:?}",
        path.to_string_lossy().to_string(),
        metadata
    );

    let parent_path = match path.parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => String::new(),
    };

    let path = path.to_string_lossy().to_string();

    match metadata {
        Some(song) => {
            query!(
                "INSERT INTO songs (path, parent_path, title, album, album_artist, disc_number, artist, year, track_number, genre) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                path,
                parent_path,
                song.title,
                song.album,
                song.album_artist,
                song.disc_number,
                song.artist,
                song.year,
                song.track_number,
                song.genre
            )
            .execute(&pool)
            .await
        }
        None => {
            query!(
                "INSERT INTO songs (path, parent_path) VALUES (?, ?)",
                path,
                parent_path
            )
            .execute(&pool)
            .await
        }
    }
}

fn scan_song_paths(directories: Vec<String>, status: Arc<AtomicU8>) -> Vec<PathBuf> {
    let mut songs: Vec<PathBuf> = Vec::new();
    for directory in directories {
        if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
            tracing::info!("Song scan cancelled");
            break;
        }

        for entry in WalkDir::new(&directory) {
            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                tracing::info!("Song scan cancelled");
                break;
            }

            match entry {
                Ok(entry) if is_music_file(&entry) => {
                    let path = entry.path().to_path_buf();

                    if !songs.contains(&path) {
                        songs.push(path);
                    }
                }
                Err(err) => {
                    tracing::error!("Song scan error: {}", err);
                }
                _ => {}
            }
        }
    }

    songs
}

fn is_music_file(entry: &DirEntry) -> bool {
    let extensions = [".mp3", ".m4a", ".flac", ".wav", ".ogg", ".wma", ".aac"];
    let file_name = entry.file_name().to_string_lossy();

    extensions.iter().any(|ext| file_name.ends_with(ext))
}
