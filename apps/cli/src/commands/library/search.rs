use anyhow::Context;
use console::style;

use crate::api::models::{AlbumDto, ArtistDto, PlaylistDto, TrackDto};
use crate::api::ApiClient;
use crate::{OutputFormat, SearchEntity};

pub async fn search(
    client: &ApiClient,
    entity: SearchEntity,
    query: Option<&str>,
    source: Option<&str>,
    genre: Option<&str>,
    needs_validation: bool,
    has_file: bool,
    limit: Option<usize>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    match entity {
        SearchEntity::Playlists => {
            let mut rows = client.get_playlists().await?;
            apply_playlist_filters(&mut rows, query, source);
            apply_limit(&mut rows, limit);
            print_playlists(&rows, format)?;
        }
        SearchEntity::Artists => {
            let mut rows = client.get_artists().await?;
            apply_artist_filters(&mut rows, query);
            apply_limit(&mut rows, limit);
            print_artists(&rows, format)?;
        }
        SearchEntity::Albums => {
            let mut rows = client.get_albums().await?;
            apply_album_filters(&mut rows, query);
            apply_limit(&mut rows, limit);
            print_albums(&rows, format)?;
        }
        SearchEntity::Tracks => {
            let mut rows = client.get_tracks().await?;
            apply_track_filters(&mut rows, query, genre, needs_validation, has_file);
            apply_limit(&mut rows, limit);
            print_tracks(&rows, format)?;
        }
    }

    Ok(())
}

fn apply_playlist_filters(rows: &mut Vec<PlaylistDto>, query: Option<&str>, source: Option<&str>) {
    if let Some(query) = query {
        let query = query.to_lowercase();
        rows.retain(|p| p.name.to_lowercase().contains(&query));
    }

    if let Some(source) = source {
        let source = source.to_lowercase();
        rows.retain(|p| p.source.to_lowercase().contains(&source));
    }
}

fn apply_artist_filters(rows: &mut Vec<ArtistDto>, query: Option<&str>) {
    if let Some(query) = query {
        let query = query.to_lowercase();
        rows.retain(|a| a.name.to_lowercase().contains(&query));
    }
}

fn apply_album_filters(rows: &mut Vec<AlbumDto>, query: Option<&str>) {
    if let Some(query) = query {
        let query = query.to_lowercase();
        rows.retain(|album| {
            album.title.to_lowercase().contains(&query)
                || album
                    .artists
                    .iter()
                    .any(|a| a.name.to_lowercase().contains(&query))
        });
    }
}

fn apply_track_filters(
    rows: &mut Vec<TrackDto>,
    query: Option<&str>,
    genre: Option<&str>,
    needs_validation: bool,
    has_file: bool,
) {
    if let Some(query) = query {
        let query = query.to_lowercase();
        rows.retain(|track| {
            track.title.to_lowercase().contains(&query)
                || track
                    .artists
                    .iter()
                    .any(|a| a.name.to_lowercase().contains(&query))
                || track
                    .album
                    .as_ref()
                    .map(|a| a.title.to_lowercase().contains(&query))
                    .unwrap_or(false)
        });
    }

    if let Some(genre) = genre {
        let genre = genre.to_lowercase();
        rows.retain(|track| {
            track
                .genre
                .as_deref()
                .map(|g| g.to_lowercase().contains(&genre))
                .unwrap_or(false)
        });
    }

    if needs_validation {
        rows.retain(|track| track.needs_validation);
    }

    if has_file {
        rows.retain(|track| track.file_path.is_some());
    }
}

fn apply_limit<T>(rows: &mut Vec<T>, limit: Option<usize>) {
    if let Some(limit) = limit {
        rows.truncate(limit);
    }
}

fn print_playlists(rows: &[PlaylistDto], format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
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

            for p in rows {
                println!(
                    "{:>4}  {:<40}  {}",
                    style(p.id).cyan(),
                    p.name,
                    style(&p.source).dim()
                );
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(rows).context("Failed to render JSON")?
            );
            Ok(())
        }
        OutputFormat::Jsonl => {
            for row in rows {
                println!(
                    "{}",
                    serde_json::to_string(row).context("Failed to render JSONL row")?
                );
            }
            Ok(())
        }
    }
}

fn print_artists(rows: &[ArtistDto], format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", style("No artists found.").yellow());
                return Ok(());
            }

            println!(
                "{:>4}  {}",
                style("ID").bold().dim(),
                style("Name").bold().dim()
            );
            println!("{}", style("─".repeat(48)).dim());
            for a in rows {
                println!("{:>4}  {}", style(a.id).cyan(), a.name);
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(rows).context("Failed to render JSON")?
            );
            Ok(())
        }
        OutputFormat::Jsonl => {
            for row in rows {
                println!(
                    "{}",
                    serde_json::to_string(row).context("Failed to render JSONL row")?
                );
            }
            Ok(())
        }
    }
}

fn print_albums(rows: &[AlbumDto], format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", style("No albums found.").yellow());
                return Ok(());
            }

            println!(
                "{:>4}  {:<38}  {}",
                style("ID").bold().dim(),
                style("Title").bold().dim(),
                style("Artists").bold().dim()
            );
            println!("{}", style("─".repeat(84)).dim());
            for album in rows {
                let artists = album
                    .artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "{:>4}  {:<38}  {}",
                    style(album.id).cyan(),
                    album.title,
                    artists
                );
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(rows).context("Failed to render JSON")?
            );
            Ok(())
        }
        OutputFormat::Jsonl => {
            for row in rows {
                println!(
                    "{}",
                    serde_json::to_string(row).context("Failed to render JSONL row")?
                );
            }
            Ok(())
        }
    }
}

fn print_tracks(rows: &[TrackDto], format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("{}", style("No tracks found.").yellow());
                return Ok(());
            }

            println!(
                "{:>4}  {:<42}  {:<24}  {}",
                style("ID").bold().dim(),
                style("Title").bold().dim(),
                style("Artists").bold().dim(),
                style("Genre").bold().dim()
            );
            println!("{}", style("─".repeat(108)).dim());
            for track in rows {
                let artists = track
                    .artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let genre = track.genre.as_deref().unwrap_or("-");
                println!(
                    "{:>4}  {:<42}  {:<24}  {}",
                    style(track.id).cyan(),
                    track.title,
                    artists,
                    genre
                );
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(rows).context("Failed to render JSON")?
            );
            Ok(())
        }
        OutputFormat::Jsonl => {
            for row in rows {
                println!(
                    "{}",
                    serde_json::to_string(row).context("Failed to render JSONL row")?
                );
            }
            Ok(())
        }
    }
}
