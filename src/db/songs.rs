use axum::response::IntoResponse;
use hyper::StatusCode;
use sqlx::query_as;

use super::{Album, DatabaseError, Song};
use std::collections::HashMap;

type Result<T, E = DatabaseError> = std::result::Result<T, E>;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum DatabaseSongError {
    #[error("Album not found")]
    AlbumNotFound,
    #[error("No albums found")]
    NoAlbums,
}

impl IntoResponse for DatabaseSongError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::AlbumNotFound | Self::NoAlbums => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
        }
    }
}

pub async fn get_album(pool: &sqlx::Pool<sqlx::Sqlite>, title: String) -> Result<Album> {
    let tracks = query_as!(Song, "SELECT * FROM songs WHERE album = ?", title)
        .fetch_all(pool)
        .await?;

    if tracks.is_empty() {
        return Err(DatabaseSongError::AlbumNotFound.into());
    }

    let album = Album::from(tracks);

    Ok(album)
}

pub async fn get_albums(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Album>> {
    let tracks = query_as!(Song, "SELECT * FROM songs WHERE album IS NOT NULL")
        .fetch_all(pool)
        .await?;

    if tracks.is_empty() {
        return Err(DatabaseSongError::NoAlbums.into());
    }

    let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();

    for track in tracks {
        album_map
            .entry(track.album.clone().expect("Album not found"))
            .or_default()
            .push(track);
    }

    Ok(album_map.into_values().map(Album::from).collect())
}
