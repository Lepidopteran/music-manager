pub use args::*;

pub mod app;
pub mod config;
pub mod logging;
pub mod metadata;
pub mod paths;

mod args;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
