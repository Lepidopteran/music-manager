use color_eyre::owo_colors::OwoColorize;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use std::{
    fs::{read_to_string, File},
    net::IpAddr,
    path::{Path, PathBuf},
};

use crate::{paths, Args};

type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),
}

/// Server configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    /// Database URL to connect to
    pub database_url: Option<String>,

    /// Whether to listen on all interfaces
    pub listen_on_all_interfaces: bool,

    /// Port to bind to
    pub port: u16,

    /// IP address to bind to (overrides `listen_on_all_interfaces` if set)
    pub host: Option<IpAddr>,
}

/// Application settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: Server {
                listen_on_all_interfaces: false,
                port: 3000,
                host: None,
                database_url: None,
            },
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        Ok(toml::from_str::<Self>(&read_to_string(path)?)?)
    }
}

/// Loads the application configuration using `Settings::load`. Handles default
/// configuration paths, argument overrides, and creates a default configuration if necessary.
pub fn load_config(args: &Args) -> Result<Settings> {
    if let Some(config) = &args.config {
        let mut settings = Settings::load(config)?;
        override_config(&mut settings, args);

        tracing::info!("Using config file: {}", config.display());

        return Ok(settings);
    }

    let path = paths::app_config_dir().join("config.toml");
    let mut settings = Settings::load(&path).or_else(|err| {
        if let ConfigError::Io(ref io_err) = err && io_err.kind() == std::io::ErrorKind::NotFound {

            let info_msg = "No config file found. Creating default config."
                .blue()
                .bold()
                .to_string();

            tracing::info!("{info_msg}");

            let settings = Settings::default();


    let file = File::create(&path).expect("Failed to create config file");
    let template = include_str!("../templates/config.toml");

    Handlebars::new()
        .render_template_to_write(template, &settings, file)
        .expect("Failed to write config file");

            Ok(settings)
        }
        else {
            Err(err)
        }
    })?;

            override_config(&mut settings, args);

    Ok(settings)
}

fn override_config(settings: &mut Settings, args: &Args) {
    args.database_url
        .as_ref()
        .map(|url| settings.server.database_url.replace(url.to_string()));

    args.host.map(|listen_on_all_interfaces| {
        settings.server.listen_on_all_interfaces = listen_on_all_interfaces
    });

    args.port.map(|port| settings.server.port = port);
}
