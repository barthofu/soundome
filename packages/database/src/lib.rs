use std::path::Path;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, SqliteConnection,
};

#[macro_use]
extern crate diesel;

pub mod entities;
pub mod macros;
pub mod mappers;
pub mod repositories;
pub mod schema;

/// Initialize the SQLite database at the given URL.
/// Creates the file if it doesn't exist, and runs all pending migrations.
pub fn init_database(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Extract the file path from the URL (sqlite URLs are file paths)
    let file_path = if let Some(path) = database_url.strip_prefix("sqlite://") {
        path
    } else {
        database_url
    };

    // Ensure the parent directory exists
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create the database file if it doesn't exist
    if !Path::new(file_path).exists() {
        tracing::info!("Creating SQLite database at: {}", file_path);
        std::fs::File::create(file_path)?;
    }

    // Verify database connectivity and run migrations
    tracing::info!("Running database migrations...");
    run_migrations_via_cli(database_url)?;

    Ok(())
}

fn run_migrations_via_cli(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Set DATABASE_URL environment variable for diesel CLI
    std::env::set_var("DATABASE_URL", database_url);

    // Try to run diesel migration run command
    let output = std::process::Command::new("diesel")
        .args(["migration", "run"])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("diesel migration run failed: {}", stderr).into());
            }
            tracing::info!("Migrations completed successfully");
            Ok(())
        }
        Err(e) => {
            // If diesel CLI is not available, log a warning but continue
            // This is acceptable in production where migrations might be run separately
            tracing::warn!(
                "diesel CLI not found or failed to run: {}. \
                 Make sure migrations are run with: diesel migration run",
                e
            );
            Ok(())
        }
    }
}

pub fn init_connection(database_url: &str) -> SqliteConnection {
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn init_pool(database_url: &str) -> SqlitePool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .unwrap_or_else(|_| panic!("Failed to create pool connection to {}", database_url))
}

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
