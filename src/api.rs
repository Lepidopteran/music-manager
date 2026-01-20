use axum::response::IntoResponse;
use super::{Error, utils::*, tasks as app_tasks};

pub mod songs;
pub mod directories;
pub mod tasks;
pub mod cover_art;
pub mod albums;
pub mod info;
pub mod ui;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Database(err) => err.into_response(),
            Error::Io(err) => internal_error(err).into_response(),
            Error::Metadata(err) => internal_error(err).into_response(),
            Error::TaskRegistry(err) => match err {
                app_tasks::RegistryError::NotFound => not_found(err).into_response(),
                app_tasks::RegistryError::StateError(err) => bad_request(err).into_response(),
                _ => internal_error(err).into_response(),
            },
        }
    }
}
