use color_eyre::eyre::Result;
use sqlx::{query, query_as};
use time::OffsetDateTime;
use tokio::{
    sync::watch::{channel, Receiver, Sender},
    task::spawn_blocking,
};

use crate::{
    db::Song,
    metadata::{item::ItemKey, read_metadata_from_path, Metadata as SongMetadata},
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

pub struct UpdateSongs {
    info: TaskInfo,
    status: Arc<AtomicU8>,
    started_at: Arc<RwLock<Option<OffsetDateTime>>>,
    completed_at: Arc<RwLock<Option<OffsetDateTime>>>,
    cancelled_at: Arc<RwLock<Option<OffsetDateTime>>>,
    db: sqlx::Pool<sqlx::Sqlite>,
    channel: (Sender<TaskEvent>, Receiver<TaskEvent>),
}

impl UpdateSongs {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self {
            db: pool,
            status: Arc::new(AtomicU8::from(TaskStatus::default() as u8)),
            started_at: Arc::new(RwLock::new(None)),
            completed_at: Arc::new(RwLock::new(None)),
            cancelled_at: Arc::new(RwLock::new(None)),
            info: TaskInfo {
                id: "update-songs".to_string(),
                name: "Update Songs".to_string(),
                description: "Updates every song's metadata in the database".to_string(),
                steps: 2,
            },
            channel: channel(TaskEvent::initial("update-songs")),
        }
    }
}

impl Task for UpdateSongs {
    fn start(&mut self) -> Result<(), TaskError> {
        let status = self.status.clone();
        if TaskStatus::is_running(status.load(Ordering::Relaxed)) {
            return Err(TaskError::Running);
        }

        let db = self.db.clone();
        let (tx, _) = self.channel.clone();
        let started_at = self.started_at.clone();
        let completed_at = self.completed_at.clone();
        let cancelled_at = self.cancelled_at.clone();

        status.store(TaskStatus::Running.into(), Ordering::Relaxed);
        tokio::spawn(async move {
            let _ = tx.send(TaskEvent::start("Starting song update"));
            let _ = started_at
                .write()
                .expect("Failed to set started_at")
                .replace(OffsetDateTime::now_utc());

            let tracks: Vec<Song> = query_as!(Song, "SELECT * FROM songs")
                .fetch_all(&db)
                .await
                .expect("Failed to get tracks");

            if tracks.is_empty() {
                let _ = tx.send(TaskEvent::stop("No Songs found, cancelling update"));
                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());

                tracing::warn!("No Songs found, cancelling update");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let song_count = tracks.len();
            let _ = tx.send(TaskEvent::info(
                format!("Found {song_count} song(s), comparing metadata...").as_str(),
            ));

            let comparison_tx = tx.clone();
            let comparison_tasks = tracks.into_iter().enumerate().map(move |(index, song)| {
                let tx = comparison_tx.clone();
                spawn_blocking(move || {
                    if index % 50 == 0 {
                        let _ = tx.send(TaskEvent::progress(
                            format!("Comparing metadata {}%", index * 100 / song_count).as_str(),
                            index as u64,
                            song_count as u64,
                            Some(1),
                        ));
                    }

                    get_updated_metadata(&song)
                })
            });

            let updated_tracks = futures::future::join_all(comparison_tasks)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .flatten()
                .collect::<Vec<(String, SongMetadata, _)>>();

            if updated_tracks.is_empty() {
                tracing::info!("No Songs need updating, cancelling update");
                let _ = tx.send(TaskEvent::stop("No Songs need updating, cancelling update"));
                let _ = cancelled_at
                    .write()
                    .expect("Failed to set cancelled_at")
                    .replace(OffsetDateTime::now_utc());

                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let song_count = updated_tracks.len();
            let _ = tx.send(TaskEvent::info(
                format!("Updating {song_count} song(s)...").as_str(),
            ));

            let mut db_tx = db.begin().await.expect("Failed to begin transaction");

            for (index, (id, metadata, created_at)) in updated_tracks.iter().cloned().enumerate() {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    tracing::info!("Refresh metadata cancelled");
                    return;
                }

                let result = query("UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ?, mood = ?, file_created_at = ? WHERE id = ?")
                    .bind(metadata.get(&ItemKey::Title).cloned())
                    .bind(metadata.get(&ItemKey::Album).cloned())
                    .bind(metadata.get(&ItemKey::AlbumArtist).cloned())
                    .bind(metadata.get(&ItemKey::DiscNumber).cloned())
                    .bind(metadata.get(&ItemKey::Artist).cloned())
                    .bind(metadata.get(&ItemKey::Year).cloned())
                    .bind(metadata.get(&ItemKey::TrackNumber).cloned())
                    .bind(metadata.get(&ItemKey::Genre).cloned())
                    .bind(metadata.get(&ItemKey::Mood).cloned())
                    .bind(created_at)
                    .bind(id)
                    .execute(&mut *db_tx)
                    .await;

                if let Err(err) = result {
                    tracing::error!("Failed to update song: {}", err);
                } 

                if index % 100 == 0 {
                    let _ = tx.send(TaskEvent::progress(
                        format!("Updating songs with new metadata {}%", index * 100 / song_count).as_str(),
                        index as u64,
                        song_count as u64,
                        Some(2),
                    ));
                }
            }

            tx.send(TaskEvent::progress(
                format!("Updated {song_count} song(s)... saving changes").as_str(),
                song_count as u64,
                song_count as u64,
                Some(2),
            ))
            .unwrap();

            db_tx.commit().await.expect("Failed to commit transaction");

            status.store(TaskStatus::Idle.into(), Ordering::Relaxed);

            tracing::info!("Song update complete");
            let _ = tx.send(TaskEvent::complete("Song update complete"));
            let _ = completed_at
                .write()
                .expect("Failed to set completed_at")
                .replace(OffsetDateTime::now_utc());
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
                .expect("Failed to read cancelled_at"),
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
        tracing::info!("Cancel request received, cancelling update");
        Ok(())
    }

    fn info(&self) -> &TaskInfo {
        &self.info
    }

    fn channel(&self) -> Option<Receiver<TaskEvent>> {
        Some(self.channel.1.clone())
    }
}

fn get_updated_metadata(song: &Song) -> Option<(String, SongMetadata, Option<OffsetDateTime>)> {
    let path = PathBuf::from(&song.path);
    tracing::debug!("Getting metadata for: {}", path.display());

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

    metadata.and_then(|metadata| {
        if song_metadata_changed(song, &metadata, created_date) {
            Some((song.id.to_string(), metadata, created_date))
        } else {
            None
        }
    })
}

fn song_metadata_changed(
    song: &Song,
    metadata: &SongMetadata,
    created_at: Option<OffsetDateTime>,
) -> bool {
    song.title != metadata.get(&ItemKey::Title).cloned()
        || song.album != metadata.get(&ItemKey::Album).cloned()
        || song.album_artist != metadata.get(&ItemKey::AlbumArtist).cloned()
        || song.disc_number != metadata.get(&ItemKey::DiscNumber).cloned()
        || song.artist != metadata.get(&ItemKey::Artist).cloned()
        || song.year != metadata.get(&ItemKey::Year).cloned()
        || song.track_number != metadata.get(&ItemKey::TrackNumber).cloned()
        || song.genre != metadata.get(&ItemKey::Genre).cloned()
        || song.mood != metadata.get(&ItemKey::Mood).cloned()
        || song.file_created_at != created_at
}

async fn update_song(
    pool: sqlx::Pool<sqlx::Sqlite>,
    id: &str,
    metadata: SongMetadata,
    created_at: Option<OffsetDateTime>,
) -> Result<(), sqlx::Error> {
    tracing::info!("Updating song: {id}, {:#?}", metadata);

    let title = metadata.get(&ItemKey::Title).cloned();
    let album = metadata.get(&ItemKey::Album).cloned();
    let album_artist = metadata.get(&ItemKey::AlbumArtist).cloned();
    let disc_number = metadata.get(&ItemKey::DiscNumber).cloned();
    let artist = metadata.get(&ItemKey::Artist).cloned();
    let year = metadata.get(&ItemKey::Year).cloned();
    let track_number = metadata.get(&ItemKey::TrackNumber).cloned();
    let genre = metadata.get(&ItemKey::Genre).cloned();
    let mood = metadata.get(&ItemKey::Mood).cloned();

    query!(
        "UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ?, mood = ?, file_created_at = ? WHERE id = ?",
        title,
        album,
        album_artist,
        disc_number,
        artist,
        year,
        track_number,
        genre,
        mood,
        created_at,
        id
    )
    .execute(&pool)
    .await
    .map(|_| ())
}
