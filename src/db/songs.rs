use std::{
    collections::HashMap,
    path::{MAIN_SEPARATOR, Path, PathBuf},
};

use sqlx::{query, query_as, query_scalar};
use time::OffsetDateTime;

use super::{Album, DatabaseError, Directory, NewSong, Result, Song, UpdatedSong, directories};

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum DatabaseSongError {
    #[error("Album not found")]
    AlbumNotFound,
    #[error("Song not found")]
    SongNotFound,
    #[error("Song already exists")]
    SongAlreadyExists,
    #[error("Metadata error: {0}")]
    Metadata(#[from] crate::metadata::Error),
    #[error("Song path doesn't exist in any directories")]
    PathNotFound,
    #[error("Song path doesn't contain directory")]
    PathDoesntContainDirectory,
}

pub async fn add_song<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    song: NewSong,
) -> Result<Song> {
    let mut connection = connection.acquire().await?;
    let uuid = uuid::Uuid::new_v4().to_string();

    let NewSong {
        path,
        title,
        album,
        album_artist,
        disc_number,
        artist,
        year,
        track_number,
        genre,
        mood,
        file_created_at,
    } = song;

    let Directory {
        name: directory_id, ..
    } = directories::find_directory_from_sub_path(&mut *connection, &path)
        .await
        .map_err(|err| match err {
            DatabaseError::Directory(directories::DatabaseDirectoryError::NotFound) => {
                DatabaseSongError::PathNotFound.into()
            }
            _ => err,
        })?;

    let added_at = Some(OffsetDateTime::now_utc());
    let _ = query!(
        "INSERT INTO songs (id, path, title, album, album_artist, disc_number, artist, year, track_number, genre, mood, added_at, file_created_at, directory_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        uuid,
        path,
        title,
        album,
        album_artist,
        disc_number,
        artist,
        year,
        track_number,
        genre,
        mood,
        added_at,
        file_created_at,
        directory_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(Song {
        id: uuid,
        path,
        title,
        album,
        album_artist,
        disc_number,
        artist,
        year,
        track_number,
        genre,
        mood,
        added_at,
        file_created_at,
        directory_id,
        ..Default::default()
    })
}

pub async fn get_song<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    id: &str,
) -> Result<Song> {
    let mut connection = connection.acquire().await?;
    query_as!(Song, "SELECT * FROM songs WHERE id = ?", id)
        .fetch_one(&mut *connection)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DatabaseSongError::SongNotFound.into(),
            _ => err.into(),
        })
}

pub async fn get_song_path<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    id: &str,
) -> Result<PathBuf> {
    let mut connection = connection.acquire().await?;
    query_scalar!("SELECT path FROM songs WHERE id = ?", id)
        .fetch_one(&mut *connection)
        .await
        .map(PathBuf::from)
        .map_err(DatabaseError::from)
}

pub async fn get_songs(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Song>> {
    query_as!(Song, "SELECT * FROM songs")
        .fetch_all(pool)
        .await
        .map_err(DatabaseError::from)
}

pub async fn delete_song<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    id: &str,
) -> Result<()> {
    let mut connection = connection.acquire().await?;
    if query!("DELETE FROM songs WHERE id = ?", id)
        .execute(&mut *connection)
        .await?
        .rows_affected()
        == 0
    {
        Err(DatabaseSongError::SongNotFound.into())
    } else {
        Ok(())
    }
}

pub async fn update_song<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    id: &str,
    song: UpdatedSong,
) -> Result<()> {
    let mut connection = connection.acquire().await?;

    let _ = query!(
        "UPDATE songs SET title = ?, album = ?, album_artist = ?, disc_number = ?, artist = ?, year = ?, track_number = ?, genre = ?, mood = ? WHERE id = ?",
        song.title,
        song.album,
        song.album_artist,
        song.disc_number,
        song.artist,
        song.year,
        song.track_number,
        song.genre,
        song.mood,
        id
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn update_song_path<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    song_id: &str,
    new_directory_id: Option<&str>,
    new_path: &str,
) -> Result<()> {
    let mut connection = connection.acquire().await?;
    let song = get_song(&mut *connection, song_id).await?;

    if let Some(directory_id) = new_directory_id {
        let new_directory = directories::get_directory(&mut *connection, directory_id).await?;
        if !new_path.starts_with(new_directory.path.as_str()) {
            return Err(DatabaseSongError::PathDoesntContainDirectory.into());
        }

        let _ = query!(
            "UPDATE songs SET directory_id = ?, path = ? WHERE id = ?",
            directory_id,
            new_path,
            song_id
        )
        .execute(&mut *connection)
        .await?;
    } else {
        let prev_directory =
            directories::get_directory(&mut *connection, &song.directory_id).await?;

        if !new_path.starts_with(prev_directory.path.as_str()) {
            return Err(DatabaseSongError::PathDoesntContainDirectory.into());
        }

        let _ = query!("UPDATE songs SET path = ? WHERE id = ?", new_path, song_id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn get_album<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    title: String,
) -> Result<Album> {
    let mut connection = connection.acquire().await?;
    let tracks = query_as!(Song, "SELECT * FROM songs WHERE album = ?", title)
        .fetch_all(&mut *connection)
        .await?;

    if tracks.is_empty() {
        return Err(DatabaseSongError::AlbumNotFound.into());
    }

    let album = Album::from(tracks);

    Ok(album)
}

pub async fn get_albums<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
) -> Result<Vec<Album>> {
    let mut connection = connection.acquire().await?;
    let tracks = query_as!(Song, "SELECT * FROM songs WHERE album IS NOT NULL")
        .fetch_all(&mut *connection)
        .await?;

    let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();

    for track in tracks {
        album_map
            .entry(track.album.clone().expect("Album not found"))
            .or_default()
            .push(track);
    }

    Ok(album_map.into_values().map(Album::from).collect())
}
