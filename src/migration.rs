use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use color_eyre::{
    eyre::{eyre, Result},
    owo_colors::OwoColorize,
};
use sqlx::{
    migrate::{AppliedMigration, Migrate, MigrateError, Migrator},
    query_as, SqlitePool,
};

use futures::future::BoxFuture;

use tracing::info;
static MIGRATOR: Migrator = sqlx::migrate!();

type MigrationFn = fn(&SqlitePool) -> BoxFuture<'_, Result<()>>;

macro_rules! custom_migrations {
    ( $( $version:literal , $function:ident);* $(;)? ) => {
        LazyLock::new(|| {
            let mut map: HashMap<i64, MigrationFn> = HashMap::new();
            $(
                map.insert($version, |pool| Box::pin($function(pool)));
            )*

            map
        })
    };
}

static CUSTOM_MIGRATIONS: LazyLock<HashMap<i64, MigrationFn>> = custom_migrations! {
    20250905175005, add_uuid_to_songs;
    20250916122132, use_uuid_for_names_in_directories;
    20260115231518, add_reference_to_directory_in_songs;
};

/// Runs the migrations
///
/// If the database is new nothing will be printed, otherwise every migration that is applied will
/// be printed to console.
pub async fn run_migrations(pool: &SqlitePool, new_database: bool) -> Result<()> {
    let mut connection = pool.acquire().await?;
    let mut made_changes = false;
    connection.lock().await?;

    if !new_database {
        info!("Checking if migrations are needed...");
    } else {
        info!("New database detected, Initializing database...");
    }

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

        if !new_database {
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

            if let Some(migration_fn) = CUSTOM_MIGRATIONS.get(&migration.version) {
                info!(
                    "\"{}\" (v{}) has after-logic, running it now...",
                    migration.description, migration.version
                );

                migration_fn(pool).await?
            }

            made_changes = true;
        }
    }

    if !new_database {
        if made_changes {
            info!("Finished applying migrations");
        } else {
            info!("No migrations needed");
        }
    } else {
        info!("Finished initializing database");
    }

    connection.unlock().await?;
    Ok(())
}

async fn add_reference_to_directory_in_songs(pool: &SqlitePool) -> Result<()> {
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

async fn use_uuid_for_names_in_directories(pool: &SqlitePool) -> Result<()> {
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
    use sqlx::{query, query_unchecked};
    use tracing::instrument;

    use super::*;

    /// Applies migrations to the database
    /// Returns true if any changes were made
    async fn apply_migrations(pool: &SqlitePool, version: i64) -> Result<bool> {
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
            if migration.version > version {
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

    const ADD_DATE_COLUMN_TO_SONGS: i64 = 20250909151516;

    #[tokio::test]
    #[test_log::test]
    #[instrument]
    async fn test_add_uuid_to_directories() {
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

    const ADD_DISPLAY_NAME_COLUMN_DIRECTORIES: i64 = 20250916122132;

    #[tokio::test]
    #[test_log::test]
    async fn test_add_reference_to_directory_in_songs() {
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
