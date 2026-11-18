use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use color_eyre::{Report, Result, eyre::OptionExt};

use ts_rs::TS;

use crate::{
    db::{self, Pool, Song, albums, directories, songs},
    fs::{Operation, OperationEvent},
    metadata::{Metadata, item::ItemKey},
    organize,
    state::FileOperationManager,
};

#[derive(serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PathRenamePreviewResult {
    pub previous_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(serde::Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PathRenameOptions {
    pub rename_original_files: bool,
    pub directory_id: Option<String>,
}

impl Default for PathRenameOptions {
    fn default() -> Self {
        Self {
            rename_original_files: true,
            directory_id: None,
        }
    }
}

pub async fn organize_album_tracks(
    pool: Pool,
    file_operation_manager: FileOperationManager,
    title: String,
    options: PathRenameOptions,
) -> Result<()> {
    let mut connection = pool.acquire().await?;

    let tracks = albums::get_album_from_title(&mut connection, &title)
        .await
        .expect("Album not found")
        .get_songs(&mut connection)
        .await
        .expect("No songs found");

    let directories = directories::get_directories(&mut connection).await?;

    let tracks = tracks.iter().try_fold(HashMap::new(), |mut paths, song| {
        let directory_id = options
            .directory_id
            .as_deref()
            .unwrap_or(&song.directory_id);

        let directory: PathBuf = directories
            .iter()
            .find(|dir| dir.name == directory_id)
            .ok_or_eyre("No Directory Found")?
            .path
            .clone()
            .into();

        let path = directory.join(organize::render_song_path(
            &handlebars::Handlebars::new(),
            organize::DEFAULT_TEMPLATE,
            &map_organize(song),
            options.rename_original_files,
        )?);

        paths.insert(song.path.clone().into(), (path.clone(), song.id.clone()));

        Ok::<HashMap<PathBuf, (PathBuf, String)>, Report>(paths)
    })?;

    let mut operation_handle = file_operation_manager
        .queue_operation(Operation::Move {
            paths: tracks
                .iter()
                .map(|(from, (to, _))| (from.clone(), to.clone()))
                .collect(),
            overwrite: true,
            delete_empty_directories_after: true,
        })
        .await?;

    while let Some(item) = operation_handle.events().recv().await {
        match item {
            OperationEvent::Completed => {
                tracing::debug!("Completed");
                break;
            }
            OperationEvent::Renamed { from, to } => {
                let (_, song_id) = tracks.get(&from).expect("Path not found");

                songs::update_song_path(
                    &mut connection,
                    song_id,
                    to.to_str().expect("Path is not valid UTF-8"),
                )
                .await?;
            }
            OperationEvent::Moved { from, to } => {
                let (_, song_id) = tracks.get(&from).expect("Path not found");

                songs::update_song_path(
                    &mut connection,
                    song_id,
                    to.to_str().expect("Path is not valid UTF-8"),
                )
                .await?;
            }
            _ => continue,
        }
    }

    Ok(())
}

pub async fn preview_organize_album_tracks(
    pool: db::Pool,
    title: String,
    options: PathRenameOptions,
) -> Result<Vec<PathRenamePreviewResult>> {
    let mut connection = pool.acquire().await?;

    let tracks = albums::get_album_from_title(&mut connection, &title)
        .await?
        .get_songs(&mut connection)
        .await?;

    let directories = directories::get_directories(&mut connection).await?;

    let previews = tracks
        .iter()
        .map(|song| {
            let directory_id = options
                .directory_id
                .as_deref()
                .unwrap_or(&song.directory_id);

            let directory: PathBuf = directories
                .iter()
                .find(|dir| dir.name == directory_id)
                .ok_or_eyre("Directory not found")?
                .path
                .clone()
                .into();

            Ok(PathRenamePreviewResult {
                previous_path: PathBuf::from(&song.path),
                new_path: directory.join(organize::render_song_path(
                    &handlebars::Handlebars::new(),
                    organize::DEFAULT_TEMPLATE,
                    &map_organize(song),
                    options.rename_original_files,
                )?),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(previews)
}

#[rustfmt::skip]
fn map_organize(song: &Song) -> organize::Song {
    organize::Song {
        file_path: PathBuf::from(&song.path),
        metadata: Metadata::new(
            BTreeMap::from([
                (ItemKey::Artist, song.artist.clone().unwrap_or_default()),
                (ItemKey::Album, song.album.clone().unwrap_or_default()),
                (ItemKey::Genre, song.genre.clone().unwrap_or_default()),
                (ItemKey::Mood, song.mood.clone().unwrap_or_default()),
                (ItemKey::AlbumArtist, song.album_artist.clone().unwrap_or_default()),
                (ItemKey::Title, song.title.clone().unwrap_or_default()),
                (ItemKey::TrackNumber, song.track_number.clone().unwrap_or_default().to_string()),
                (ItemKey::DiscNumber, song.disc_number.clone().unwrap_or_default().to_string()),
                (ItemKey::Year, song.year.clone().unwrap_or_default().to_string()),
            ]),
            BTreeMap::new(),
        ),
    }
}
