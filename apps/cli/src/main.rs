mod api;
mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
}

#[derive(Subcommand)]
enum LibraryCommands {
    /// Playlist operations.
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommands,
    },
}

#[derive(Subcommand)]
enum PlaylistCommands {
    /// List all playlists in the library.
    List,

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
    },
}

#[dotenvy::load(path = "./.env", required = false)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.api_url);

    match cli.command {
        Commands::Library { command } => match command {
            LibraryCommands::Playlist { command } => match command {
                PlaylistCommands::List => {
                    commands::library::playlist::list(&client).await?;
                }
                PlaylistCommands::Download {
                    playlist,
                    output,
                    flat,
                } => {
                    commands::library::playlist::download(&client, &playlist, &output, flat)
                        .await?;
                }
            },
        },
    }

    Ok(())
}
