use std::{
    env,
    fs::File,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::signal;

use muusik::{
    APP_DIRECTORIES, AppState, Args, create_default_database, initialize_logging, load_config,
    routes, run_migrations,
};

#[tokio::main]
async fn main() {
    initialize_logging();
    dotenv().ok();

    ensure_paths_exist().expect("Failed to ensure paths exist");

    tracing::info!(
        "Launching {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let args = Args::parse();
    let settings = load_config(&args).expect("Failed to load settings");

    let database_url = match &settings.server.database_url {
        Some(url) if !url.trim().is_empty() => url,
        _ => {
            let info_msg = "No database URL specified. Using default database."
                .blue()
                .bold()
                .to_string();

            tracing::info!("{info_msg}");

            &create_default_database("data").expect("Failed to create default database")
        }
    };

    tracing::info!("Database URL: {}", database_url.underline().blue());

    let database_options = SqliteConnectOptions::from_str(database_url)
        .expect("Invalid database URL")
        .create_if_missing(true);

    let is_new_database = !database_options.get_filename().exists();

    let pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(database_options)
        .await
        .expect("Failed to connect to database");

    run_migrations(&pool, is_new_database)
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

    let state = AppState::new(pool, settings);

    tracing::info!(
        "Listening on {}{}",
        "http://".underline().blue(),
        addr.underline().blue()
    );

    axum::serve(listener, routes(state))
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
    for dir in APP_DIRECTORIES.iter() {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
    }

    Ok(())
}
