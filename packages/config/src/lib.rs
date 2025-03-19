use config::{Config, ConfigError};
use model::AppConfig;

pub mod model;

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // get config toml dir from env, with default
        let config_dir =
            std::env::var("SOUNDOME__CONFIG_DIR").unwrap_or_else(|_| String::from("config.toml"));

        let config = Config::builder()
            // Add in `./Settings.toml`
            .add_source(config::File::with_name(&config_dir))
            // Add in settings from the environment (with a prefix of SOUNDOME)
            .add_source(config::Environment::with_prefix("SOUNDOME").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
