use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Add,
    path::PathBuf,
};
use ts_rs::TS;

use crate::metadata::{Metadata, TAG_SEPARATOR, item::ItemKey};

use super::metadata;

use handlebars::Handlebars;

// TODO: Move these templates to a files
pub const ARTIST_TEMPLATE: &str = "{{artist}}/{{album}}/";
pub const ALBUM_ARTIST_TEMPLATE: &str = "{{albumArtist}}/{{album}}/";

const VARIOUS_THRESHOLD: f64 = 0.5;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum OrganizeError {
    #[error("Handlebars render error: {0}")]
    Handlebars(#[from] handlebars::RenderError),
}

pub type Result<T, E = OrganizeError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OrganizableSong")]
pub struct Song {
    pub file_path: PathBuf,
    #[serde(flatten)]
    pub metadata: metadata::Metadata,
}

pub fn render_song_path(handlebar: &Handlebars, template: &str, song: &Song) -> Result<PathBuf> {
    Ok(PathBuf::from(handlebar.render_template(template, &song)?))
}

fn render_grouped_folder_path(
    handlebar: &Handlebars,
    template: &str,
    album: &Vec<Song>,
) -> Result<PathBuf> {
    let mut value_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut key_values: BTreeMap<ItemKey, BTreeSet<String>> = BTreeMap::new();

    for song in album {
        for (key, value) in song.metadata.fields() {
            for part in value.split(TAG_SEPARATOR) {
                let _ = key_values
                    .entry(key.clone())
                    .or_default()
                    .insert(part.to_string());

                let _ = value_counts
                    .entry(part.to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
    }

    let mut common_metadata: BTreeMap<ItemKey, String> = BTreeMap::new();

    for (key, values) in key_values {
        let biggest_value = values
            .iter()
            .max_by_key(|v| value_counts.get(v.as_str()).unwrap_or(&0))
            .expect("No value found...");

        let biggest_value_count = value_counts.get(biggest_value).unwrap_or(&0);

        let percentage = *biggest_value_count as f64 / album.len() as f64;

        log::debug!("Key: {key:?} Value: {biggest_value} Percentage: {percentage}",);

        if percentage <= VARIOUS_THRESHOLD {
            common_metadata.insert(key, "Various".to_string());
        } else {
            common_metadata.insert(key, biggest_value.to_string());
        }
    }

    Ok(PathBuf::from(
        handlebar.render_template(template, &common_metadata)?,
    ))
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn test_render_grouped_folder_path() {
        let handlebar = Handlebars::new();

        let album = vec![
            Song {
                file_path: PathBuf::from("Artist Feat. Another Artist/Album/Track Name.mp3"),
                metadata: metadata::Metadata::new(
                    BTreeMap::from([
                        (ItemKey::Album, "Album".to_string()),
                        (ItemKey::Artist, "Artist".to_string()),
                        (ItemKey::AlbumArtist, "Album Artist".to_string()),
                        (ItemKey::Genre, "Genre".to_string()),
                        (ItemKey::Year, "Year".to_string()),
                        (ItemKey::DiscNumber, "Disc Number".to_string()),
                        (ItemKey::TrackNumber, "Track Number".to_string()),
                        (ItemKey::Mood, "Mood".to_string()),
                    ]),
                    BTreeMap::new(),
                ),
            },
            Song {
                file_path: PathBuf::from("Artist Feat. Another Artist/Album/Track Name.mp3"),
                metadata: metadata::Metadata::new(
                    BTreeMap::from([
                        (ItemKey::Album, "Album".to_string()),
                        (ItemKey::Artist, "Artist".to_string()),
                        (ItemKey::AlbumArtist, "Album Artist".to_string()),
                        (ItemKey::Genre, "Genre".to_string()),
                        (ItemKey::Year, "Year".to_string()),
                        (ItemKey::DiscNumber, "Disc Number".to_string()),
                        (ItemKey::TrackNumber, "Track Number".to_string()),
                        (ItemKey::Mood, "Mood".to_string()),
                    ]),
                    BTreeMap::new(),
                ),
            },
            Song {
                file_path: PathBuf::from("Artist Feat. Another Artist/Album/Track Name.mp3"),
                metadata: metadata::Metadata::new(
                    BTreeMap::from([
                        (ItemKey::Album, "Album".to_string()),
                        (ItemKey::Artist, "Artist Feat. Another Artist".to_string()),
                        (ItemKey::AlbumArtist, "Album Artist".to_string()),
                        (ItemKey::Genre, "Genre".to_string()),
                        (ItemKey::Year, "Year".to_string()),
                        (ItemKey::DiscNumber, "Disc Number".to_string()),
                        (ItemKey::TrackNumber, "Track Number".to_string()),
                        (ItemKey::Mood, "Mood".to_string()),
                    ]),
                    BTreeMap::new(),
                ),
            },
        ];

        let result = render_grouped_folder_path(&handlebar, ARTIST_TEMPLATE, &album).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Artist/Album/"));

        let result = render_grouped_folder_path(&handlebar, ALBUM_ARTIST_TEMPLATE, &album).unwrap();
        log::debug!("Folder path: {}", result.display());

        assert_eq!(result, PathBuf::from("Album Artist/Album/"));
    }
}
