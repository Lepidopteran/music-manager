//! Utility functions for working with paths.

use directories::BaseDirs;
use std::{env, path::PathBuf};

/// Get the path to the app config directory.
pub fn app_config_dir() -> PathBuf {
    let app_name = env!("CARGO_PKG_NAME");

    if let Ok(config_dir) = env::var(format!("{}_CONFIG_DIR", app_name.to_uppercase()).as_str()) {
        return PathBuf::from(config_dir);
    }

    if let Some(base_dirs) = BaseDirs::new() {
        return base_dirs.config_dir().join(app_name);
    }

    env::current_dir().expect("Failed to get current directory")
}

/// Get the path to the app data directory.
pub fn app_data_dir() -> PathBuf {
    let app_name = env!("CARGO_PKG_NAME");

    if let Ok(data_dir) = env::var(format!("{}_DATA_DIR", app_name.to_uppercase()).as_str()) {
        return PathBuf::from(data_dir);
    }

    if let Some(base_dirs) = BaseDirs::new() {
        return base_dirs.data_dir().join(app_name);
    }

    env::current_dir().expect("Failed to get current directory")
}

/// Get the path to the app cache directory.
pub fn app_cache_dir() -> PathBuf {
    let app_name = env!("CARGO_PKG_NAME");

    if let Ok(cache_dir) = env::var(format!("{}_CACHE_DIR", app_name.to_uppercase()).as_str()) {
        return PathBuf::from(cache_dir);
    }

    if let Some(base_dirs) = BaseDirs::new() {
        return base_dirs.cache_dir().join(app_name);
    }

    env::current_dir().expect("Failed to get current directory")
}
