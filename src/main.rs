use std::{env, fs::File, path::PathBuf};

use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;

use music_manager::{app, config, create_default_database, logging, Args};

#[tokio::main]
async fn main() {
    logging::init().expect("Failed to initialize logging");
    dotenv().ok();

    app::ensure_paths_exist().expect("Failed to ensure paths exist");

    tracing::info!(
        "Launching {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let args = Args::parse();
    let settings = config::load(&args).expect("Failed to load settings");

    let database_url = match &settings.server.database_url {
        Some(url) => url,
        None => {
            let info_msg = "No database URL specified. Using default database."
                .blue()
                .bold()
                .to_string();

            tracing::info!("{info_msg}");

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
