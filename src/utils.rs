use axum::http::StatusCode;
use std::fmt::Display;

use crate::metadata::{item::ItemKey, Metadata};

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
