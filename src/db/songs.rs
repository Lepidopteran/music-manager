use std::{
    collections::HashMap,
    path::{MAIN_SEPARATOR, Path, PathBuf},
};

use sqlx::{query, query_as, query_scalar};
use time::OffsetDateTime;

use super::{
    Album, Connection, DatabaseError, Directory, NewSong, Result, Song, UpdatedSong, directories,
};

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

pub async fn add_song(connection: &mut Connection, song: NewSong) -> Result<Song> {
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

pub async fn get_song(connection: &mut Connection, id: &str) -> Result<Song> {
    query_as!(Song, "SELECT * FROM songs WHERE id = ?", id)
        .fetch_one(&mut *connection)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DatabaseSongError::SongNotFound.into(),
            _ => err.into(),
        })
}

pub async fn get_song_path(connection: &mut Connection, id: &str) -> Result<PathBuf> {
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

pub async fn delete_song(connection: &mut Connection, id: &str) -> Result<()> {
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

pub async fn update_song(connection: &mut Connection, id: &str, song: UpdatedSong) -> Result<()> {
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

pub async fn update_song_path(
    connection: &mut Connection,
    song_id: &str,
    new_path: &str,
) -> Result<()> {
    let previous_directory_id =
        query_scalar!("SELECT directory_id FROM songs WHERE id = ?", song_id)
            .fetch_one(&mut *connection)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => DatabaseError::from(DatabaseSongError::SongNotFound),
                _ => err.into(),
            })?;

    let directories = sqlx::query_as::<_, (String, String)>("SELECT path, name FROM directories")
        .fetch_all(&mut *connection)
        .await?;

    let new_directory_id = directories
        .iter()
        .find_map(|(path, id)| new_path.starts_with(path.as_str()).then_some(id))
        .ok_or(DatabaseSongError::PathDoesntContainDirectory)?;

    if new_directory_id != &previous_directory_id {
        let _ = query!(
            "UPDATE songs SET directory_id = ?, path = ? WHERE id = ?",
            new_directory_id,
            new_path,
            song_id
        )
        .execute(&mut *connection)
        .await?;
    } else {
        let _ = query!("UPDATE songs SET path = ? WHERE id = ?", new_path, song_id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn get_album(connection: &mut Connection, title: String) -> Result<Album> {
    let tracks = query_as!(Song, "SELECT * FROM songs WHERE album = ?", title)
        .fetch_all(&mut *connection)
        .await?;

    if tracks.is_empty() {
        return Err(DatabaseSongError::AlbumNotFound.into());
    }

    let album = Album::from(tracks);

    Ok(album)
}

pub async fn get_albums(connection: &mut Connection) -> Result<Vec<Album>> {
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
