use serde::Deserialize;
use std::path::Path;

pub use error::Error;

pub const DEFAULT_CONFIG_FILE: &str = "/etc/davyjones/config.toml";

mod error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub nfty: Ntfy,
    pub topic: Topic,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ntfy {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Topic {
    pub default: String,
}

pub fn load(path: &Path) -> Result<Config, Error> {
    let config_file = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_file)?;

    Ok(config)
}
