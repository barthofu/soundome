// imports
mod libs;

// exports
pub mod errors;
pub mod models;
pub mod types;
pub mod utils;

// re-exports
pub use libs::*;
// pub use utils::logs;

// ===============================================================================

// Special function to initialize all global singletons
pub fn init_globals() -> crate::types::SoundomeResult<()> {
    config::Config::init()
        .map_err(|e| crate::errors::Error::Config(format!("Failed to initialize config: {}", e)))?;
    libs::http::ProxyRotator::init()?;
    Ok(())
}