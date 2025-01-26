use color_eyre::eyre::Result;
use sqlx::query;

use crate::metadata::SongMetadata;

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
            let tracks: Vec<(String, i64)> = query!("SELECT path, id FROM songs")
                .fetch_all(&db)
                .await
                .map(|rows| rows.into_iter().map(|row| (row.path, row.id)).collect())
                .map_err(|err| err.to_string())
                .unwrap_or_default();

            if tracks.is_empty() {
                tracing::warn!("No Songs found, cancelling update");
                status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                return;
            }

            for (path, id) in tracks {
                if TaskStatus::is_stopped(status.load(Ordering::Relaxed)) {
                    status.store(TaskStatus::Idle.into(), Ordering::Relaxed);
                    tracing::info!("Refresh metadata cancelled");
                    return;
                }

                let _ = update_song(db.clone(), id, path.into())
                    .await
                    .map_err(|err| {
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

async fn update_song(
    pool: sqlx::Pool<sqlx::Sqlite>,
    id: i64,
    path: PathBuf,
) -> Result<(), sqlx::Error> {
    let metadata = match SongMetadata::from_path(&path) {
        Ok(song) => Some(song),
        Err(err) => {
            tracing::warn!("Failed to read tags: {}", err);
            None
        }
    };

    tracing::info!(
        "Updating song: {}, {:?}",
        path.to_string_lossy().to_string(),
        metadata
    );

    match metadata {
        Some(song) => {
            query!(
                "UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ? WHERE id = ?",
                song.title,
                song.album,
                song.album_artist,
                song.disc_number,
                song.artist,
                song.year,
                song.track_number,
                song.genre,
                id
            )
            .execute(&pool)
            .await
            .map(|_| ())
        }
        _ => Ok(())
    }
}
