use std::path::Path;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate diesel;

pub mod entities;
pub mod macros;
pub mod mappers;
pub mod repositories;
pub mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Initialize the SQLite database at the given URL.
/// Creates the file and parent directories if they don't exist, then runs all pending migrations.
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
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create database directory '{}': {}",
                    parent.display(),
                    e
                )
            })?;
        }
    }

    // Create the database file if it doesn't exist
    if !Path::new(file_path).exists() {
        tracing::info!("Creating SQLite database at: {}", file_path);
        std::fs::File::create(file_path).map_err(|e| {
            format!("Failed to create database file '{}': {}", file_path, e)
        })?;
    }

    // Run pending migrations using the embedded migrations compiled into the binary
    tracing::info!("Running database migrations...");
    let mut conn = SqliteConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database '{}': {}", database_url, e))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run database migrations: {}", e))?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
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
