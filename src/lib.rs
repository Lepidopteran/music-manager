use std::{
    fs::{File, create_dir},
    path::PathBuf,
    sync::LazyLock,
};

use clap::Parser;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use axum::{
    Router,
    extract::{MatchedPath, Request},
};

mod metadata;

mod api;
mod config;
mod db;
mod events;
mod migration;
mod organize;
mod paths;
mod state;
mod tasks;
mod fs;

pub use config::load_config;
pub use migration::run_migrations;
pub use state::AppState;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static APP_DIRECTORIES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    vec![
        paths::app_config_dir(),
        paths::app_cache_dir(),
        paths::app_data_dir(),
        paths::metadata_history_dir(),
    ]
});

pub fn initialize_logging() {
    color_eyre::install().expect("Failed to install color_eyre");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "Music Manager", version, author)]
pub struct Args {
    /// Database URL to connect to
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    /// IP address to bind to
    #[arg(long, action = clap::ArgAction::SetTrue, env = "LISTEN_ON_ALL_INTERFACES")]
    pub host: Option<bool>,

    /// Port to bind to
    #[arg(long, env = "PORT")]
    pub port: Option<u16>,

    /// Path to config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .merge(api::tasks::router())
        .merge(api::songs::router())
        .merge(api::albums::router())
        .merge(api::directories::router())
        .merge(api::cover_art::router())
        .merge(api::info::router())
        .nest(
            "/api",
            Router::new()
                .merge(events::router())
                .merge(api::organize::router()),
        )
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
    #[error("Organization error: {0}")]
    Organization(#[from] organize::OrganizeError),
}
