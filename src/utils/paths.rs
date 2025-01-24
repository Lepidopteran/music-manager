//! Utility functions for working with paths.

use dirs::config_dir;
use std::{env, path::PathBuf};

/// Get the path to the config directory.
pub fn get_config_dir() -> PathBuf {
    let app_name = env!("CARGO_PKG_NAME");

    if let Ok(config_dir) = env::var(format!("{}_CONFIG_DIR", app_name.to_uppercase()).as_str()) {
        return PathBuf::from(config_dir);
    }

    if let Some(config_dir) = config_dir() {
        return config_dir;
    }

    env::current_dir().expect("Failed to get current directory")
}

/// Get the path to the app config directory.
pub fn get_app_config_dir() -> PathBuf {
    let app_name = env!("CARGO_PKG_NAME");

    if let Ok(config_dir) = env::var(format!("{}_CONFIG_DIR", app_name.to_uppercase()).as_str()) {
        return PathBuf::from(config_dir);
    }

    if let Some(config_dir) = config_dir() {
        return config_dir.join(app_name);
    }

    env::current_dir().expect("Failed to get current directory")
}
