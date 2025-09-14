use std::str::FromStr;

use config::Config;
use tracing::Level;

pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_max_level(
            Level::from_str(&Config::get().logs.level)
                .unwrap_or(Level::INFO)
        )
        .init();
}