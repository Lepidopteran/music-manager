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
pub mod task;
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
    TaskRegistry(#[from] task::RegistryError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Database(err) => err.into_response(),
            Error::Io(err) => internal_error(err).into_response(),
            Error::Metadata(err) => internal_error(err).into_response(),
            Error::TaskRegistry(err) => match err {
                task::RegistryError::NotFound => not_found(err).into_response(),
                task::RegistryError::StateError(err) => bad_request(err).into_response(),
                _ => internal_error(err).into_response(),
            },
        }
    }
}
