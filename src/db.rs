//! Database models

use std::fs::{File, create_dir};

use super::paths;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::time::OffsetDateTime;
use ts_rs::TS;

#[derive(Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub name: String,
    pub path: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize, Serialize, FromRow, Debug, Clone, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "DatabaseSong", export)]
pub struct Song {
    pub id: String,
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub year: Option<String>,
    pub mood: Option<String>,
    #[ts(type = "Date")]
    pub added_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    pub updated_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    pub file_created_at: Option<OffsetDateTime>,
    pub directory_id: String,
}

/// Utility function for creating a default database.
///
/// # Arguments
/// * `name` - The name of the database.
///
/// # Errors
/// Returns an `io::Error` if the database could not be created.
///
/// # Returns
/// Returns the connection string of the database.
pub fn create_default_database(name: &str) -> Result<String, std::io::Error> {
    let db_name = format!("{name}.db");
    let config_dir = paths::app_config_dir();
    let conn_str = format!("sqlite://{}", config_dir.join(db_name.clone()).display());

    if !config_dir.exists() {
        create_dir(&config_dir).map_err(|err| {
            tracing::error!("Failed to create config directory: {}", err);
            err
        })?;
    }

    let db_path = paths::app_data_dir().join(db_name);

    if !db_path.exists() {
        File::create(&db_path).map_err(|err| {
            tracing::error!("Failed to create database file: {}", err);
            err
        })?;
    }

    Ok(conn_str)
}
