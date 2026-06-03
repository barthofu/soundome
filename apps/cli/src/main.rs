<<<<<<< HEAD
use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use config::Config;
use database::repositories;
use domain::{
    ports::repositories::RepositoryLayer,
    services::{scan_service::ScanReport, ServiceLayer},
};
use shared::init_globals;

#[derive(Parser)]
#[command(name = "soundome", about = "Soundome CLI")]
struct Cli {
=======
mod api;
mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use api::ApiClient;

const DEFAULT_API_URL: &str = "http://localhost:8000";

/// Soundome CLI — interact with your local Soundome library via the API.
#[derive(Parser)]
#[command(name = "soundome", version, about, long_about = None)]
struct Cli {
    /// Soundome server base URL.
    #[arg(long, env = "SOUNDOME_API_URL", default_value = DEFAULT_API_URL, global = true)]
    api_url: String,

>>>>>>> origin/main
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
<<<<<<< HEAD
    /// Walk the library directory and reconcile the filesystem against the database.
    Scan {
        /// Root of the library directory to scan. Defaults to `general.base_library_dir`
        /// from the active config.
        #[arg(long)]
        library_root: Option<PathBuf>,

        /// Report only — do not apply any mutations to the database.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[dotenvy::load(path = "./.env", required = false)]
#[tokio::main]
async fn main() {
    init_globals().unwrap_or_else(|err| {
        eprintln!("Failed to initialize globals: {}", err);
        std::process::exit(1);
    });

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            library_root,
            dry_run,
        } => {
            let root = library_root
                .unwrap_or_else(|| PathBuf::from(&Config::get().general.base_library_dir));

            let db_url = Config::get().database.url.clone();
            let conn = &mut database::init_connection(&db_url);

            let track_repo = Arc::new(repositories::track::DieselTrackRepository::new());
            let album_repo = Arc::new(repositories::album::DieselAlbumRepository::new());
            let artist_repo = Arc::new(repositories::artist::DieselArtistRepository::new());
            let playlist_repo = Arc::new(repositories::playlist::DieselPlaylistRepository::new());
            let task_repo = Arc::new(repositories::task::DieselTaskRepository::new());
            let sync_schedule_repo =
                Arc::new(repositories::sync_schedule::DieselSyncScheduleRepository::new());

            let repositories = Arc::new(RepositoryLayer {
                track: track_repo,
                album: album_repo,
                artist: artist_repo,
                playlist: playlist_repo,
                task: task_repo,
                sync_schedule: sync_schedule_repo,
            });
            let services = ServiceLayer::new(repositories);

            println!("Scanning library at {:?} (dry_run={})", root, dry_run);

            match services.scan_service.scan(conn, &root, dry_run) {
                Ok(report) => print_scan_report(&report),
                Err(e) => {
                    eprintln!("Scan failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_scan_report(report: &ScanReport) {
    println!("\n=== Scan Report ===");
    println!("Library root : {}", report.library_root);
    println!("Dry run      : {}", report.dry_run);
    println!();
    println!("  ok            : {}", report.ok);
    println!(
        "  path_changed  : {} (updated: {})",
        report.path_changed, report.paths_updated
    );
    println!(
        "  tag_conflict  : {} (flagged: {})",
        report.tag_conflict, report.conflicts_flagged
    );
    println!("  missing       : {}", report.missing);
    println!("  orphan        : {}", report.orphan);
    println!("  legacy_match  : {}", report.legacy_match);
    println!("  unmanaged     : {}", report.unmanaged);
    println!();

    for entry in &report.entries {
        if !matches!(
            entry.category,
            domain::services::scan_service::ScanCategory::Ok
        ) {
            println!(
                "  [{:?}] {:?}  (track_id={:?}  title={:?})",
                entry.category, entry.file_path, entry.track_id, entry.title
            );
        }
    }
=======
    /// Manage the local library.
    Library {
        #[command(subcommand)]
        command: LibraryCommands,
    },
}

#[derive(Subcommand)]
enum LibraryCommands {
    /// Search the library with optional filters.
    Search {
        /// Entity to search.
        #[arg(value_enum)]
        entity: SearchEntity,

        /// Free-text query (title/name/artist depending on entity).
        #[arg(short, long)]
        query: Option<String>,

        /// Filter by source (playlists only).
        #[arg(long)]
        source: Option<String>,

        /// Filter by genre (tracks only).
        #[arg(long)]
        genre: Option<String>,

        /// Filter tracks requiring validation (tracks only).
        #[arg(long)]
        needs_validation: bool,

        /// Filter tracks with an available local file (tracks only).
        #[arg(long)]
        has_file: bool,

        /// Maximum number of results.
        #[arg(long)]
        limit: Option<usize>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Playlist operations.
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommands,
    },
}

#[derive(Subcommand)]
enum PlaylistCommands {
    /// List all playlists in the library.
    List {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Download all local tracks from a playlist into a directory.
    Download {
        /// Playlist ID (numeric) or name (partial match).
        playlist: String,

        /// Output directory (defaults to current directory).
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Write files directly into --output (skip the playlist sub-directory).
        #[arg(long, default_value_t = false)]
        flat: bool,

        /// Only download new files (skip files already present at destination).
        #[arg(long, default_value_t = false)]
        sync: bool,

        /// Optional manifest output path (default: <target>/manifest.json).
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Jsonl,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum SearchEntity {
    Tracks,
    Albums,
    Artists,
    Playlists,
}

#[dotenvy::load(path = "./.env", required = false)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.api_url);

    match cli.command {
        Commands::Library { command } => match command {
            LibraryCommands::Search {
                entity,
                query,
                source,
                genre,
                needs_validation,
                has_file,
                limit,
                format,
            } => {
                commands::library::search::search(
                    &client,
                    entity,
                    query.as_deref(),
                    source.as_deref(),
                    genre.as_deref(),
                    needs_validation,
                    has_file,
                    limit,
                    format,
                )
                .await?;
            }
            LibraryCommands::Playlist { command } => match command {
                PlaylistCommands::List { format } => {
                    commands::library::playlist::list(&client, format).await?;
                }
                PlaylistCommands::Download {
                    playlist,
                    output,
                    flat,
                    sync,
                    manifest,
                } => {
                    commands::library::playlist::download(
                        &client,
                        &playlist,
                        &output,
                        flat,
                        sync,
                        manifest.as_deref(),
                    )
                    .await?;
                }
            },
        },
    }

    Ok(())
>>>>>>> origin/main
}
