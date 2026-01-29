use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse};

use super::{
    Error,
    db::{DatabaseError, songs::DatabaseSongError},
    organize::OrganizeError,
    state::FileOperationManagerError,
    tasks::RegistryError,
};

pub mod albums;
pub mod cover_art;
pub mod directories;
pub mod fs;
pub mod info;
pub mod organize;
pub mod songs;
pub mod tasks;
pub mod ui;

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

/// Utility function for mapping any error into a `409 Conflict` response.
pub fn conflict(err: impl Display) -> (StatusCode, String) {
    tracing::error!("conflict: {}", err);
    (StatusCode::CONFLICT, err.to_string())
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(err) => err.into_response(),
            Self::Io(err) => internal_error(err).into_response(),
            Self::Metadata(err) => internal_error(err).into_response(),
            Self::Organization(err) => err.into_response(),
            Self::TaskRegistry(err) => err.into_response(),
            Self::FileOperationManager(err) => err.into_response(),
        }
    }
}

impl IntoResponse for OrganizeError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Handlebars(err) => match err.reason() {
                handlebars::RenderErrorReason::TemplateError(err) => {
                    bad_request(err).into_response()
                }
                _ => internal_error(err).into_response(),
            },
            Self::NoFileName(err) => bad_request(err.display()).into_response(),
        }
    }
}

impl IntoResponse for RegistryError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RegistryError::NotFound => not_found(self).into_response(),
            RegistryError::StateError(err) => bad_request(err).into_response(),
            _ => internal_error(self).into_response(),
        }
    }
}

impl IntoResponse for DatabaseSongError {
    fn into_response(self) -> axum::response::Response {
        match self {
            DatabaseSongError::SongAlreadyExists => conflict(self).into_response(),
            DatabaseSongError::Metadata(err) => internal_error(err).into_response(),
            DatabaseSongError::PathNotFound | Self::PathDoesntContainDirectory => {
                bad_request(self).into_response()
            }
            DatabaseSongError::AlbumNotFound | Self::SongNotFound => {
                not_found(self).into_response()
            }
        }
    }
}

impl IntoResponse for DatabaseError {
    fn into_response(self) -> axum::response::Response {
        match self {
            DatabaseError::Song(err) => err.into_response(),
            DatabaseError::Directory(err) => err.into_response(),
            DatabaseError::Sqlx(err) => internal_error(err).into_response(),
        }
    }
}

impl IntoResponse for FileOperationManagerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NotFound => not_found(self).into_response(),
            Self::FailedToAddOperation(err) => internal_error(err).into_response(),
        }
    }
}
