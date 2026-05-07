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
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
}
