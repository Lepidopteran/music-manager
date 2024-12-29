use std::path::PathBuf;
use clap::Parser;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "Music Manager", version, author)]
pub struct Args {
    /// Database URL to connect to
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,

    /// IP address to bind to
    #[arg(long, action = clap::ArgAction::SetTrue, env = "LISTEN_ON_ALL_INTERFACES")]
    pub host: Option<bool>,

    /// Port to bind to
    #[arg(long, env = "PORT")]
    pub port: Option<u16>,

    /// Path to config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
