use color_eyre::owo_colors::OwoColorize;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::get_app_config_dir;

use super::{
    config,
    task::{Registry, RegistryError, TaskInfo},
};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{MatchedPath, Request},
    Router,
};

mod api;
mod tasks;
mod ui;

pub async fn serve(settings: config::Settings, db: sqlx::Pool<sqlx::Sqlite>) {
    let app = Router::new()
        .merge(ui::router())
        .merge(api::songs::router().with_state(db.clone()))
        .merge(api::albums::router().with_state(db.clone()))
        .merge(api::directories::router().with_state(db.clone()))
        .merge(api::tasks::router().with_state(setup_tasks(db.clone())))
        .merge(api::cover_art::router().with_state(db))
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
        );

    let host = settings.server.host.unwrap_or_else(|| {
        if settings.server.listen_on_all_interfaces {
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
        } else {
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        }
    });

    let addr = SocketAddr::from((host, settings.server.port));
    tracing::info!(
        "Listening on {}{}",
        "http://".underline().blue(),
        addr.underline().blue()
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn setup_tasks(pool: sqlx::Pool<sqlx::Sqlite>) -> Arc<Mutex<Registry>> {
    let mut registry = Registry::default();

    let _ = registry.register(move || Box::new(tasks::ScanSongs::new(pool.clone())));

    Arc::new(Mutex::new(registry))
}

/// Ensure that the app directories exist.
pub fn ensure_paths_exist() -> Result<(), std::io::Error> {
    let dirs = vec![
        get_app_config_dir()
    ];

    for dir in dirs {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
    }

    Ok(())
}
