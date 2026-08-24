use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use color_eyre::{
    eyre::{Result, eyre},
    owo_colors::OwoColorize,
};
use sqlx::{
    SqlitePool,
    migrate::{AppliedMigration, Migrate, MigrateError, Migrator},
    query_as, query_unchecked,
};

use futures::future::BoxFuture;

use tracing::info;
static MIGRATOR: Migrator = sqlx::migrate!();

type MigrationFn = fn(&SqlitePool) -> BoxFuture<'_, Result<()>>;

mod add_reference_to_directory_in_songs;
mod add_uuid_to_songs;
mod use_uuid_for_names_in_directories;

macro_rules! migration_map {
    ( $( $migration_module:ident), * ) => {
        LazyLock::new(|| {
            let mut map: HashMap<i64, MigrationFn> = HashMap::new();
            $(
                map.insert($migration_module::VERSION, |pool| Box::pin($migration_module::migrate(pool)));
            )*

            map
        })
    };
}

static MIGRATIONS: LazyLock<HashMap<i64, MigrationFn>> = migration_map! {
    add_uuid_to_songs,
    add_reference_to_directory_in_songs,
    use_uuid_for_names_in_directories
};

const MIGRATION_TABLE_NAME: &str = "_sqlx_migrations";

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

    connection
        .ensure_migrations_table(MIGRATION_TABLE_NAME)
        .await?;

    let dirty_version = connection.dirty_version(MIGRATION_TABLE_NAME).await?;
    if let Some(version) = dirty_version {
        return Err(MigrateError::Dirty(version).into());
    }

    let applied_migrations = connection
        .list_applied_migrations(MIGRATION_TABLE_NAME)
        .await?;

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

        connection.apply(MIGRATION_TABLE_NAME, migration).await?;

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

            if let Some(migration_fn) = MIGRATIONS.get(&migration.version) {
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
/// Applies migrations to the database
/// Returns true if any changes were made
async fn apply_migrations(pool: &SqlitePool, version: i64) -> Result<bool> {
    let mut connection = pool.acquire().await?;
    let mut made_changes = false;

    connection.lock().await?;

    connection
        .ensure_migrations_table(MIGRATION_TABLE_NAME)
        .await?;

    let dirty_version = connection.dirty_version(MIGRATION_TABLE_NAME).await?;
    if let Some(version) = dirty_version {
        return Err(MigrateError::Dirty(version).into());
    }

    let applied_migrations = connection
        .list_applied_migrations(MIGRATION_TABLE_NAME)
        .await?;

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

        connection.apply(MIGRATION_TABLE_NAME, migration).await?;
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
