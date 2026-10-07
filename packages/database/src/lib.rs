use std::path::Path;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
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
        std::fs::File::create(file_path)
            .map_err(|e| format!("Failed to create database file '{}': {}", file_path, e))?;
    }

    // Run pending migrations using the embedded migrations compiled into the binary
    tracing::info!("Running database migrations...");
    let mut conn = SqliteConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database '{}': {}", database_url, e))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run database migrations: {}", e))?;

    tracing::info!("Database migrations completed successfully");

    let normalized_paths = normalize_existing_track_file_paths(&mut conn)
        .map_err(|e| format!("Failed to normalize stored track paths: {}", e))?;
    if normalized_paths > 0 {
        tracing::info!(
            "Normalized {} stored track file path(s) relative to the configured audio roots",
            normalized_paths
        );
    }

    Ok(())
}

/// Convert legacy absolute (or root-prefixed relative) track paths to paths
/// relative to the configured library or staging root. This is idempotent and
/// only updates database values; it never moves or removes audio files.
fn normalize_existing_track_file_paths(
    conn: &mut SqliteConnection,
) -> Result<usize, diesel::result::Error> {
    conn.transaction::<usize, diesel::result::Error, _>(|conn| {
        let tracks = schema::track::table.load::<entities::TrackEntity>(conn)?;
        let mut normalized_count = 0;

        for track in tracks {
            let Some(stored_path) = track.file_path else {
                continue;
            };
            let normalized_path = shared::utils::fs::track_file_path_for_storage_from_config(
                Path::new(&stored_path),
                track.needs_validation,
            );
            let normalized_path = normalized_path.to_string_lossy().into_owned();

            if normalized_path != stored_path {
                diesel::update(schema::track::table.filter(schema::track::id.eq(track.id)))
                    .set(schema::track::file_path.eq(Some(normalized_path)))
                    .execute(conn)?;
                normalized_count += 1;
            }
        }

        Ok(normalized_count)
    })
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
