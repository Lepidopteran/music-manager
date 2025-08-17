use color_eyre::eyre::Result;
use sqlx::{query, query_as};
use tokio::{
    sync::watch::{channel, Receiver, Sender},
    task::spawn_blocking,
};

use crate::{db::Song, metadata::SongMetadata, task::TaskEvent};

use super::*;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

pub struct UpdateSongs {
    info: TaskInfo,
    status: Arc<AtomicU8>,
    db: sqlx::Pool<sqlx::Sqlite>,
    channel: (Sender<TaskEvent>, Receiver<TaskEvent>),
}

impl UpdateSongs {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self {
            db: pool,
            status: Arc::new(AtomicU8::from(TaskStatus::default() as u8)),
            info: TaskInfo {
                id: "update-songs".to_string(),
                name: "Update Songs".to_string(),
                description: "Updates every song's metadata in the database".to_string(),
            },
            channel: channel(TaskEvent::default()),
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

        status.store(TaskStatus::Running.into(), Ordering::Relaxed);
        tokio::spawn(async move {
            let _ = tx.send(TaskEvent::info("Starting song update"));

            let tracks: Vec<Song> = query_as!(Song, "SELECT * FROM songs")
                .fetch_all(&db)
                .await
                .expect("Failed to get tracks");

            if tracks.is_empty() {
                let _ = tx.send(TaskEvent::warning("No Songs found, cancelling update"));

                tracing::warn!("No Songs found, cancelling update");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let song_count = tracks.len();
            let _ = tx.send(TaskEvent::info(
                format!("Found {} song(s), comparing metadata...", song_count).as_str(),
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
                .collect::<Vec<(i64, SongMetadata)>>();

            if updated_tracks.is_empty() {
                tracing::info!("No Songs need updating, cancelling update");
                let _ = tx.send(TaskEvent::info("No Songs need updating, cancelling update"));
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let song_count = updated_tracks.len();
            let _ = tx.send(TaskEvent::info(
                format!("Updating {} song(s)...", song_count).as_str(),
            ));

            for (id, metadata) in updated_tracks {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    tracing::info!("Refresh metadata cancelled");
                    return;
                }

                if let Err(err) = update_song(db.clone(), id, metadata).await {
                    tracing::error!("Failed to update song: {}", err);
                }
            }

            status.store(TaskStatus::Idle.into(), Ordering::Relaxed);

            tracing::info!("Song update complete");
            let _ = tx.send(TaskEvent::complete("Song update complete"));
        });

        Ok(())
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

    fn status(&self) -> TaskStatus {
        TaskStatus::from(self.status.clone().load(Ordering::Relaxed))
    }

    fn info(&self) -> &TaskInfo {
        &self.info
    }

    fn channel(&self) -> Option<Receiver<TaskEvent>> {
        Some(self.channel.1.clone())
    }
}

fn get_updated_metadata(song: &Song) -> Option<(i64, SongMetadata)> {
    let path = PathBuf::from(&song.path);
    tracing::debug!("Getting metadata for: {}", path.display());

    let metadata = match SongMetadata::from_path(&path) {
        Ok(song) => Some(song),
        Err(err) => {
            tracing::warn!("Failed to read tags: {}", err);
            None
        }
    };

    metadata.and_then(|metadata| {
        if song_metadata_changed(song, &metadata) {
            Some((song.id, metadata))
        } else {
            None
        }
    })
}

fn song_metadata_changed(song: &Song, metadata: &SongMetadata) -> bool {
    song.title != metadata.title
        || song.album != metadata.album
        || song.album_artist != metadata.album_artist
        || song.disc_number != metadata.disc_number
        || song.artist != metadata.artist
        || song.year != metadata.year
        || song.track_number != metadata.track_number
        || song.genre != metadata.genre
        || song.mood != metadata.mood
}

async fn update_song(
    pool: sqlx::Pool<sqlx::Sqlite>,
    id: i64,
    metadata: SongMetadata,
) -> Result<(), sqlx::Error> {
    tracing::info!("Updating song: {id}, {:?}", metadata);

    query!(
        "UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ?, mood = ? WHERE id = ?",
        metadata.title,
        metadata.album,
        metadata.album_artist,
        metadata.disc_number,
        metadata.artist,
        metadata.year,
        metadata.track_number,
        metadata.genre,
        metadata.mood,
        id
    )
    .execute(&pool)
    .await
    .map(|_| ())
}
