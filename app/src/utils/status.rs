use axum::http::StatusCode;

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    tracing::error!("internal error: {}", err);
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// Utility function for mapping any error into a `400 Bad Request` response.
pub fn bad_request<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    tracing::error!("bad request: {}", err);
    (StatusCode::BAD_REQUEST, err.to_string())
}

/// Utility function for mapping any error into a `404 Not Found` response.
pub fn not_found<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    tracing::error!("not found: {}", err);
    (StatusCode::NOT_FOUND, err.to_string())
}
