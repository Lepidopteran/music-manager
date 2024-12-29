use std::{env, fs::File, path::PathBuf};

use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;

use music_manager::{app, config::Settings, create_default_database, logging, Args};

#[tokio::main]
async fn main() {
    logging::init().expect("Failed to initialize logging");
    dotenv().ok();

    tracing::info!(
        "Launching {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let args = Args::parse();
    let mut settings = Settings::load(args.config).expect("Failed to load settings");

    if let Some(url) = &args.database_url {
        settings.database_url = Some(url.to_string());
    }

    if let Some(port) = &args.port {
        settings.server.port = *port;
    }

    if let Some(host) = &args.host {
        settings.server.listen_on_all_interfaces = *host;
    }

    let database_url = match &settings.database_url {
        Some(url) => url,
        None => {
            tracing::info!(
                "{}",
                "No database URL specified. Using default database."
                    .blue()
                    .bold()
            );

            &create_default_database("data").expect("Failed to create default database")
        }
    };

    if let Some(database_url) = database_url.strip_prefix("sqlite://") {
        let path = PathBuf::from(database_url);

        if !path.exists() {
            File::create(&path).expect("Failed to create database file");
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect(database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    app::serve(settings, pool).await;
}
