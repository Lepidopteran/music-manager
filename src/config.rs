use color_eyre::eyre::{Error, Result};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    net::IpAddr,
    path::PathBuf,
};

/// Server configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
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
    pub database_url: Option<String>,
    pub server: Server,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_url: None,
            server: Server {
                listen_on_all_interfaces: false,
                port: 3000,
                host: None,
            },
        }
    }
}

impl Settings {
    /// Loads the application configuration.
    pub fn load(path: Option<PathBuf>) -> Result<Self, Error> {
        let path = match path {
            Some(path) => path,
            None => {
                let path = crate::get_app_config_dir();

                if !path.exists() {
                    create_dir_all(&path)?;
                }

                path.join("config.toml")
            }
        };

        let mut settings = Self::default();

        if !path.exists() {
            create_default_config(path.clone(), &settings)?;
        }

        let config: Option<String> = match read_to_string(&path) {
            Ok(config) => Some(config),
            Err(_) => None,
        };

        if let Some(config) = config {
            settings = toml::from_str(&config)?;
        }

        Ok(settings)
    }
}

fn create_default_config(path: PathBuf, settings: &Settings) -> Result<(), Error> {
    let template = include_str!("../templates/config.toml");

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("config", template)?;
    let config = handlebars.render("config", settings)?;
    let mut file = File::create(&path)?;
    file.write_all(config.as_bytes())?;

    Ok(())
}
