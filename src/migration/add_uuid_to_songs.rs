use super::*;

pub const VERSION: i64 = 20250905175005;

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    let now = std::time::SystemTime::now();
    let songs: Vec<(i64,)> = query_as("SELECT id FROM songs").fetch_all(pool).await?;

    if songs.is_empty() {
        info!("No songs found, skipping uuid addition");
        return Ok(());
    }

    info!("Found {} songs... Generating uuids...", songs.len());
    let uuid_song_map: HashMap<_, _> = songs
        .iter()
        .map(|(id,)| (id, uuid::Uuid::new_v4().to_string()))
        .collect();

    info!(
        "Generated uuids in {}ms. Adding generated uuids to {} songs...",
        now.elapsed()?.as_millis(),
        songs.len()
    );

    let now = std::time::SystemTime::now();

    let total = uuid_song_map.len();
    let mut tx = pool.begin().await?;

    for (song_id, uuid) in uuid_song_map.iter() {
        let result = sqlx::query("UPDATE songs SET uuid = ? WHERE id = ?")
            .bind(uuid)
            .bind(song_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        if result != 1 {
            return Err(eyre!("Failed to add uuid to song with id {song_id}"));
        }
    }

    info!(
        "Added uuids to {} songs in {}ms, committing transaction...",
        total,
        now.elapsed()?.as_millis()
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
mod test {
    use super::*;
    const ADD_MOOD_TO_SONGS: i64 = 20250725224500;

    #[tokio::test]
    #[test_log::test]
    async fn test_add_uuid_to_songs() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_migrations(&pool, ADD_MOOD_TO_SONGS).await.unwrap();

        query_unchecked!("INSERT INTO directories (name, path) VALUES ('directory', '/path/to/')")
            .execute(&pool)
            .await
            .unwrap();

        query_unchecked!("INSERT INTO songs (path) VALUES ('/path/to/song.mp3')")
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool, false)
            .await
            .expect("Failed to run migrations");

        let uuid: String =
            sqlx::query_scalar("select id from songs where path = '/path/to/song.mp3'")
                .fetch_one(&pool)
                .await
                .unwrap();

        log::info!("Song uuid id: {uuid}");

        assert!(uuid.contains('-'));
    }
}
