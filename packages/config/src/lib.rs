use config::{Config, ConfigError};
use model::AppConfig;

pub mod model;

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // get config toml dir from env, with default
        let config_path =
            std::env::var("SOUNDOME_CONFIG_PATH").unwrap_or_else(|_| String::from("./config.toml"));

        let config = Config::builder()
            // Add in config toml
            .add_source(config::File::with_name(&config_path))
            // Add in settings from the environment (with a prefix of SOUNDOME)
            .add_source(config::Environment::with_prefix("SOUNDOME").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
