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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage the local library.
    Library {
        #[command(subcommand)]
        command: LibraryCommands,
    },

    /// Walk the library directory and reconcile the filesystem against the database.
    Scan {
        /// Override the library root to scan. Defaults to `general.base_library_dir` in config.
        #[arg(long)]
        library_root: Option<String>,

        /// Report only — do not apply any mutations to the database.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
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
        Commands::Scan {
            library_root,
            dry_run,
        } => {
            commands::scan::scan(&client, library_root.as_deref(), dry_run).await?;
        }
    }

    Ok(())
}
