pub use args::*;
pub use utils::*;

use axum::response::IntoResponse;

pub mod app;
pub mod config;
pub mod db;
pub mod logging;
pub mod metadata;
pub mod migration;
pub mod paths;
pub mod utils;

mod api;
mod args;
mod events;
mod tasks;
mod ui;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

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
