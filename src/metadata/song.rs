use std::path::Path;

use lofty::prelude::*;
use lofty::probe::Probe;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use super::tags::sanitize_tag;

#[derive(Deserialize, Serialize, FromRow, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub year: Option<String>,
}

impl Song {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Path does not exist".to_string());
        }

        if !path.is_file() {
            return Err("Path is not a file".to_string());
        }

        let stem = match path.file_stem() {
            Some(stem) => stem.to_str().map(|stem| stem.to_string()),
            None => return Err("No file stem found".to_string()),
        };

        let tagged_file = match Probe::open(path) {
            Ok(tag) => tag.read().map_err(|err| err.to_string())?,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or("No tags found".to_string())?,
        };

        Ok(Self {
            title: match tag.title().as_deref() {
                Some(title) => Some(sanitize_tag(title)),
                None => stem,
            },
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
            ..Default::default()
        })
    }
}
