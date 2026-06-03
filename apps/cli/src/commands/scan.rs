use crate::api::{models::ScanCategory, ApiClient};

pub async fn scan(
    client: &ApiClient,
    library_root: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let report = client.scan(library_root, dry_run).await?;

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
        if entry.category != ScanCategory::Ok {
            println!(
                "  [{:?}] {:?}  (track_id={:?}  title={:?})",
                entry.category, entry.file_path, entry.track_id, entry.title
            );
        }
    }

    Ok(())
}
