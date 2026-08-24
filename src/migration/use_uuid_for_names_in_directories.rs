use super::*;

pub const VERSION: i64 = 20250916122132;

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    let now = std::time::SystemTime::now();
    let directories: Vec<(String,)> = query_as("SELECT name FROM directories")
        .fetch_all(pool)
        .await?;

    if directories.is_empty() {
        info!("No directories found, skipping uuid addition");
        return Ok(());
    }

    info!(
        "Found {} directories... Generating uuids...",
        directories.len()
    );
    let uuid_directory_map: HashMap<_, _> = directories
        .iter()
        .map(|(id,)| (id, uuid::Uuid::new_v4().to_string()))
        .collect();

    info!(
        "Generated uuids in {}ms. Adding generated uuids to {} directories...",
        now.elapsed()?.as_millis(),
        directories.len()
    );

    let now = std::time::SystemTime::now();

    let total = uuid_directory_map.len();
    let mut tx = pool.begin().await?;

    for (directory_name, uuid) in uuid_directory_map.iter() {
        let result =
            sqlx::query("UPDATE directories SET display_name = ?, name = ? WHERE name = ?")
                .bind(directory_name)
                .bind(uuid)
                .bind(directory_name)
                .execute(&mut *tx)
                .await?
                .rows_affected();

        if result != 1 {
            return Err(eyre!(
                "Failed to add uuid to directory with id {directory_name}"
            ));
        }
    }

    info!(
        "Added uuids to {} directories in {}ms, committing transaction...",
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
mod tests {
    use tracing::instrument;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    #[instrument]
    async fn test_add_uuid_to_directories() {
        const ADD_DATE_COLUMN_TO_SONGS: i64 = 20250909151516;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_migrations(&pool, ADD_DATE_COLUMN_TO_SONGS)
            .await
            .unwrap();

        query_unchecked!(
            "INSERT INTO directories (name, path) VALUES ('directory', '/path/to/directory')"
        )
        .execute(&pool)
        .await
        .unwrap();

        run_migrations(&pool, false)
            .await
            .expect("Failed to run migrations");

        let uuid: String =
            sqlx::query_scalar("SELECT name FROM directories WHERE path = '/path/to/directory'")
                .fetch_one(&pool)
                .await
                .unwrap();

        log::info!("Directory uuid name: {uuid}");

        assert!(uuid.contains('-'));
    }
}
