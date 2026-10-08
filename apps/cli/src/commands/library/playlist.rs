use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use audiotags::Tag;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use crate::api::models::{PlaylistDto, PlaylistTrackDto};
use crate::api::ApiClient;
use crate::OutputFormat;

const STATUS_DOWNLOADED: &str = "downloaded";
const STATUS_SKIPPED: &str = "skipped";
const STATUS_FAILED: &str = "failed";

#[derive(Debug, Serialize, Deserialize)]
struct DownloadManifest {
    playlist_id: i32,
    playlist_name: String,
    output_root: String,
    total_tracks: usize,
    downloaded: usize,
    skipped: usize,
    #[serde(default)]
    renamed: usize,
    #[serde(default)]
    removed: usize,
    failed: usize,
    entries: Vec<DownloadManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadManifestEntry {
    index: usize,
    track_id: i32,
    track_title: String,
    artists: Vec<String>,
    destination: String,
    /// File name relative to the output root. Stable across working directories,
    /// used to locate the file on the next sync. Older manifests do not have it.
    #[serde(default)]
    file_name: Option<String>,
    status: String,
    message: Option<String>,
}

/// Options for `library playlist download`.
pub struct DownloadOptions<'a> {
    /// Write files directly into the output directory.
    pub flat: bool,
    /// Re-download every track, even when an up-to-date local copy exists.
    pub force: bool,
    /// Prefix the playlist directory with the zero-padded playlist ID (`0001 - Name`).
    pub with_playlist_id: bool,
    /// Prefix file names with the zero-padded playlist position (`01 - Artist - Title`).
    pub with_track_number: bool,
    /// Custom manifest path (default: `<target>/manifest.json`).
    pub manifest_path: Option<&'a Path>,
}

/// A file written by a previous run, as recorded in the manifest.
struct PreviousFile {
    path: PathBuf,
    index: usize,
    total: usize,
}

/// What to do for one playlist track during this run.
struct PlannedTrack<'a> {
    position: usize,
    track: &'a PlaylistTrackDto,
    file_name: String,
    dest: PathBuf,
    /// Local file to reuse instead of downloading (may need a rename to `dest`).
    existing: Option<PathBuf>,
    /// Whether the track-number tags must be rewritten on the reused file.
    retag: bool,
    renamed: bool,
}

/// Resolve a playlist by ID (if numeric) or by name (fuzzy case-insensitive).
async fn resolve_playlist(client: &ApiClient, id_or_name: &str) -> anyhow::Result<PlaylistDto> {
    let playlists = client.get_playlists().await?;

    if let Ok(id) = id_or_name.parse::<i32>() {
        return playlists
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("No playlist found with id {}", id));
    }

    let needle = id_or_name.to_lowercase();
    let matched: Vec<_> = playlists
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&needle))
        .collect();

    match matched.len() {
        0 => anyhow::bail!("No playlist matching '{}'", id_or_name),
        1 => Ok(matched[0].clone()),
        _ => {
            let names: Vec<_> = matched.iter().map(|p| p.name.as_str()).collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Multiple playlists matched — pick one")
                .items(&names)
                .default(0)
                .interact()?;
            Ok(matched[selection].clone())
        }
    }
}

/// List all playlists available in the library.
pub async fn list(client: &ApiClient, format: OutputFormat) -> anyhow::Result<()> {
    let playlists = client.get_playlists().await?;

    match format {
        OutputFormat::Table => {
            if playlists.is_empty() {
                println!("{}", style("No playlists found.").yellow());
                return Ok(());
            }

            println!(
                "{:>4}  {:<40}  {}",
                style("ID").bold().dim(),
                style("Name").bold().dim(),
                style("Source").bold().dim()
            );
            println!("{}", style("─".repeat(64)).dim());

            for p in &playlists {
                println!(
                    "{:>4}  {:<40}  {}",
                    style(p.id).cyan(),
                    p.name,
                    style(&p.source).dim()
                );
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&playlists).context("Failed to render JSON")?
            );
        }
        OutputFormat::Jsonl => {
            for playlist in playlists {
                println!(
                    "{}",
                    serde_json::to_string(&playlist).context("Failed to render JSONL row")?
                );
            }
        }
    }

    Ok(())
}

/// Download all local tracks from a playlist via the API (HTTP streaming).
///
/// Behaves as a synchronisation when a manifest from a previous run exists:
/// tracks already present locally are kept (renamed / re-tagged if their
/// position changed), new tracks are downloaded, and files of tracks removed
/// from the playlist are deleted.
pub async fn download(
    client: &ApiClient,
    id_or_name: &str,
    output: &Path,
    opts: DownloadOptions<'_>,
) -> anyhow::Result<()> {
    let playlist = resolve_playlist(client, id_or_name).await?;
    let tracks = client.get_playlist_tracks(playlist.id).await?;

    let playlist_dir_name = if opts.with_playlist_id {
        format!("{:04} - {}", playlist.id, sanitize_filename(&playlist.name))
    } else {
        sanitize_filename(&playlist.name)
    };
    let target_root = if opts.flat {
        output.to_path_buf()
    } else {
        output.join(playlist_dir_name)
    };

    let manifest_dest = opts
        .manifest_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| target_root.join("manifest.json"));
    let previous_manifest = load_previous_manifest(&manifest_dest, playlist.id).await;

    if tracks.is_empty() && previous_manifest.is_none() {
        println!("{}", style("Playlist is empty.").yellow());
        return Ok(());
    }

    tokio::fs::create_dir_all(&target_root).await?;

    // ── Files owned by the previous run ──────────────────────────────────────
    let mut previous_files: HashMap<i32, VecDeque<PreviousFile>> = HashMap::new();
    let mut known_previous_paths: HashSet<PathBuf> = HashSet::new();
    if let Some(manifest) = &previous_manifest {
        for entry in &manifest.entries {
            if entry.status == STATUS_FAILED {
                continue;
            }
            let Some(path) = previous_entry_path(&target_root, entry) else {
                continue;
            };
            if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
                continue;
            }
            known_previous_paths.insert(path.clone());
            previous_files
                .entry(entry.track_id)
                .or_default()
                .push_back(PreviousFile {
                    path,
                    index: entry.index,
                    total: manifest.total_tracks,
                });
        }
    }

    // ── Plan ─────────────────────────────────────────────────────────────────
    let total = tracks.len();
    let mut used_names: HashSet<String> = HashSet::new();
    let mut plans: Vec<PlannedTrack> = Vec::with_capacity(total);

    for (index, track) in tracks.iter().enumerate() {
        let position = index + 1;
        let artist_names: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let ext = track
            .file_path
            .as_deref()
            .and_then(|p| Path::new(p).extension())
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");

        let order = opts
            .with_track_number
            .then_some((position as u32, total as u32));
        let file_name = build_file_name(&artist_names, &track.title, ext, order, &mut used_names);
        let dest = target_root.join(&file_name);

        let mut existing = None;
        let mut retag = true;
        if !opts.force {
            if let Some(prev) = previous_files
                .get_mut(&track.id)
                .and_then(|queue| queue.pop_front())
            {
                retag = prev.index != position || prev.total != total;
                existing = Some(prev.path);
            } else if !known_previous_paths.contains(&dest)
                && tokio::fs::try_exists(&dest).await.unwrap_or(false)
            {
                // Untracked file at the expected location (no/legacy manifest,
                // interrupted run): adopt it rather than re-downloading.
                existing = Some(dest.clone());
            }
        }

        plans.push(PlannedTrack {
            position,
            track,
            file_name,
            dest,
            existing,
            retag,
            renamed: false,
        });
    }

    // ── Remove files that are no longer part of the playlist ─────────────────
    let kept_paths: HashSet<&PathBuf> = plans.iter().filter_map(|p| p.existing.as_ref()).collect();
    let planned_dests: HashSet<&PathBuf> = plans.iter().map(|p| &p.dest).collect();
    let mut removed = 0usize;
    for path in &known_previous_paths {
        if kept_paths.contains(path) {
            continue;
        }
        match tokio::fs::remove_file(path).await {
            Ok(()) => {
                // Files about to be overwritten (e.g. `--force`) are not reported as removed.
                if !planned_dests.contains(path) {
                    removed += 1;
                    println!(
                        "  {} {}",
                        style("remove").red(),
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
            }
            Err(err) => println!(
                "  {} could not remove {} ({})",
                style("warn").yellow(),
                path.display(),
                err
            ),
        }
    }
    // Release the borrows on `plans` before mutating it below.
    drop(kept_paths);
    drop(planned_dests);

    // ── Rename reused files whose name changed (two phases to avoid clobbering) ─
    let mut staged: Vec<(usize, PathBuf)> = Vec::new();
    for (i, plan) in plans.iter_mut().enumerate() {
        let Some(src) = plan.existing.as_ref() else {
            continue;
        };
        if *src == plan.dest {
            continue;
        }
        let tmp = target_root.join(format!(".{}.soundome-rename", plan.file_name));
        match tokio::fs::rename(src, &tmp).await {
            Ok(()) => staged.push((i, tmp)),
            Err(err) => {
                println!(
                    "  {} could not rename {} ({}), it will be re-downloaded",
                    style("warn").yellow(),
                    src.display(),
                    err
                );
                plan.existing = None;
            }
        }
    }
    let mut renamed = 0usize;
    for (i, tmp) in staged {
        let plan = &mut plans[i];
        match tokio::fs::rename(&tmp, &plan.dest).await {
            Ok(()) => {
                plan.existing = Some(plan.dest.clone());
                plan.renamed = true;
                renamed += 1;
            }
            Err(err) => {
                println!(
                    "  {} could not rename to {} ({}), it will be re-downloaded",
                    style("warn").yellow(),
                    plan.dest.display(),
                    err
                );
                let _ = tokio::fs::remove_file(&tmp).await;
                plan.existing = None;
            }
        }
    }

    // ── Download / re-tag ────────────────────────────────────────────────────
    let multi = MultiProgress::new();

    let overall = multi.add(ProgressBar::new(total as u64));
    overall.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} tracks  {msg}",
        )?
        .progress_chars("=>-"),
    );

    let byte_bar = multi.add(ProgressBar::new(0));
    byte_bar.set_style(
        ProgressStyle::with_template(
            "  {spinner:.dim} {bytes}/{total_bytes} @ {bytes_per_sec}  {wide_msg}",
        )?
        .progress_chars("=>-"),
    );

    let mut downloaded = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut manifest_entries: Vec<DownloadManifestEntry> = Vec::with_capacity(total);

    for plan in &plans {
        let track = plan.track;
        let track_number = plan.position as u32;
        let artist_names: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let display = format!("{} — {}", artist_names.join(", "), track.title);
        overall.set_message(display.clone());

        let (status, message) = if plan.existing.is_some() {
            skipped += 1;
            let mut message = if plan.renamed {
                "already exists (renamed)".to_string()
            } else {
                "already exists".to_string()
            };

            if plan.retag {
                if let Err(err) =
                    set_playlist_order_metadata(&plan.dest, track_number, total as u32)
                {
                    overall.println(format!(
                        "  {} {} ({})",
                        style("warn").yellow(),
                        display,
                        err
                    ));
                    message = format!("{message}; tagging failed: {err}");
                }
            }

            (STATUS_SKIPPED, Some(message))
        } else {
            byte_bar.set_length(0);
            byte_bar.set_position(0);
            byte_bar.set_message(display.clone());

            match download_to(client, track.id, &plan.dest, |n| byte_bar.inc(n)).await {
                Ok(()) => {
                    downloaded += 1;
                    let mut message = None;
                    if let Err(err) =
                        set_playlist_order_metadata(&plan.dest, track_number, total as u32)
                    {
                        let msg = err.to_string();
                        overall.println(format!(
                            "  {} {} ({})",
                            style("warn").yellow(),
                            display,
                            msg
                        ));
                        message = Some(msg);
                    }
                    (STATUS_DOWNLOADED, message)
                }
                Err(err) => {
                    failed += 1;
                    overall.println(format!(
                        "  {} {} ({})",
                        style("skip").yellow(),
                        display,
                        err
                    ));
                    (STATUS_FAILED, Some(err.to_string()))
                }
            }
        };

        manifest_entries.push(DownloadManifestEntry {
            index: plan.position,
            track_id: track.id,
            track_title: track.title.clone(),
            artists: track.artists.iter().map(|a| a.name.clone()).collect(),
            destination: plan.dest.display().to_string(),
            file_name: Some(plan.file_name.clone()),
            status: status.to_string(),
            message,
        });

        overall.inc(1);
    }

    byte_bar.finish_and_clear();
    overall.finish_and_clear();

    println!(
        "{} {} track(s) downloaded to {} ({} unchanged, {} renamed, {} removed, {} failed)",
        style("✓").green().bold(),
        downloaded,
        style(target_root.display()).cyan(),
        style(skipped).yellow(),
        style(renamed).yellow(),
        style(removed).red(),
        style(failed).red()
    );

    let manifest = DownloadManifest {
        playlist_id: playlist.id,
        playlist_name: playlist.name,
        output_root: target_root.display().to_string(),
        total_tracks: total,
        downloaded,
        skipped,
        renamed,
        removed,
        failed,
        entries: manifest_entries,
    };

    if let Some(parent) = manifest_dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(
        &manifest_dest,
        serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?,
    )
    .await
    .with_context(|| format!("Failed to write manifest at {}", manifest_dest.display()))?;

    println!(
        "{} manifest written to {}",
        style("✓").green().bold(),
        style(manifest_dest.display()).cyan()
    );

    Ok(())
}

/// Load the manifest of a previous run, if any and if it belongs to the same playlist.
async fn load_previous_manifest(path: &Path, playlist_id: i32) -> Option<DownloadManifest> {
    let raw = tokio::fs::read_to_string(path).await.ok()?;
    match serde_json::from_str::<DownloadManifest>(&raw) {
        Ok(manifest) if manifest.playlist_id == playlist_id => Some(manifest),
        Ok(manifest) => {
            println!(
                "  {} manifest {} belongs to playlist {}, ignoring it",
                style("warn").yellow(),
                path.display(),
                manifest.playlist_id
            );
            None
        }
        Err(err) => {
            println!(
                "  {} could not parse manifest {} ({}), ignoring it",
                style("warn").yellow(),
                path.display(),
                err
            );
            None
        }
    }
}

/// Resolve the on-disk path of a manifest entry. Files are always direct
/// children of the target root; anything else is ignored (never deleted).
fn previous_entry_path(target_root: &Path, entry: &DownloadManifestEntry) -> Option<PathBuf> {
    let name = match &entry.file_name {
        Some(name) => Path::new(name.as_str()).to_path_buf(),
        // Legacy manifests only stored `destination`.
        None => PathBuf::from(Path::new(&entry.destination).file_name()?),
    };
    let mut components = name.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Some(target_root.join(name)),
        _ => None,
    }
}

/// Stream a track into a temporary `.part` file, then move it into place so an
/// interrupted run never leaves a truncated file at the final destination.
async fn download_to(
    client: &ApiClient,
    track_id: i32,
    dest: &Path,
    on_chunk: impl FnMut(u64),
) -> anyhow::Result<()> {
    let mut part_name = dest.file_name().unwrap_or_default().to_os_string();
    part_name.push(".part");
    let part = dest.with_file_name(part_name);

    if let Err(err) = client.download_track(track_id, &part, on_chunk).await {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(err);
    }
    tokio::fs::rename(&part, dest).await?;
    Ok(())
}

/// Build a unique file name: `[<Order> - ]<Artist> - <Title>.<ext>`.
/// Name clashes within the playlist get a ` (2)`, ` (3)`… suffix.
fn build_file_name(
    artists: &[&str],
    title: &str,
    ext: &str,
    order: Option<(u32, u32)>,
    used_names: &mut HashSet<String>,
) -> String {
    let safe_title = sanitize_filename(title);
    let mut stem = if artists.is_empty() {
        safe_title
    } else {
        format!(
            "{} - {}",
            sanitize_filename(&artists.join(", ")),
            safe_title
        )
    };

    if let Some((track_number, total_tracks)) = order {
        let width = if total_tracks >= 1000 {
            4
        } else if total_tracks >= 100 {
            3
        } else {
            2
        };
        stem = format!("{:0width$} - {}", track_number, stem, width = width);
    }

    let mut candidate = format!("{stem}.{ext}");
    let mut n = 2;
    // Case-insensitive to stay safe on case-insensitive filesystems.
    while !used_names.insert(candidate.to_lowercase()) {
        candidate = format!("{stem} ({n}).{ext}");
        n += 1;
    }
    candidate
}

fn set_playlist_order_metadata(
    path: &Path,
    track_number: u32,
    total_tracks: u32,
) -> anyhow::Result<()> {
    let mut tag = Tag::new().read_from_path(path)?;
    tag.set_track_number(track_number as u16);
    tag.set_total_tracks(total_tracks as u16);
    tag.write_to_path(path.to_string_lossy().as_ref())?;
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
