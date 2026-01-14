mod args;

pub mod app;
pub mod task;
pub mod config; 
pub mod metadata;
pub mod utils;
pub mod db;
pub mod paths;
pub mod logging;
pub mod migration;

mod ui;
mod events;
mod tasks;
mod api;

pub use args::*;
pub use utils::*;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
