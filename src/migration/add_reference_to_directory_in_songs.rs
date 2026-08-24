use super::*;

pub const VERSION: i64 = 20260115231518;

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    let song_paths: Vec<(String, String)> = query_as("SELECT id, path FROM songs")
        .fetch_all(pool)
        .await?;

    if song_paths.is_empty() {
        info!("No songs found, skipping directory reference addition");
        return Ok(());
    }

    let directories: Vec<(String, String)> = query_as("SELECT name, path FROM directories")
        .fetch_all(pool)
        .await?;

    if directories.is_empty() {
        tracing::error!("No directories found, this should not be possible");
        return Err(eyre!("No directories found, this should not be possible"));
    }

    info!(
        "Found {} directories and {} songs... Adding directory references to songs...",
        directories.len(),
        song_paths.len()
    );

    let now = std::time::SystemTime::now();

    let song_directory_map: HashMap<_, _> = song_paths
        .iter()
        .map(|(id, path)| {
            directories
                .iter()
                .find(|(_, directory_path)| path.starts_with(directory_path))
                .map(|(dir_id, _)| (id, dir_id))
                .expect("No directory found for song, this is a bug!")
        })
        .collect();

    let mut tx = pool.begin().await?;

    for (song_id, directory_id) in song_directory_map.iter() {
        let result = sqlx::query("UPDATE songs SET directory_id = ? WHERE id = ?")
            .bind(directory_id)
            .bind(song_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        if result != 1 {
            return Err(eyre!(
                "Failed to add directory reference to song with id {song_id}"
            ));
        }
    }

    info!(
        "Added directory references to {} songs in {}ms, committing transaction...",
        song_directory_map.len(),
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
mod tests {
    use sqlx::query;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn test_add_reference_to_directory_in_songs() {
        const ADD_DISPLAY_NAME_COLUMN_DIRECTORIES: i64 = 20250916122132;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_migrations(&pool, ADD_DISPLAY_NAME_COLUMN_DIRECTORIES)
            .await
            .unwrap();

        query_unchecked!("INSERT INTO directories (name, path) VALUES ('directory', '/path/to/')")
            .execute(&pool)
            .await
            .unwrap();

        query("INSERT INTO songs (id, path) VALUES (?, '/path/to/song.mp3')")
            .bind(uuid::Uuid::new_v4().to_string())
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool, false)
            .await
            .expect("Failed to run migrations");
    }
}
