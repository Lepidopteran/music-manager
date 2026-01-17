use color_eyre::eyre::{eyre, Result};
use sqlx::{query, query_as, sqlite::SqliteQueryResult};
use time::OffsetDateTime;
use tokio::{
    sync::watch::{channel, Receiver, Sender},
    task::spawn_blocking,
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    db::Song,
    get_metadata_field,
    metadata::{item::ItemKey, read_metadata_from_path, Metadata as SongMetadata},
    task::{TaskEvent, TaskState},
};

use super::*;

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, RwLock,
    },
};

const SONG_SCAN_CANCEL_MESSAGE: &str = "Scan cancelled";
const SONG_FILE_TYPES: [&str; 8] = ["mp3", "m4a", "flac", "wav", "ogg", "wma", "aac", "opus"];

// TODO: Clean up task event sending, possibly use a use broadcast channel instead of watch.

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
                description: "Scans for new songs, updates existing ones, and deletes songs that no longer exist".to_string(),
                steps: 3,
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

            let _ = tx.send(TaskEvent::info("Fetching directories..."));

            let directories: Vec<(String, String)> = query!("SELECT name, path FROM directories")
                .fetch_all(&db)
                .await
                .map(|result| result.into_iter().map(|row| (row.name, row.path)).collect())
                .map_err(|err| {
                    tracing::error!("Song scan error: {}", err);
                    drop(err)
                })
                .unwrap_or_default();

            if directories.is_empty() {
                tracing::warn!("No directories found, cancelling scan");

                tx.send(TaskEvent::warning("No directories found, cancelling scan"))
                    .unwrap();

                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            } else {
                let message = format!("Found {} directory(s)", directories.len());
                tracing::info!(message);
                tx.send(TaskEvent::info(&message)).unwrap();
            }

            let _ = tx.send(TaskEvent::info("Fetching existing songs..."));

            let existing_songs = query_as!(Song, "SELECT * FROM songs")
                .fetch_all(&db)
                .await
                .unwrap_or_default();

            if !existing_songs.is_empty() {
                let message = format!(
                    "Found {} existing song(s)... scanning for non-existing songs",
                    existing_songs.len()
                );

                tracing::info!(message);
                tx.send(TaskEvent::info(&message)).unwrap();
            } else {
                let message = "No existing songs found... scanning for new songs";

                tracing::info!(message);
                let _ = tx.send(TaskEvent::info(message));
            }

            let non_existing_song_ids = existing_songs
                .iter()
                .cloned()
                .filter_map(|song| (!PathBuf::from(&song.path).exists()).then_some(song.id))
                .collect::<HashSet<_>>();

            if !existing_songs.is_empty() {
                let message = format!(
                    "Found {} existing song(s) that no longer exist... scanning for new songs",
                    non_existing_song_ids.len()
                );

                tracing::info!(message);
                let _ = tx.send(TaskEvent::info(&message));
            }

            let existing_song_paths = existing_songs
                .iter()
                .map(|song| PathBuf::from(&song.path.clone()))
                .collect::<HashSet<_>>();

            let status_clone = status.clone();
            let tx_clone = tx.clone();
            let directories_clone = directories.clone();
            let song_paths = spawn_blocking(move || {
                scan_song_paths(
                    &directories_clone,
                    existing_song_paths,
                    status_clone,
                    tx_clone,
                )
            })
            .await
            .expect("Failed to join thread");

            if song_paths.is_empty() && !existing_songs.is_empty() {
                let message = format!(
                    "No new song(s) found... comparing metadata on {} existing song(s)...",
                    existing_songs.len()
                );

                tracing::info!(message);
                let _ = tx.send(TaskEvent::info(&message));
            } else if !song_paths.is_empty() && existing_songs.is_empty() {
                let message = format!("Found {} new song(s)...", song_paths.len());

                tracing::info!(message);
                let _ = tx.send(TaskEvent::info(&message));
            } else {
                let message = format!(
                    "Found {} new song(s)... comparing metadata on {} existing song(s)...",
                    song_paths.len(),
                    existing_songs.len()
                );

                tracing::info!(message);
                let _ = tx.send(TaskEvent::progress(&message, 1, 1, Some(1)));
            }

            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());
                return;
            }

            let existing_song_count = existing_songs.len();
            let comparison_tx = tx.clone();
            let comparison_tasks = existing_songs
                .into_iter()
                .filter(|song| !non_existing_song_ids.contains(&song.id))
                .enumerate()
                .map(move |(index, song)| {
                    let tx = comparison_tx.clone();
                    spawn_blocking(move || {
                        if index % 50 == 0 {
                            let _ = tx.send(TaskEvent::progress(
                                format!(
                                    "Comparing metadata {}%",
                                    index * 100 / existing_song_count
                                )
                                .as_str(),
                                index as u64,
                                existing_song_count as u64,
                                Some(2),
                            ));
                        }

                        get_updated_metadata(&song)
                    })
                });

            let updated_songs = futures::future::join_all(comparison_tasks)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .flatten()
                .collect::<Vec<(_, _, _)>>();

            if !updated_songs.is_empty() {
                tracing::info!("Found {} updated song(s)...", updated_songs.len());

                let _ = tx.send(TaskEvent::info(
                    format!("Found {} updated song(s)", updated_songs.len()).as_str(),
                ));
            } else {
                let message = "No updated song(s) found...";
                tracing::info!(message);
                let _ = tx.send(TaskEvent::progress(message, 1, 1, Some(2)));
            }

            if song_paths.is_empty() && non_existing_song_ids.is_empty() && updated_songs.is_empty()
            {
                tracing::warn!("No changes found, stopping task...");
                tx.send(TaskEvent::stop("No changes found, stopping task..."))
                    .unwrap();

                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());

                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let change_count =
                (song_paths.len() + updated_songs.len() + non_existing_song_ids.len()) as u64;
            let change_message = format!("Found {change_count} change(s)... Applying changes...");
            let mut transaction = db.begin().await.unwrap();
            let mut current_change_index = 0;

            tracing::info!(change_message);
            let _ = tx.send(TaskEvent::info(&change_message));

            for song in song_paths.iter() {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    let _ = tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE));
                    tracing::info!("Song scan cancelled");
                    let _ = cancelled_at
                        .write()
                        .expect("Failed to set cancelled_at")
                        .replace(OffsetDateTime::now_utc());
                    return;
                }

                match add_song(&mut transaction, &directories, song.to_path_buf()).await {
                    Ok(result) => {
                        if result.rows_affected() == 0 {
                            tracing::warn!(
                                "Change didn't result in any changes for song: {}",
                                song.to_string_lossy()
                            );
                            let _ = tx.send(TaskEvent::error(
                                format!("Unable to add song: {}", song.to_string_lossy()).as_str(),
                            ));
                        }
                    }

                    Err(err) => {
                        tracing::error!("Song scan error: {err}");
                        let _ = tx.send(TaskEvent::error(
                            format!("Unable to add song: {err}").as_str(),
                        ));
                    }
                }

                update_change_progress(&tx, &mut current_change_index, change_count);
            }

            for (song_id, metadata, created_at) in updated_songs {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    let _ = tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE));
                    tracing::info!("Song scan cancelled");
                    let _ = cancelled_at
                        .write()
                        .expect("Failed to set cancelled_at")
                        .replace(OffsetDateTime::now_utc());
                    return;
                }

                match update_song(&mut transaction, &song_id, &metadata, created_at).await {
                    Ok(result) => {
                        if result.rows_affected() == 0 {
                            tracing::warn!(
                                "Change didn't result in any changes for song: {}",
                                song_id
                            );
                            let _ = tx.send(TaskEvent::error(
                                format!("Unable to update song: {song_id}").as_str(),
                            ));
                        }
                    }

                    Err(err) => {
                        tracing::error!("Song scan error: {err}");
                        let _ = tx.send(TaskEvent::error(
                            format!("Unable to update song: {err}").as_str(),
                        ));
                    }
                }

                update_change_progress(&tx, &mut current_change_index, change_count);
            }

            for song_id in non_existing_song_ids {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    let _ = tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE));
                    tracing::info!("Song scan cancelled");
                    let _ = cancelled_at
                        .write()
                        .expect("Failed to set cancelled_at")
                        .replace(OffsetDateTime::now_utc());
                    return;
                }

                match delete_song(&mut transaction, &song_id).await {
                    Ok(result) => {
                        if result.rows_affected() == 0 {
                            tracing::warn!(
                                "Change didn't result in any changes for song: {}",
                                song_id
                            );
                            let _ = tx.send(TaskEvent::error(
                                format!("Unable to delete song: {song_id}").as_str(),
                            ));
                        }
                    }

                    Err(err) => {
                        tracing::error!("Song scan error: {err}");
                        let _ = tx.send(TaskEvent::error(
                            format!("Unable to delete song: {err}").as_str(),
                        ));
                    }
                }

                update_change_progress(&tx, &mut current_change_index, change_count);
            }

            tracing::info!("Finished applying changes, saving changes...");
            let _ = tx.send(TaskEvent::info(
                "Finished applying changes, saving changes...",
            ));

            if let Err(err) = transaction.commit().await {
                tracing::error!("Song scan error: {err}");
                let _ = tx.send(TaskEvent::error(
                    format!("Unable to save changes: {err}").as_str(),
                ));

                status.store(TaskStatus::Stopped.into(), Ordering::Relaxed);
                tx.send(TaskEvent::stop("Song scan could not save changes"))
                    .unwrap();
                let _ = completed_at
                    .write()
                    .expect("Failed to set completed_at")
                    .replace(OffsetDateTime::now_utc());

                return;
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

fn update_change_progress(tx: &Sender<TaskEvent>, index: &mut i32, total: u64) {
    *index += 1;
    let current = *index as u64;
    if current % 100 == 0 {
        let message = format!("Applying changes... {}%", current * 100 / total);
        tracing::info!(message);
        let _ = tx.send(TaskEvent::progress(
            message.as_str(),
            current,
            total,
            Some(3),
        ));
    }
}

async fn delete_song(
    connection: &mut sqlx::SqliteConnection,
    id: &str,
) -> color_eyre::eyre::Result<SqliteQueryResult> {
    Ok(query("DELETE FROM songs WHERE id = ?")
        .bind(id)
        .execute(connection)
        .await?)
}

async fn update_song(
    connection: &mut sqlx::SqliteConnection,
    id: &str,
    metadata: &Option<SongMetadata>,
    created_at: Option<OffsetDateTime>,
) -> color_eyre::eyre::Result<SqliteQueryResult> {
    let result = query("UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ?, mood = ?, file_created_at = ? WHERE id = ?")
        .bind(get_metadata_field(metadata, ItemKey::Title))
        .bind(get_metadata_field(metadata, ItemKey::Album))
        .bind(get_metadata_field(metadata, ItemKey::AlbumArtist))
        .bind(get_metadata_field(metadata, ItemKey::DiscNumber))
        .bind(get_metadata_field(metadata, ItemKey::Artist))
        .bind(get_metadata_field(metadata, ItemKey::Year))
        .bind(get_metadata_field(metadata, ItemKey::TrackNumber))
        .bind(get_metadata_field(metadata, ItemKey::Genre))
        .bind(get_metadata_field(metadata, ItemKey::Mood))
        .bind(created_at)
        .bind(id)
        .execute(&mut *connection)
        .await;

    Ok(result?)
}

async fn add_song(
    connection: &mut sqlx::SqliteConnection,
    directories: &[(String, String)],
    path: PathBuf,
) -> color_eyre::eyre::Result<SqliteQueryResult> {
    let created_at = OffsetDateTime::from(path.metadata()?.created()?);
    let metadata = &read_metadata_from_path(&path).ok();

    if metadata.is_none() {
        tracing::warn!(
            "No song tag metadata found for song: {}",
            path.to_string_lossy().to_string()
        );
    }

    let path = path.to_string_lossy().to_string();

    let uuid = Uuid::new_v4().to_string();
    let title = get_metadata_field(metadata, ItemKey::Title);
    let album = get_metadata_field(metadata, ItemKey::Album);
    let album_artist = get_metadata_field(metadata, ItemKey::AlbumArtist);
    let disc_number = get_metadata_field(metadata, ItemKey::DiscNumber);
    let artist = get_metadata_field(metadata, ItemKey::Artist);
    let year = get_metadata_field(metadata, ItemKey::Year);
    let track_number = get_metadata_field(metadata, ItemKey::TrackNumber);
    let genre = get_metadata_field(metadata, ItemKey::Genre);
    let mood = get_metadata_field(metadata, ItemKey::Mood);
    let added_at = OffsetDateTime::now_utc();
    let directory_id = directories
        .iter()
        .find_map(|(id, dir_path)| path.starts_with(dir_path).then_some(id.clone()))
        .ok_or(eyre!("Failed to find directory for song"))?;

    Ok(
        query!(
            "INSERT INTO songs (id, path, title, album, album_artist, disc_number, artist, year, track_number, genre, mood, added_at, file_created_at, directory_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            uuid,
            path,
            title,
            album,
            album_artist,
            disc_number,
            artist,
            year,
            track_number,
            genre,
            mood,
            added_at,
            created_at,
            directory_id
        )
        .execute(&mut *connection)
        .await?
    )
}

fn scan_song_paths(
    directories: &[(String, String)],
    existing_songs: HashSet<PathBuf>,
    status: Arc<AtomicU8>,
    tx: Sender<TaskEvent>,
) -> Vec<PathBuf> {
    let mut songs: Vec<PathBuf> = Vec::new();

    tx.send(TaskEvent::info("Scanning directories for music"))
        .unwrap();

    for (_, directory) in directories {
        if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
            tracing::info!(SONG_SCAN_CANCEL_MESSAGE);
            tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
            break;
        }

        let files = WalkDir::new(&directory)
            .into_iter()
            .filter_entry(|entry| {
                !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with('.'))
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                !e.file_type().is_dir()
                    && !existing_songs.contains(e.path())
                    && e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| SONG_FILE_TYPES.contains(&ext.to_lowercase().as_str()))
            })
            .collect::<Vec<_>>();

        tracing::info!("Found {} files in {}", files.len(), directory);
        tracing::info!("Scanning directory: {}", directory);

        for (index, file) in files.iter().enumerate() {
            if index % 50 == 0 {
                let _ = tx.send(TaskEvent::progress(
                    format!("Scanning {}%", (index + 1) * 100 / files.len()).as_str(),
                    index as u64,
                    files.len() as u64,
                    Some(1),
                ));
            }

            if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                tracing::info!(SONG_SCAN_CANCEL_MESSAGE);
                tx.send(TaskEvent::stop(SONG_SCAN_CANCEL_MESSAGE)).unwrap();
                break;
            }

            songs.push(file.path().to_path_buf());
        }
    }

    songs
}

fn get_updated_metadata(
    song: &Song,
) -> Option<(String, Option<SongMetadata>, Option<OffsetDateTime>)> {
    let path = PathBuf::from(&song.path);
    let created_date = path
        .metadata()
        .and_then(|metadata| metadata.created())
        .ok()
        .map(OffsetDateTime::from);

    let metadata = match read_metadata_from_path(&path) {
        Ok(song) => Some(song),
        Err(err) => {
            tracing::warn!("Failed to read tags: {}", err);
            None
        }
    };

    song_metadata_changed(song, &metadata, created_date).then_some((
        song.id.to_string(),
        metadata,
        created_date,
    ))
}

fn song_metadata_changed(
    song: &Song,
    metadata: &Option<SongMetadata>,
    created_at: Option<OffsetDateTime>,
) -> bool {
    song.title != get_metadata_field(metadata, ItemKey::Title)
        || song.album != get_metadata_field(metadata, ItemKey::Album)
        || song.album_artist != get_metadata_field(metadata, ItemKey::AlbumArtist)
        || song.disc_number != get_metadata_field(metadata, ItemKey::DiscNumber)
        || song.artist != get_metadata_field(metadata, ItemKey::Artist)
        || song.year != get_metadata_field(metadata, ItemKey::Year)
        || song.track_number != get_metadata_field(metadata, ItemKey::TrackNumber)
        || song.genre != get_metadata_field(metadata, ItemKey::Genre)
        || song.mood != get_metadata_field(metadata, ItemKey::Mood)
        || song.file_created_at != created_at
}
