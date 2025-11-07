use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use std::env;

const DEFAULT_CONFIG_FILE: &str = "configuration.yaml";

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = env::current_dir().expect("failed to determine current directory");
    let settings = Config::builder()
        .add_source(File::from(base_path.join(DEFAULT_CONFIG_FILE)))
        .add_source(
            Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
