use std::collections::{HashMap, HashSet};

use color_eyre::{
    eyre::{eyre, Result},
    owo_colors::OwoColorize,
};
use sqlx::{
    migrate::{AppliedMigration, Migrate, MigrateError, Migrator},
    query_as, SqlitePool,
};

use tracing::info;
static MIGRATOR: Migrator = sqlx::migrate!();
const ADD_UUID_VERSION: i64 = 20250905175005;

/// Runs the migrations
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Checking if any migrations need to be applied...");

    if apply_migrations(pool, Some(ADD_UUID_VERSION)).await? {
        info!("\"add uuid to songs\" migration has been applied, running after logic...");
        add_uuid_to_songs(pool).await?;
    };

    if apply_migrations(pool, None).await? {
        info!("Migrations complete");
    } else {
        info!("No migrations needed applied");
    }

    Ok(())
}

/// Applies migrations to the database
/// Returns true if any changes were made
async fn apply_migrations(pool: &SqlitePool, version: Option<i64>) -> Result<bool> {
    let mut connection = pool.acquire().await?;
    let mut made_changes = false;

    connection.lock().await?;

    connection.ensure_migrations_table().await?;

    let dirty_version = connection.dirty_version().await?;
    if let Some(version) = dirty_version {
        return Err(MigrateError::Dirty(version).into());
    }

    let applied_migrations = connection.list_applied_migrations().await?;

    validate_applied_migrations(&applied_migrations, &MIGRATOR)?;

    let applied_migration_map: HashMap<_, _> = applied_migrations
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for migration in MIGRATOR.iter() {
        if version.is_some_and(|version| migration.version > version) {
            break;
        }

        if migration.migration_type.is_down_migration() {
            continue;
        }

        if let Some(applied_migration) = applied_migration_map.get(&migration.version) {
            if migration.checksum != applied_migration.checksum {
                return Err(MigrateError::VersionMismatch(migration.version).into());
            }

            continue;
        }

        connection.apply(migration).await?;
        made_changes = true;

        info!(
            "Applied migration: \"{}\" (v{}) {}",
            migration.description,
            migration.version,
            if migration.migration_type.is_reversible() {
                "reversible".bright_blue().to_string()
            } else {
                "non-reversible".bright_yellow().to_string()
            }
        );
    }

    connection.unlock().await?;

    Ok(made_changes)
}

async fn add_uuid_to_songs(pool: &SqlitePool) -> Result<()> {
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

/// Validates that all applied migrations are present in the list of migrations.
///
/// Took from [migrator.rs](https://github.com/launchbadge/sqlx/blob/69bb5952ab665f6edfb461b45e63cc3b6d99a4d0/sqlx-core/src/migrate/migrator.rs#L298) file in sqlx repo
fn validate_applied_migrations(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
) -> Result<(), MigrateError> {
    if migrator.ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    for applied_migration in applied_migrations {
        if !migrations.contains(&applied_migration.version) {
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use sqlx::query_unchecked;

    use super::*;

    const ADD_MOOD_VERSION: i64 = 20250725224500;

    #[tokio::test]
    async fn test_add_uuid_to_songs() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_migrations(&pool, Some(ADD_MOOD_VERSION))
            .await
            .unwrap();

        query_unchecked!("INSERT INTO songs (path) VALUES ('/path/to/song.mp3')")
            .execute(&pool)
            .await
            .unwrap();

        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        let uuid: String =
            sqlx::query_scalar("SELECT id FROM songs WHERE path = '/path/to/song.mp3'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(uuid.contains('-'));
    }
}
