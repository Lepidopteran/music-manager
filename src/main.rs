use std::{
    env,
    fs::File,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::signal;

use muusik::{app, config, db, logging, migration::run_migrations, paths, Args};

#[tokio::main]
async fn main() {
    logging::init().expect("Failed to initialize logging");
    dotenv().ok();

    ensure_paths_exist().expect("Failed to ensure paths exist");

    tracing::info!(
        "Launching {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let args = Args::parse();
    let settings = config::load(&args).expect("Failed to load settings");

    let database_url = match &settings.server.database_url {
        Some(url) if !url.trim().is_empty() => url,
        _ => {
            let info_msg = "No database URL specified. Using default database."
                .blue()
                .bold()
                .to_string();

            tracing::info!("{info_msg}");

            &db::create_default_database("data").expect("Failed to create default database")
        }
    };

    tracing::info!("Database URL: {}", database_url.underline().blue());

    let mut new_database = false;
    if let Some(database_url) = database_url.strip_prefix("sqlite://") {
        let path = PathBuf::from(database_url);

        if !path.exists() {
            File::create(&path).expect("Failed to create database file");
            new_database = true;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect(database_url)
        .await
        .expect("Failed to connect to database");

    run_migrations(&pool, new_database)
        .await
        .expect("Failed to run migrations");

    let host = settings.server.host.unwrap_or_else(|| {
        if settings.server.listen_on_all_interfaces {
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
        } else {
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        }
    });

    let addr = SocketAddr::from((host, settings.server.port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    let state = app::AppState::new(pool, settings);

    tracing::info!(
        "Listening on {}{}",
        "http://".underline().blue(),
        addr.underline().blue()
    );

    axum::serve(listener, app::routes(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start server");
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

pub fn ensure_paths_exist() -> Result<(), std::io::Error> {
    let dirs = vec![
        paths::app_config_dir(),
        paths::app_cache_dir(),
        paths::app_data_dir(),
        paths::metadata_history_dir(),
    ];

    for dir in dirs {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
    }

    Ok(())
}
