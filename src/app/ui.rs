use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "dist/"]
struct Asset;

pub fn router() -> Router {
    Router::new().fallback(get(static_handler))
}

async fn index() -> Response {
    match Asset::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() || path == "index.html" {
        return index().await;
    }

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains(".") || path == "/api" {
                return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
            }

            index().await
        },
    }
}
