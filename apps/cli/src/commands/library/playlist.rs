use std::path::{Path, PathBuf};

use audiotags::Tag;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::api::models::PlaylistDto;
use crate::api::ApiClient;

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
pub async fn list(client: &ApiClient) -> anyhow::Result<()> {
    let playlists = client.get_playlists().await?;

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

    Ok(())
}

/// Download all local tracks from a playlist via the API (HTTP streaming).
pub async fn download(
    client: &ApiClient,
    id_or_name: &str,
    output: &Path,
    flat: bool,
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

    for (index, track) in tracks.iter().enumerate() {
        let track_number = (index + 1) as u32;
        let artist_names: Vec<_> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let display = format!("{} — {}", artist_names.join(", "), track.title);
        overall.set_message(display.clone());

        // Infer extension from server-side file_path when available, fallback to mp3.
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
                if let Err(err) = set_playlist_order_metadata(&dest, track_number, total as u32) {
                    overall.println(format!(
                        "  {} {} ({})",
                        style("warn").yellow(),
                        display,
                        err
                    ));
                }
            }
            Err(err) => {
                overall.println(format!(
                    "  {} {} ({})",
                    style("skip").yellow(),
                    display,
                    err
                ));
                skipped += 1;
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
        if skipped > 0 {
            format!(" ({} skipped)", style(skipped).yellow())
        } else {
            String::new()
        }
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
