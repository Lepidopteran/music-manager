use std::{collections::BTreeMap, path::PathBuf};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use color_eyre::eyre::Result;

use crate::{api::albums::Album, app::AppState, db::Song};
use handlebars::Handlebars;

pub fn router() -> Router<AppState> {
    Router::new()
}

fn generate_folder_path_from_album(
    handlebar: &Handlebars,
    template: &str,
    album: &Album,
    various_artist_percentage_threshold: f64,
) -> Result<PathBuf> {
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

                log::debug!(
                    "{}: {}, {}",
                    artist,
                    percentage,
                    percentage >= various_artist_percentage_threshold
                );
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

    Ok(PathBuf::from(handlebar.render(template, &context)?))
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    const ARTIST_TEMPLATE: &str = "{{artist}}/{{album}}/";
    const ALBUM_ARTIST_TEMPLATE: &str = "{{album_artist}}/{{album}}/";

    #[test]
    fn test_generate_folder_path() {
        let mut handlebar = Handlebars::new();

        handlebar
            .register_template_string("artist", ARTIST_TEMPLATE)
            .unwrap();
        handlebar
            .register_template_string("album_artist", ALBUM_ARTIST_TEMPLATE)
            .unwrap();

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

        let result = generate_folder_path_from_album(&handlebar, "artist", &album, 0.25).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Artist/Album/"));

        let result = generate_folder_path_from_album(&handlebar, "artist", &album, 0.5).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Various Artists/Album/"));

        let result =
            generate_folder_path_from_album(&handlebar, "album_artist", &album, 0.25).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Album Artist/Album/"));
    }
}
