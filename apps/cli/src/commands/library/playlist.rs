use std::path::{Path, PathBuf};

use anyhow::Context;
use audiotags::Tag;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::api::models::PlaylistDto;
use crate::api::ApiClient;
use crate::OutputFormat;

#[derive(Debug, Serialize)]
struct DownloadManifest {
    playlist_id: i32,
    playlist_name: String,
    output_root: String,
    total_tracks: usize,
    downloaded: usize,
    skipped: usize,
    failed: usize,
    entries: Vec<DownloadManifestEntry>,
}

#[derive(Debug, Serialize)]
struct DownloadManifestEntry {
    index: usize,
    track_id: i32,
    track_title: String,
    artists: Vec<String>,
    destination: String,
    status: String,
    message: Option<String>,
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
pub async fn download(
    client: &ApiClient,
    id_or_name: &str,
    output: &Path,
    flat: bool,
    sync: bool,
    manifest_path: Option<&Path>,
) -> anyhow::Result<()> {
    let playlist = resolve_playlist(client, id_or_name).await?;
    let tracks = client.get_playlist_tracks(playlist.id).await?;

    if tracks.is_empty() {
        println!("{}", style("Playlist is empty.").yellow());
        return Ok(());
    }

    let playlist_dir_name = format!("{:04} - {}", playlist.id, sanitize_filename(&playlist.name));
    let target_root = if flat {
        output.to_path_buf()
    } else {
        output.join(playlist_dir_name)
    };
    tokio::fs::create_dir_all(&target_root).await?;

    let total = tracks.len() as u64;
    let multi = MultiProgress::new();

    let overall = multi.add(ProgressBar::new(total));
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

    let mut downloaded = 0u64;
    let mut skipped = 0u64;
    let mut failed = 0u64;
    let mut manifest_entries: Vec<DownloadManifestEntry> = Vec::with_capacity(tracks.len());

    for (index, track) in tracks.iter().enumerate() {
        let track_number = (index + 1) as u32;
        let artist_names: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let display = format!("{} — {}", artist_names.join(", "), track.title);
        overall.set_message(display.clone());

        let ext = track
            .file_path
            .as_deref()
            .and_then(|p| Path::new(p).extension())
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");

        let dest = build_dest_path(
            &target_root,
            &artist_names,
            &track.title,
            ext,
            track_number,
            total as u32,
        );

        if sync && tokio::fs::try_exists(&dest).await.unwrap_or(false) {
            skipped += 1;
            overall.println(format!(
                "  {} {} ({})",
                style("skip").yellow(),
                display,
                "already exists"
            ));
            manifest_entries.push(DownloadManifestEntry {
                index: index + 1,
                track_id: track.id,
                track_title: track.title.clone(),
                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                destination: dest.display().to_string(),
                status: "skipped".to_string(),
                message: Some("already exists".to_string()),
            });
            overall.inc(1);
            continue;
        }

        byte_bar.set_length(0);
        byte_bar.set_position(0);
        byte_bar.set_message(display.clone());

        let result = client
            .download_track(track.id, &dest, |n| {
                byte_bar.inc(n);
            })
            .await;

        match result {
            Ok(()) => {
                downloaded += 1;
                let mut message = None;

                if let Err(err) = set_playlist_order_metadata(&dest, track_number, total as u32) {
                    let msg = err.to_string();
                    overall.println(format!(
                        "  {} {} ({})",
                        style("warn").yellow(),
                        display,
                        msg
                    ));
                    message = Some(msg);
                }

                manifest_entries.push(DownloadManifestEntry {
                    index: index + 1,
                    track_id: track.id,
                    track_title: track.title.clone(),
                    artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                    destination: dest.display().to_string(),
                    status: "downloaded".to_string(),
                    message,
                });
            }
            Err(err) => {
                failed += 1;
                overall.println(format!(
                    "  {} {} ({})",
                    style("skip").yellow(),
                    display,
                    err
                ));
                manifest_entries.push(DownloadManifestEntry {
                    index: index + 1,
                    track_id: track.id,
                    track_title: track.title.clone(),
                    artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                    destination: dest.display().to_string(),
                    status: "failed".to_string(),
                    message: Some(err.to_string()),
                });
            }
        }

        overall.inc(1);
    }

    byte_bar.finish_and_clear();
    overall.finish_and_clear();

    println!(
        "{} {} track(s) downloaded to {}{}",
        style("✓").green().bold(),
        downloaded,
        style(target_root.display()).cyan(),
        if skipped > 0 || failed > 0 {
            format!(
                " ({} skipped, {} failed)",
                style(skipped).yellow(),
                style(failed).red()
            )
        } else {
            String::new()
        }
    );

    let manifest = DownloadManifest {
        playlist_id: playlist.id,
        playlist_name: playlist.name,
        output_root: target_root.display().to_string(),
        total_tracks: total as usize,
        downloaded: downloaded as usize,
        skipped: skipped as usize,
        failed: failed as usize,
        entries: manifest_entries,
    };

    let manifest_dest = manifest_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| target_root.join("manifest.json"));

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

fn build_dest_path(
    target_root: &Path,
    artists: &[&str],
    title: &str,
    ext: &str,
    track_number: u32,
    total_tracks: u32,
) -> PathBuf {
    let width = if total_tracks >= 1000 {
        4
    } else if total_tracks >= 100 {
        3
    } else {
        2
    };

    let safe_title = sanitize_filename(title);
    let track_prefix = format!("{:0width$}", track_number, width = width);

    let filename = if artists.is_empty() {
        format!("{} - {}.{}", track_prefix, safe_title, ext)
    } else {
        format!(
            "{} - {} - {}.{}",
            track_prefix,
            sanitize_filename(&artists.join(", ")),
            safe_title,
            ext
        )
    };

    target_root.join(filename)
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
