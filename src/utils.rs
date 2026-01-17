use std::fs::{create_dir, File};
use axum::http::StatusCode;
use std::fmt::Display;

use crate::{metadata::{item::ItemKey, Metadata}, paths};
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

/// A very lax function for getting a metadata field.
///
/// # Arguments
/// * `metadata` - The metadata to get the field from.
/// * `field` - The field to get.
///
/// # Returns
/// Returns the value of the field if it exists, otherwise `None`.
/// Additionally, if the metadata is `None`, this will return `None`.
pub fn get_metadata_field(metadata: &Option<Metadata>, field: ItemKey) -> Option<String> {
    metadata.as_ref().and_then(|m| m.get(&field).cloned())
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
pub fn internal_error(err: impl Display) -> (StatusCode, String) {
    tracing::error!("internal error: {}", err);
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// Utility function for mapping any error into a `400 Bad Request` response.
pub fn bad_request(err: impl Display) -> (StatusCode, String) {
    tracing::error!("bad request: {}", err);
    (StatusCode::BAD_REQUEST, err.to_string())
}

/// Utility function for mapping any error into a `404 Not Found` response.
pub fn not_found(err: impl Display) -> (StatusCode, String) {
    tracing::error!("not found: {}", err);
    (StatusCode::NOT_FOUND, err.to_string())
}
