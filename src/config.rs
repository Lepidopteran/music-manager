use color_eyre::{
    eyre::{Error, Result},
    owo_colors::OwoColorize,
};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use std::{
    fs::{read_to_string, File},
    io::Write,
    net::IpAddr,
    path::{Path, PathBuf},
};

use crate::Args;

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
    /// Load settings from a TOML file located at the specified path.
    pub fn load(path: &Path) -> Result<Self, Error> {
        Ok(toml::from_str::<Self>(&read_to_string(path)?)?)
    }
}

/// Loads the application configuration using `Settings::load`. Handles default
/// configuration paths, argument overrides, and creates a default configuration if necessary.
pub fn load(args: &Args) -> Result<Settings, Error> {

    if let Some(config) = &args.config {
        let mut settings = Settings::load(config)?;
        override_config(&mut settings, args);

        tracing::info!("Using config file: {}", config.display());

        return Ok(settings);
    }

    let path = crate::get_app_config_dir().join("config.toml");
    let mut settings = match Settings::load(&path) {
        Ok(settings) => settings,
        Err(err) => {
            if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                if io_error.kind() != std::io::ErrorKind::NotFound {
                    return Err(err);
                }
            } else {
                return Err(err);
            }

            let info_msg = "No config file found. Creating default config."
                .blue()
                .bold()
                .to_string();

            tracing::info!("{info_msg}");

            let settings = Settings::default();
            create_default_config(path, &settings)?;

            settings
        }
    };

    override_config(&mut settings, args);

    Ok(settings)
}

fn override_config(config: &mut Settings, args: &Args) {
    if let Some(url) = &args.database_url {
        config.server.database_url = Some(url.to_string());
    }

    if let Some(port) = &args.port {
        config.server.port = *port;
    }

    if let Some(host) = &args.host {
        config.server.listen_on_all_interfaces = *host;
    }
}

fn create_default_config(path: PathBuf, settings: &Settings) -> Result<(), Error> {
    let template = include_str!("../templates/config.toml");

    let config = {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("config", template)?;
        handlebars.render("config", settings)?
    };

    let mut file = File::create(&path)?;
    file.write_all(config.as_bytes())?;

    Ok(())
}
