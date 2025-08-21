use color_eyre::eyre::Result;
use sqlx::{query, sqlite::SqliteQueryResult};
use time::OffsetDateTime;
use tokio::sync::watch::{channel, Receiver, Sender};
use walkdir::{DirEntry, WalkDir};

use crate::{
    metadata::SongMetadata,
    task::{TaskEvent, TaskState},
};

use super::*;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
};

const SONG_SCAN_CANCEL_MESSAGE: &str = "Scan cancelled";

pub struct ScanSongs {
    info: TaskInfo,
    status: Arc<AtomicU8>,
    channel: (Sender<TaskEvent>, Receiver<TaskEvent>),
    started_at: Arc<RwLock<Option<OffsetDateTime>>>,
    completed_at: Arc<RwLock<Option<OffsetDateTime>>>,
    cancelled_at: Arc<RwLock<Option<OffsetDateTime>>>,
    db: sqlx::Pool<sqlx::Sqlite>,
}

impl ScanSongs {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        let status = Arc::new(AtomicU8::from(TaskStatus::default() as u8));
        Self {
            db: pool,
            status: status.clone(),
            started_at: Arc::new(RwLock::new(None)),
            completed_at: Arc::new(RwLock::new(None)),
            cancelled_at: Arc::new(RwLock::new(None)),
            info: TaskInfo {
                id: "scan-songs".to_string(),
                name: "Scan Songs".to_string(),
                description: "Scans directories for songs".to_string(),
                ..Default::default()
            },
            channel: channel(TaskEvent::initial("scan-songs")),
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
        let started_at = self.started_at.clone();
        let completed_at = self.completed_at.clone();
        let cancelled_at = self.cancelled_at.clone();
        let (tx, _) = self.channel.clone();

        tx.send(TaskEvent::start("Starting song scan")).unwrap();

        status.store(TaskStatus::Running.into(), Ordering::Relaxed);
        tokio::spawn(async move {
            let _ = started_at
                .write()
                .expect("Failed to set started_at")
                .replace(OffsetDateTime::now_utc());

            let directories: Vec<String> = query!("SELECT path FROM directories")
                .fetch_all(&db)
                .await
                .map(|result| result.into_iter().map(|row| row.path).collect())
                .map_err(|err| {
                    tracing::error!("Song scan error: {}", err);
                    drop(err)
                })
                .unwrap_or_default();

            tx.send(TaskEvent::info("Scanning directories")).unwrap();

            if directories.is_empty() {
                tracing::warn!("No directories found, cancelling scan");

                tx.send(TaskEvent::warning("No directories found, cancelling scan"))
                    .unwrap();

                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let mut song_paths = scan_song_paths(directories, status.clone(), tx.clone());

            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());
                return;
            }

            if song_paths.is_empty() {
                tracing::warn!("No songs found");
                tx.send(TaskEvent::stop("No songs found, stopping task..."))
                    .unwrap();
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
                tx.send(TaskEvent::stop("No new songs found, stopping task..."))
                    .unwrap();
                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let song_count = song_paths.len();

            tx.send(TaskEvent::info(
                format!("Found {song_count} new song(s)").as_str(),
            ))
            .unwrap();

            for (index, song) in song_paths.iter().enumerate() {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
                    tracing::info!("Song scan cancelled");
                    let _ = cancelled_at
                        .write()
                        .expect("Failed to set cancelled_at")
                        .replace(OffsetDateTime::now_utc());
                    return;
                }

                if let Err(err) = add_song(db.clone(), song.to_path_buf()).await {
                    tracing::error!("Song scan error: {err}");
                    tx.send(TaskEvent::error(
                        format!("Unable to add song: {err}").as_str(),
                    ))
                    .unwrap();
                } else {
                    tx.send(TaskEvent::progress(
                        format!("Added song \"{}\"", song.display()).as_str(),
                        index as u64,
                        song_count as u64,
                        None,
                    ))
                    .unwrap();
                }
            }

            status.store(TaskStatus::Idle.into(), Ordering::Relaxed);

            tx.send(TaskEvent::complete("Song scan complete")).unwrap();
            let _ = completed_at
                .write()
                .expect("Failed to set completed_at")
                .replace(OffsetDateTime::now_utc());

            tracing::info!("Song scan complete");
        });

        Ok(())
    }

    fn state(&self) -> TaskState {
        let status = self.status.clone();

        TaskState {
            status: TaskStatus::from(status.load(Ordering::Relaxed)),
            started_at: *self
                .started_at
                .clone()
                .read()
                .expect("Failed to read started_at"),
            completed_at: *self
                .completed_at
                .clone()
                .read()
                .expect("Failed to read completed_at"),
            stopped_at: *self
                .cancelled_at
                .clone()
                .read()
                .expect("Failed to read cancelled_at"),
        }
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

    fn info(&self) -> &TaskInfo {
        &self.info
    }

    fn channel(&self) -> Option<Receiver<TaskEvent>> {
        Some(self.channel.1.clone())
    }
}

async fn add_song(
    pool: sqlx::Pool<sqlx::Sqlite>,
    path: PathBuf,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let metadata = match SongMetadata::from_path(&path) {
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

    let path = path.to_string_lossy().to_string();

    match metadata {
        Some(song) => {
            let SongMetadata {
                title,
                album,
                album_artist,
                disc_number,
                artist,
                year,
                track_number,
                genre,
                mood,
            } = song;

            query!(
                "INSERT INTO songs (path, title, album, album_artist, disc_number, artist, year, track_number, genre, mood) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                path,
                title,
                album,
                album_artist,
                disc_number,
                artist,
                year,
                track_number,
                genre,
                mood
            )
            .execute(&pool)
            .await
        }
        None => {
            query!("INSERT INTO songs (path) VALUES (?)", path,)
                .execute(&pool)
                .await
        }
    }
}

fn scan_song_paths(
    directories: Vec<String>,
    status: Arc<AtomicU8>,
    tx: Sender<TaskEvent>,
) -> Vec<PathBuf> {
    let mut songs: Vec<PathBuf> = Vec::new();

    tx.send(TaskEvent::info("Scanning directories for music"))
        .unwrap();

    for directory in directories {
        if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
            tracing::info!(SONG_SCAN_CANCEL_MESSAGE);
            tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
            break;
        }

        for entry in WalkDir::new(&directory) {
            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                tracing::info!(SONG_SCAN_CANCEL_MESSAGE);
                tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
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
                    tracing::error!("Song scan error: {err}");
                    tx.send(TaskEvent::error(format!("Song scan error: {err}").as_str()))
                        .unwrap();
                }
                _ => {}
            }
        }
    }

    songs
}

fn is_music_file(entry: &DirEntry) -> bool {
    let extensions = [
        ".mp3", ".m4a", ".flac", ".wav", ".ogg", ".wma", ".aac", ".opus",
    ];
    let file_name = entry.file_name().to_string_lossy();

    extensions.iter().any(|ext| file_name.ends_with(ext))
}
