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
pub const ALBUM_ARTIST_TEMPLATE: &str = "{{album_artist}}/{{album}}/";

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
    album: Vec<Song>,
    various_percentage_threshold: f64,
) -> Result<PathBuf> {
    let mut value_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut key_values: BTreeMap<ItemKey, BTreeSet<String>> = BTreeMap::new();

    for song in &album {
        for (key, value) in song.metadata.fields() {
            for part in value.split(TAG_SEPARATOR) {
                if key_values
                    .entry(key.clone())
                    .or_default()
                    .insert(part.to_string())
                {
                    let _ = value_counts.entry(part.to_string()).or_default().add(1);
                }
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

        if *biggest_value_count == 1 {
            continue;
        }

        common_metadata.insert(key, biggest_value.to_string());
    }

    Ok(PathBuf::from(
        handlebar.render_template(template, &common_metadata)?,
    ))
}

// #[cfg(test)]
// mod tests {
//     use test_log::test;
//
//     use super::*;
//
//     #[test]
//     fn test_render_grouped_folder_path() {
//         let handlebar = Handlebars::new();
//
//         let album = Album::from(vec![
//             Song {
//                 artist: Some("Artist".to_string()),
//                 album_artist: Some("Album Artist".to_string()),
//                 genre: Some("Genre".to_string()),
//                 year: Some("Year".to_string()),
//                 album: Some("Album".to_string()),
//                 disc_number: Some("Disc Number".to_string()),
//                 track_number: Some("Track Number".to_string()),
//                 mood: Some("Mood".to_string()),
//                 ..Default::default()
//             },
//             Song {
//                 artist: Some("Artist Feat. Another Artist".to_string()),
//                 album_artist: Some("Album Artist".to_string()),
//                 genre: Some("Genre".to_string()),
//                 year: Some("Year".to_string()),
//                 album: Some("Album".to_string()),
//                 disc_number: Some("Disc Number".to_string()),
//                 track_number: Some("Track Number".to_string()),
//                 mood: Some("Mood".to_string()),
//                 ..Default::default()
//             },
//             Song {
//                 album_artist: Some("Album Artist".to_string()),
//                 genre: Some("Genre".to_string()),
//                 year: Some("Year".to_string()),
//                 album: Some("Album".to_string()),
//                 disc_number: Some("Disc Number".to_string()),
//                 track_number: Some("Track Number".to_string()),
//                 mood: Some("Mood".to_string()),
//                 ..Default::default()
//             },
//         ]);
//
//         let result =
//             generate_grouped_folder_path(&handlebar, ARTIST_TEMPLATE, &album, 0.25).unwrap();
//         log::debug!("Folder path: {}", result.display());
//
//         assert_eq!(result, PathBuf::from("Artist/Album/"));
//
//         let result =
//             generate_grouped_folder_path(&handlebar, ARTIST_TEMPLATE, &album, 0.5).unwrap();
//         log::debug!("Folder path: {}", result.display());
//
//         assert_eq!(result, PathBuf::from("Various Artists/Album/"));
//
//         let result =
//             generate_grouped_folder_path(&handlebar, ALBUM_ARTIST_TEMPLATE, &album, 0.25).unwrap();
//         log::debug!("Folder path: {}", result.display());
//
//         assert_eq!(result, PathBuf::from("Album Artist/Album/"));
//     }
// }
