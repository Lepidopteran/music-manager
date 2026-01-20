use std::fs::{create_dir, File};

use tower_http::trace::TraceLayer;
use tracing::info_span;

use axum::{
    extract::{MatchedPath, Request},
    Router,
};

mod api;
mod db;
mod events;
mod state;
mod tasks;

pub mod migration;
pub use state::*;

use crate::{metadata, paths};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .merge(api::tasks::router())
        .merge(api::songs::router())
        .merge(api::albums::router())
        .merge(api::directories::router())
        .merge(api::cover_art::router())
        .merge(api::info::router())
        .nest("/api", events::router())
        .with_state(state)
        .merge(api::ui::router())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path = matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
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
pub fn create_default_database(name: &str) -> Result<String> {
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

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] db::DatabaseError),
    #[error("Metadata error: {0}")]
    Metadata(#[from] metadata::Error),
    #[error("Task registry error: {0}")]
    TaskRegistry(#[from] tasks::RegistryError),
}
