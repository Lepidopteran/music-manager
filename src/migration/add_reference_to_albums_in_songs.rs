use std::collections::BTreeMap;

use super::*;
use sqlx::prelude::*;
use time::OffsetDateTime;

pub const VERSION: i64 = 20260822000717;

#[derive(FromRow, Debug, Clone, Default)]
struct Song {
    pub id: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    let songs: Vec<Song> = query_as("SELECT id, album, album_artist FROM songs")
        .fetch_all(pool)
        .await?;

    if songs.is_empty() {
        info!("No songs found, skipping album addition");
        return Ok(());
    }

    let song_position_map =
        songs
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut map, (position, song)| {
                map.entry(song.id.clone()).or_insert(position);
                map
            });

    let album_map = songs.iter().fold(BTreeMap::new(), |mut map, song| {
        if let Some(album) = &song.album {
            map.entry(album.clone())
                .or_insert_with(Vec::new)
                .push(song.id.clone());
        }

        map
    });

    if album_map.is_empty() {
        info!("No albums found, skipping album addition");
        return Ok(());
    }

    info!("Found {} albums... Adding albums...", album_map.len());

    let mut tx = pool.begin().await?;

    let now = std::time::SystemTime::now();

    for (album, song_ids) in album_map.iter() {
        let id = uuid::Uuid::new_v4().to_string();

        let artist: Option<String> = song_position_map
            .get(&song_ids[0])
            .and_then(|pos| songs.get(*pos).and_then(|s| s.album_artist.clone()));

        let result =
            sqlx::query("INSERT INTO albums (id, title, artist, added_at) VALUES (?, ?, ?, ?)")
                .bind(&id)
                .bind(album)
                .bind(artist)
                .bind(OffsetDateTime::now_utc())
                .execute(&mut *tx)
                .await?
                .rows_affected();

        if result != 1 {
            return Err(eyre!("Failed to add album {album}"));
        }

        for song_id in song_ids {
            let result = sqlx::query("UPDATE songs SET album_id = ? WHERE id = ?")
                .bind(&id)
                .bind(song_id)
                .execute(&mut *tx)
                .await?
                .rows_affected();

            if result != 1 {
                return Err(eyre!("Failed to add album reference to song"));
            }
        }
    }

    info!(
        "Added albums and references in {}ms. Committing transaction...",
        now.elapsed()?.as_millis(),
    );

    let now = std::time::SystemTime::now();
    tx.commit().await?;

    info!(
        "Successfully committed transaction in {}ms",
        now.elapsed()?.as_millis()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::query;

    #[tokio::test]
    #[test_log::test]
    async fn test_add_reference_to_albums_in_songs() {
        const ENFORCE_DIRECTORY_REFERENCES_IN_SONGS: i64 = 20260115233850;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_migrations(&pool, ENFORCE_DIRECTORY_REFERENCES_IN_SONGS)
            .await
            .unwrap();

        query_unchecked!("INSERT INTO directories (name, path) VALUES ('directory', '/path/to/')")
            .execute(&pool)
            .await
            .unwrap();

        query("INSERT INTO songs (id, album, album_artist, path, directory_id) VALUES ('real-song', 'album', 'artist', '/path/to/song.mp3', 'directory')")
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool, false)
            .await
            .expect("Failed to run migrations");

        let song = query("SELECT * FROM songs WHERE id = 'real-song'")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(song.get::<String, _>("album_id").contains("-"));
    }
}
