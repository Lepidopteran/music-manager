//! Database models

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::metadata::SongMetadata;

#[derive(Deserialize, Serialize, FromRow)]
pub struct Directory {
    pub name: String,
    pub path: String,
}

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
    pub mood: Option<String>,
}
