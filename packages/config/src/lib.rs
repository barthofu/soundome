use std::sync::OnceLock;

// exports
pub mod models;

// re-exports
pub use models::Config;

pub static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {

    pub fn init() -> Result<(), config::ConfigError> {
        let config = Self::load()?;
        GLOBAL_CONFIG.set(config).map_err(|_| {
            config::ConfigError::Message("Failed to set global config".into())
        })
    }

    pub fn load() -> Result<Self, config::ConfigError> {
        // get config toml dir from env, with default
        let config_path =
            std::env::var("SOUNDOME_CONFIG_PATH").unwrap_or_else(|_| String::from("./config.toml"));

        let config = config::Config::builder()
            // Add in config toml
            .add_source(config::File::with_name(&config_path))
            // Add in settings from the environment (with a prefix of SOUNDOME)
            .add_source(config::Environment::with_prefix("SOUNDOME").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn get() -> &'static Self {
        GLOBAL_CONFIG.get().expect("Config is not initialized")
    }
}