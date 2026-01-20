use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::time::OffsetDateTime;
use ts_rs::TS;

use crate::metadata::{SongFile, item::ItemKey};

pub mod directories;
pub mod songs;

type Result<T, E = DatabaseError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    Song(#[from] songs::DatabaseSongError),
    #[error(transparent)]
    Directory(#[from] directories::DatabaseDirectoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub name: String,
    pub path: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewDirectory {
    /// The path of the directory.
    pub path: String,
    /// The display name of the directory, only used in the UI.
    pub display_name: Option<String>,
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
    #[ts(type = "Date")]
    pub added_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    pub updated_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    pub file_created_at: Option<OffsetDateTime>,
    pub directory_id: String,
}

#[derive(Deserialize, Debug, Clone, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "NewDatabaseSong", export)]
pub struct NewSong {
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
    #[ts(type = "Date")]
    pub file_created_at: Option<OffsetDateTime>,
}

impl From<SongFile> for NewSong {
    fn from(file: SongFile) -> Self {
        let metadata = file.metadata().as_ref();
        NewSong {
            path: file.path().to_string_lossy().to_string(),
            title: metadata.and_then(|m| m.get(&ItemKey::Title).cloned()),
            artist: metadata.and_then(|m| m.get(&ItemKey::Artist).cloned()),
            album: metadata.and_then(|m| m.get(&ItemKey::Album).cloned()),
            album_artist: metadata.and_then(|m| m.get(&ItemKey::AlbumArtist).cloned()),
            genre: metadata.and_then(|m| m.get(&ItemKey::Genre).cloned()),
            track_number: metadata.and_then(|m| m.get(&ItemKey::TrackNumber).cloned()),
            disc_number: metadata.and_then(|m| m.get(&ItemKey::DiscNumber).cloned()),
            year: metadata.and_then(|m| m.get(&ItemKey::Year).cloned()),
            mood: metadata.and_then(|m| m.get(&ItemKey::Mood).cloned()),
            file_created_at: Some(file.created()),
        }
    }
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdatedSong {
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

impl From<SongFile> for UpdatedSong {
    fn from(file: SongFile) -> Self {
        let metadata = file.metadata().as_ref();
        UpdatedSong {
            title: metadata.and_then(|m| m.get(&ItemKey::Title).cloned()),
            artist: metadata.and_then(|m| m.get(&ItemKey::Artist).cloned()),
            album: metadata.and_then(|m| m.get(&ItemKey::Album).cloned()),
            album_artist: metadata.and_then(|m| m.get(&ItemKey::AlbumArtist).cloned()),
            genre: metadata.and_then(|m| m.get(&ItemKey::Genre).cloned()),
            track_number: metadata.and_then(|m| m.get(&ItemKey::TrackNumber).cloned()),
            disc_number: metadata.and_then(|m| m.get(&ItemKey::DiscNumber).cloned()),
            year: metadata.and_then(|m| m.get(&ItemKey::Year).cloned()),
            mood: metadata.and_then(|m| m.get(&ItemKey::Mood).cloned()),
        }
    }
}

/// A collection of songs. Does not correlate to a table in the database.
#[derive(serde::Serialize, TS)]
#[ts(rename = "Album", export)]
pub struct Album {
    pub title: String,
    pub artist: Option<String>,
    pub tracks: Vec<Song>,
}

impl From<Vec<Song>> for Album {
    fn from(tracks: Vec<Song>) -> Self {
        let title = tracks[0].album.clone().expect("Album not found");
        let artist = tracks[0].album_artist.clone();
        Album {
            title,
            artist,
            tracks,
        }
    }
}
