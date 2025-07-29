//! Methods for getting app paths.

use directories::{ProjectDirs};
use std::{env, path::PathBuf};
use crate::APP_NAME;


/// Get the path to the app config directory.
pub fn app_config_dir() -> PathBuf {
    if let Ok(config_dir) = env::var(format!("{}_CONFIG_DIR", APP_NAME.to_uppercase()).as_str()) {
        return PathBuf::from(config_dir);
    }

    if let Some(base_dirs) = project_dirs() {
        return base_dirs.config_dir().into();
    }

    env::current_dir().expect("Failed to get current directory")
}

/// Get the path to the app data directory.
pub fn app_data_dir() -> PathBuf {
    if let Ok(data_dir) = env::var(format!("{}_DATA_DIR", APP_NAME.to_uppercase()).as_str()) {
        return PathBuf::from(data_dir);
    }

    if let Some(base_dirs) = project_dirs() {
        return base_dirs.data_dir().into();
    }

    env::current_dir().expect("Failed to get current directory")
}

/// Get the path to the metadata history directory.
pub fn metadata_history_dir() -> PathBuf {
    app_data_dir().join("history")
}

/// Get the path to the app cache directory.
pub fn app_cache_dir() -> PathBuf {
    if let Ok(cache_dir) = env::var(format!("{}_CACHE_DIR", APP_NAME.to_uppercase()).as_str()) {
        return PathBuf::from(cache_dir);
    }

    if let Some(base_dirs) = project_dirs() {
        return base_dirs.cache_dir().into();
    }

    env::current_dir().expect("Failed to get current directory")
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("org", "muusik", "Muusik")
}
