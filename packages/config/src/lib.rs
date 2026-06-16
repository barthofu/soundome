use std::sync::OnceLock;

// exports
pub mod models;

// re-exports
pub use models::Config;

pub static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn init() -> Result<(), config::ConfigError> {
        let config = Self::load()?;
        GLOBAL_CONFIG
            .set(config)
            .map_err(|_| config::ConfigError::Message("Failed to set global config".into()))
    }

    pub fn load() -> Result<Self, config::ConfigError> {
        // get config toml dir from env, with default
        let config_path =
            std::env::var("SOUNDOME_CONFIG_PATH").unwrap_or_else(|_| String::from("./config.toml"));

        let config = config::Config::builder()
            // Add in config toml (optional, allows config-from-env-only setup)
            .add_source(config::File::with_name(&config_path).required(false))
            // Add in settings from the environment (with a prefix of SOUNDOME)
            .add_source(config::Environment::with_prefix("SOUNDOME").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn get() -> &'static Self {
        GLOBAL_CONFIG.get().expect("Config is not initialized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads_without_file() {
        // The main goal is to verify that config loading doesn't fail
        // when the config file doesn't exist. We'll check that defaults are applied.
        let result = Config::load();
        assert!(
            result.is_ok(),
            "Config should load successfully even when no file exists"
        );

        let config = result.unwrap();
        // Verify that defaults are applied when values are missing
        assert!(
            !config.database.url.is_empty(),
            "Database URL should have a default"
        );
        assert!(
            !config.general.base_library_dir.is_empty(),
            "Base library dir should have a default"
        );
    }

    #[test]
    fn test_config_database_defaults() {
        // Load config and verify database defaults are applied
        let result = Config::load();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.database.url, "./data/soundome.db");
    }

    #[test]
    fn test_config_general_defaults() {
        // Load config and verify general defaults are applied
        let result = Config::load();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.general.base_library_dir, "./library");
        assert_eq!(config.general.temp_download_dir, "./temp");
    }
}
