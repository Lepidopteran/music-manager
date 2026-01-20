use std::{collections::BTreeMap, path::PathBuf};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::Result,
    routing::{get, post},
};
use ts_rs::TS;

use crate::{
    db::{Album, Song, songs},
    state::AppState,
};

use super::internal_error;

use handlebars::Handlebars;

const VARIOUS_ARTIST_THRESHOLD: f64 = 0.5;

// TODO: Move these templates to a files
const ARTIST_TEMPLATE: &str = "{{artist}}/{{album}}/";
const ALBUM_ARTIST_TEMPLATE: &str = "{{album_artist}}/{{album}}/";

#[derive(serde::Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PathRenamePreviewResult {
    pub previous_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(serde::Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct PathRenameOptions {
    pub threshold: f64,
    pub grouped: bool,
}

impl Default for PathRenameOptions {
    fn default() -> Self {
        PathRenameOptions {
            threshold: VARIOUS_ARTIST_THRESHOLD,
            grouped: true,
        }
    }
}

// TODO: Add ability to use a custom templates for folder structure.

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/organize/album/{title}",
        get(preview_organize_album_tracks),
    )
}

async fn preview_organize_album_tracks(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(title): Path<String>,
    Query(options): Query<PathRenameOptions>,
) -> Result<Json<Vec<PathRenamePreviewResult>>> {
    let album = songs::get_album(&db, title).await?;

    // TODO: Move handlebars registry creation to appstate.
    let handlebars = Handlebars::new();

    if options.grouped {
        let folder_path =
            generate_grouped_folder_path(&handlebars, ARTIST_TEMPLATE, &album, options.threshold)
                .map_err(internal_error)?;

        let previews = album
            .tracks
            .iter()
            .map(|song| {
                let path = PathBuf::from(song.path.clone());
                PathRenamePreviewResult {
                    previous_path: path.clone(),
                    new_path: folder_path.join(path.file_name().expect("Unable to get file name")),
                }
            })
            .collect();

        Ok(Json(previews))
    } else {
        let previews = album
            .tracks
            .iter()
            .map(|song| {
                let path = PathBuf::from(song.path.clone());
                PathRenamePreviewResult {
                    previous_path: path.clone(),
                    new_path: generate_folder_path(&handlebars, ARTIST_TEMPLATE, song)
                        .map_err(internal_error)
                        .expect("Unable to generate folder path")
                        .join(path.file_name().expect("Unable to get file name")),
                }
            })
            .collect();

        Ok(Json(previews))
    }
}

fn generate_folder_path(
    handlebar: &Handlebars,
    template: &str,
    song: &Song,
) -> color_eyre::Result<PathBuf> {
    let context: BTreeMap<&str, Option<&str>> = BTreeMap::from([
        ("artist", song.artist.as_deref()),
        ("album_artist", song.album_artist.as_deref()),
        ("genre", song.genre.as_deref()),
        ("year", song.year.as_deref()),
        ("album", song.album.as_deref()),
        ("mood", song.mood.as_deref()),
    ]);

    Ok(PathBuf::from(
        handlebar.render_template(template, &context)?,
    ))
}

fn generate_grouped_folder_path(
    handlebar: &Handlebars,
    template: &str,
    album: &Album,
    various_artist_percentage_threshold: f64,
) -> color_eyre::Result<PathBuf> {
    let artist = if album.tracks.iter().all(|song| song.artist.is_none()) {
        "Unknown Artist"
    } else {
        let songs_with_artists = album
            .tracks
            .iter()
            .filter(|song| song.artist.is_some())
            .collect::<Vec<_>>();

        songs_with_artists
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut acc, song| {
                *acc.entry(song.artist.as_ref().expect("Artist not found"))
                    .or_insert(0) += 1;
                acc
            })
            .into_iter()
            .rev()
            .max_by_key(|(_, count)| *count)
            .and_then(|(artist, count)| {
                let percentage = count as f64 / songs_with_artists.len() as f64;
                (percentage > various_artist_percentage_threshold).then_some(artist)
            })
            .unwrap_or("Various Artists")
    }
    .to_string();

    let Song {
        genre, year, mood, ..
    } = album.tracks.first().expect("No songs found");

    let context: BTreeMap<&str, Option<&str>> = BTreeMap::from([
        ("artist", Some(artist.as_str())),
        ("album_artist", album.artist.as_deref()),
        ("genre", genre.as_deref()),
        ("year", year.as_deref()),
        ("album", Some(&album.title)),
        ("mood", mood.as_deref()),
    ]);

    Ok(PathBuf::from(
        handlebar.render_template(template, &context)?,
    ))
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn test_generate_folder_path() {
        let handlebar = Handlebars::new();

        let album = Album::from(vec![
            Song {
                artist: Some("Artist".to_string()),
                album_artist: Some("Album Artist".to_string()),
                genre: Some("Genre".to_string()),
                year: Some("Year".to_string()),
                album: Some("Album".to_string()),
                disc_number: Some("Disc Number".to_string()),
                track_number: Some("Track Number".to_string()),
                mood: Some("Mood".to_string()),
                ..Default::default()
            },
            Song {
                artist: Some("Artist Feat. Another Artist".to_string()),
                album_artist: Some("Album Artist".to_string()),
                genre: Some("Genre".to_string()),
                year: Some("Year".to_string()),
                album: Some("Album".to_string()),
                disc_number: Some("Disc Number".to_string()),
                track_number: Some("Track Number".to_string()),
                mood: Some("Mood".to_string()),
                ..Default::default()
            },
            Song {
                album_artist: Some("Album Artist".to_string()),
                genre: Some("Genre".to_string()),
                year: Some("Year".to_string()),
                album: Some("Album".to_string()),
                disc_number: Some("Disc Number".to_string()),
                track_number: Some("Track Number".to_string()),
                mood: Some("Mood".to_string()),
                ..Default::default()
            },
        ]);

        let result =
            generate_grouped_folder_path(&handlebar, ARTIST_TEMPLATE, &album, 0.25).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Artist/Album/"));

        let result =
            generate_grouped_folder_path(&handlebar, ARTIST_TEMPLATE, &album, 0.5).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Various Artists/Album/"));

        let result =
            generate_grouped_folder_path(&handlebar, ALBUM_ARTIST_TEMPLATE, &album, 0.25).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Album Artist/Album/"));
    }
}
