use color_eyre::owo_colors::OwoColorize;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info_span;

use crate::{
    config::Settings,
    paths::{app_cache_dir, app_config_dir, app_data_dir, metadata_history_dir},
};

use super::{
    config,
    task::{Registry, RegistryError, TaskInfo},
};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{FromRef, MatchedPath, Request},
    Router,
};

mod api;
mod tasks;
mod ui;

pub type Database = sqlx::Pool<sqlx::Sqlite>;
pub type Tasks = Arc<Mutex<Registry>>;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub db: Database,
}

/// Start the server
pub async fn serve(settings: config::Settings, db: sqlx::Pool<sqlx::Sqlite>) {
    let tasks = setup_tasks(db.clone());
    let app = Router::new()
        .merge(api::tasks::router().with_state(tasks))
        .merge(api::songs::router())
        .merge(api::albums::router())
        .merge(api::directories::router())
        .merge(api::cover_art::router())
        .merge(api::info::router())
        .with_state(AppState {
            settings: settings.clone(),
            db,
        })
        .merge(ui::router())
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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Wait for a shutdown signal
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

/// Set up the tasks that can run in the background
fn setup_tasks(pool: sqlx::Pool<sqlx::Sqlite>) -> Arc<Mutex<Registry>> {
    let mut registry = Registry::default();

    let scan_songs_pool = pool.clone();
    let refresh_songs_pool = pool.clone();

    let err = registry.register(move || Box::new(tasks::ScanSongs::new(scan_songs_pool.clone())));
    if let Err(RegistryError::AlreadyExists) = err {
        tracing::warn!("Task already registered");
    }

    let err =
        registry.register(move || Box::new(tasks::UpdateSongs::new(refresh_songs_pool.clone())));
    if let Err(RegistryError::AlreadyExists) = err {
        tracing::warn!("Task already registered");
    }

    Arc::new(Mutex::new(registry))
}

/// Ensure that the app directories exist.
pub fn ensure_paths_exist() -> Result<(), std::io::Error> {
    let dirs = vec![
        app_config_dir(),
        app_cache_dir(),
        app_data_dir(),
        metadata_history_dir(),
    ];

    for dir in dirs {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
    }

    Ok(())
}

impl FromRef<AppState> for sqlx::Pool<sqlx::Sqlite> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Settings {
    fn from_ref(state: &AppState) -> Self {
        state.settings.clone()
    }
}
