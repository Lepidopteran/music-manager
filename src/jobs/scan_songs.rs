use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use color_eyre::eyre::Result;
use sqlx::query_as;
use time::OffsetDateTime;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;

use crate::{
    db::{self, Song},
    metadata::{item::ItemKey, read_metadata_from_path},
    state::job::JobInfo,
};

use super::*;

const SONG_FILE_TYPES: [&str; 8] = ["mp3", "m4a", "flac", "wav", "ogg", "wma", "aac", "opus"];

#[derive(Debug)]
pub struct ScanSongs {
    db: sqlx::Pool<sqlx::Sqlite>,
}

impl ScanSongs {
    pub fn new(db: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { db }
    }

    pub fn job_info() -> JobInfo {
        JobInfo::new(
            "Scan Songs",
            "Scans for new songs, updates existing ones, and deletes songs that no longer exist",
            BTreeMap::from([
                (1, String::from("Scanning for songs to delete")),
                (2, String::from("Scanning for new songs")),
                (3, String::from("Scanning for updated songs")),
                (4, String::from("Applying and saving changes")),
            ]),
        )
    }
}

#[async_trait]
impl JobHandle for ScanSongs {
    async fn execute(&self, token: CancellationToken, tx: &Sender) -> Result<()> {
        let directories =
            sqlx::query_as::<_, (String, String)>("SELECT path, name FROM directories")
                .fetch_all(&self.db)
                .await
                .unwrap_or_default();

        if directories.is_empty() {
            tracing::warn!("No directories found, cancelling scan");

            return Ok(());
        }

        let message = format!("Found {} directory(s)", directories.len());
        tracing::info!(message);

        let existing_songs = query_as!(Song, "SELECT * FROM songs")
            .fetch_all(&self.db)
            .await
            .unwrap_or_default();

        let non_existing_song_ids = existing_songs
            .iter()
            .cloned()
            .filter_map(|song| (!PathBuf::from(&song.path).exists()).then_some(song.id))
            .collect::<HashSet<_>>();

        tx.send(JobEvent::StepCompleted {
            step: 1,
            value: non_existing_song_ids.len().to_string().into(),
        })
        .await?;

        let existing_song_paths = existing_songs
            .iter()
            .map(|song| PathBuf::from(&song.path.clone()))
            .collect::<HashSet<_>>();

        let tx_clone = tx.clone();
        let directories_clone = directories.clone();
        let block_token = token.child_token();
        let song_paths = spawn_blocking(move || {
            let (tx, file_rx) = std::sync::mpsc::channel();
            let mut directories = directories_clone.iter();
            while let Some((path, _)) = directories.next()
                && !block_token.is_cancelled()
            {
                ignore::WalkBuilder::new(path)
                    .add_custom_ignore_filename(".muusik-ignore")
                    .add_custom_ignore_filename(".muusik_ignore")
                    .add_custom_ignore_filename(".muusikignore")
                    .hidden(false)
                    .follow_links(true)
                    .build_parallel()
                    .run(|| {
                        let child_token = block_token.child_token();
                        let existing_song_paths = existing_song_paths.clone();
                        let event_channel = tx_clone.clone();

                        let file_tx = tx.clone();
                        Box::new(move |result| {
                            use ignore::WalkState::*;
                            if child_token.is_cancelled() {
                                return Quit;
                            }

                            if let Ok(entry) = result.inspect_err(|err| {
                                let message = format!("Skipping entry due to error: {err}");
                                tracing::warn!(message);
                                let _ = event_channel.blocking_send(JobEvent::Warning { message });
                            }) && entry
                                .file_type()
                                .is_some_and(|file_type| file_type.is_file())
                                && !existing_song_paths.contains(entry.path())
                                && entry
                                    .path()
                                    .extension()
                                    .and_then(|ext| ext.to_str())
                                    .is_some_and(|ext| {
                                        SONG_FILE_TYPES.contains(&ext.to_lowercase().as_str())
                                    })
                                && let Err(err) = file_tx.send(entry.path().to_path_buf())
                            {
                                tracing::error!("Failed to send file to channel: {err}");
                            }

                            Continue
                        })
                    });
            }

            drop(tx);

            file_rx.into_iter().collect::<Vec<PathBuf>>()
        })
        .await
        .expect("Failed to join thread");

        tx.send(JobEvent::StepCompleted {
            step: 2,
            value: song_paths.len().to_string().into(),
        })
        .await?;

        if token.is_cancelled() {
            return Ok(());
        }

        let existing_song_count = existing_songs.len();
        let comparison_tx = tx.clone();
        let comparison_tasks = existing_songs
            .into_iter()
            .filter(|song| !non_existing_song_ids.contains(&song.id))
            .enumerate()
            .map(move |(index, song)| {
                let tx = comparison_tx.clone();
                let tx_clone = tx.clone();
                spawn_blocking(move || {
                    let _ = tx.blocking_send(JobEvent::Progress {
                        current: index as u64,
                        total: existing_song_count as u64,
                        step: 3,
                    });

                    let path = PathBuf::from(&song.path);
                    let metadata = match read_metadata_from_path(&path) {
                        Ok(song) => Some(song),
                        Err(err) => {
                            let _ = tx_clone.blocking_send(JobEvent::Warning {
                                message: format!("Failed to read metadata for song: {err}"),
                            });
                            None
                        }
                    };

                    let created_date = path
                        .metadata()
                        .and_then(|metadata| metadata.created())
                        .ok()
                        .map(OffsetDateTime::from);

                    let metadata_ref = metadata.as_ref();
                    if song.title.as_ref() != metadata_ref.and_then(|m| m.get(&ItemKey::Title))
                        || song.album.as_ref() != metadata_ref.and_then(|m| m.get(&ItemKey::Album))
                        || song.year.as_ref() != metadata_ref.and_then(|m| m.get(&ItemKey::Year))
                        || song.genre.as_ref() != metadata_ref.and_then(|m| m.get(&ItemKey::Genre))
                        || song.mood.as_ref() != metadata_ref.and_then(|m| m.get(&ItemKey::Mood))
                        || song.file_created_at != created_date
                        || song.album_artist.as_ref()
                            != metadata_ref.and_then(|m| m.get(&ItemKey::AlbumArtist))
                        || song.track_number.as_ref()
                            != metadata_ref.and_then(|m| m.get(&ItemKey::TrackNumber))
                        || song.artist.as_ref()
                            != metadata_ref.and_then(|m| m.get(&ItemKey::Artist))
                        || song.disc_number.as_ref()
                            != metadata_ref.and_then(|m| m.get(&ItemKey::DiscNumber))
                    {
                        Some((song.id.to_string(), metadata))
                    } else {
                        None
                    }
                })
            });

        let updated_songs = futures::future::join_all(comparison_tasks)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect::<Vec<(_, _)>>();

        if !updated_songs.is_empty() {
            tracing::info!("Found {} updated song(s)...", updated_songs.len());
        } else {
            tracing::info!("No updated song(s) found...");
        }

        if song_paths.is_empty() && non_existing_song_ids.is_empty() && updated_songs.is_empty() {
            tracing::warn!("No changes found, stopping task...");

            return Ok(());
        }

        tx.send(JobEvent::StepCompleted {
            step: 3,
            value: updated_songs.len().to_string().into(),
        })
        .await?;

        let change_count =
            (song_paths.len() + updated_songs.len() + non_existing_song_ids.len()) as u64;

        let mut transaction = self.db.begin().await?;
        let mut current_change_index = 0;

        for song in song_paths.iter() {
            if token.is_cancelled() {
                break;
            }

            let path_buf = song.to_path_buf();
            let metadata = spawn_blocking(move || read_metadata_from_path(&path_buf).ok()).await?;

            let file_created_at = tokio::fs::metadata(song)
                .await?
                .created()
                .map(OffsetDateTime::from)
                .ok();

            let metadata = metadata.as_ref();
            if let Err(err) = db::songs::add_song(
                &mut transaction,
                db::NewSong {
                    path: song.to_string_lossy().to_string(),
                    title: metadata.and_then(|m| m.get(&ItemKey::Title)).cloned(),
                    artist: metadata.and_then(|m| m.get(&ItemKey::Artist)).cloned(),
                    album: metadata.and_then(|m| m.get(&ItemKey::Album)).cloned(),
                    album_artist: metadata.and_then(|m| m.get(&ItemKey::AlbumArtist)).cloned(),
                    genre: metadata.and_then(|m| m.get(&ItemKey::Genre)).cloned(),
                    track_number: metadata.and_then(|m| m.get(&ItemKey::TrackNumber)).cloned(),
                    disc_number: metadata.and_then(|m| m.get(&ItemKey::DiscNumber)).cloned(),
                    year: metadata.and_then(|m| m.get(&ItemKey::Year)).cloned(),
                    mood: metadata.and_then(|m| m.get(&ItemKey::Mood)).cloned(),
                    file_created_at,
                },
            )
            .await
            {
                tracing::error!("Song scan error: {err}");
            }

            current_change_index += 1;

            tx.send(JobEvent::Progress {
                current: current_change_index,
                total: change_count,
                step: 4,
            })
            .await?;
        }

        if token.is_cancelled() {
            return Ok(());
        }

        for (song_id, metadata) in updated_songs {
            if token.is_cancelled() {
                break;
            }

            let metadata = metadata.as_ref();
            if let Err(err) = db::songs::update_song(
                &mut transaction,
                &song_id,
                db::UpdatedSong {
                    title: metadata.and_then(|m| m.get(&ItemKey::Title)).cloned(),
                    album: metadata.and_then(|m| m.get(&ItemKey::Album)).cloned(),
                    album_artist: metadata.and_then(|m| m.get(&ItemKey::AlbumArtist)).cloned(),
                    disc_number: metadata.and_then(|m| m.get(&ItemKey::DiscNumber)).cloned(),
                    artist: metadata.and_then(|m| m.get(&ItemKey::Artist)).cloned(),
                    year: metadata.and_then(|m| m.get(&ItemKey::Year)).cloned(),
                    track_number: metadata.and_then(|m| m.get(&ItemKey::TrackNumber)).cloned(),
                    genre: metadata.and_then(|m| m.get(&ItemKey::Genre)).cloned(),
                    mood: metadata.and_then(|m| m.get(&ItemKey::Mood)).cloned(),
                },
            )
            .await
            {
                tracing::error!("Song scan error: {err}");
            }

            current_change_index += 1;

            tx.send(JobEvent::Progress {
                current: current_change_index,
                total: change_count,
                step: 4,
            })
            .await?;
        }

        if token.is_cancelled() {
            return Ok(());
        }

        for song_id in non_existing_song_ids {
            if token.is_cancelled() {
                break;
            }

            if let Err(err) = db::songs::delete_song(&mut transaction, &song_id).await {
                tracing::error!("Song scan error: {err}");
            }

            current_change_index += 1;

            tx.send(JobEvent::Progress {
                current: current_change_index,
                total: change_count,
                step: 4,
            })
            .await?;
        }

        if token.is_cancelled() {
            return Ok(());
        }

        if let Err(err) = transaction.commit().await {
            tracing::error!("Song scan error: {err}");
        }

        tx.send(JobEvent::StepCompleted {
            step: 4,
            value: None,
        })
        .await?;

        tracing::info!("Finished song scans...");

        Ok(())
    }
}
