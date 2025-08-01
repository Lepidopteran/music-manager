use color_eyre::eyre::Result;
use sqlx::{query, query_as};
use tokio::task::spawn_blocking;

use crate::db::Song;
use metadata::SongMetadata;

use super::*;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

// TODO: add progress information

pub struct UpdateSongs {
    info: TaskInfo,
    status: Arc<AtomicU8>,
    db: sqlx::Pool<sqlx::Sqlite>,
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

        status.store(TaskStatus::Running.into(), Ordering::Relaxed);
        tokio::spawn(async move {
            let tracks: Vec<Song> = query_as!(Song, "SELECT * FROM songs")
                .fetch_all(&db)
                .await
                .expect("Failed to get tracks");

            if tracks.is_empty() {
                tracing::warn!("No Songs found, cancelling update");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            let comparison_tasks = tracks
                .into_iter()
                .map(|song| spawn_blocking(move || get_updated_metadata(&song)));

            let updated_tracks = futures::future::join_all(comparison_tasks)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .flatten()
                .collect::<Vec<(i64, SongMetadata)>>();

            if updated_tracks.is_empty() {
                tracing::info!("No Songs need updating, cancelling update");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            for (id, metadata) in updated_tracks {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    tracing::info!("Refresh metadata cancelled");
                    return;
                }

                let _ = update_song(db.clone(), id, metadata).await.map_err(|err| {
                    tracing::error!("Failed to update song: {}", err);
                });
            }

            status.store(TaskStatus::Idle.into(), Ordering::Relaxed);

            tracing::info!("Song update complete");
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
