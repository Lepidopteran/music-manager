use tower_http::trace::TraceLayer;
use tracing::info_span;

use axum::{
    extract::{MatchedPath, Request},
    Router,
};

use super::{api, events};

pub mod migration;
mod state;

pub use state::*;

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
