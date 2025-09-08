//! Database models

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;

#[derive(Deserialize, Serialize, FromRow, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "NewDirectory", export)]
pub struct Directory {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, FromRow, Debug, Clone, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "DatabaseSong", export)]
pub struct Song {
    pub id: String,
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
