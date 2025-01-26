use std::path::Path;

use lofty::prelude::*;
use lofty::probe::Probe;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use super::tags::sanitize_tag;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub year: Option<String>,
}

impl SongMetadata {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        check_path(path)?;

        let tagged_file = match Probe::open(path) {
            Ok(tag) => tag.read().map_err(|err| err.to_string())?,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or("No tags found".to_string())?,
        };

        Ok(Self::from(tag))
    }
}

impl From<&Tag> for SongMetadata {
    fn from(tag: &Tag) -> Self {
        Self {
            title: tag.title().as_deref().map(sanitize_tag),
            artist: tag.artist().as_deref().map(sanitize_tag),
            album: tag.album().as_deref().map(sanitize_tag),
            album_artist: tag.get_string(&ItemKey::AlbumArtist).map(sanitize_tag),
            genre: tag
                .get_strings(&ItemKey::Genre)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
            track_number: tag.track().map(|track| track.to_string()),
            disc_number: tag.disk().map(|disc| disc.to_string()),
            year: tag.year().map(|year| year.to_string()),
        }
    }
}

#[doc(hidden)]
pub fn check_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    Ok(())
}
