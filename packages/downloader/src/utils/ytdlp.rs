use serde::Deserialize;
use serde_json::Value;
use shared::{errors::Error, http::ProxyRotator, types::SoundomeResult};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

pub async fn download_with_ytdlp(
    url: &str,
    file_name: &str,
    base_library_dir: PathBuf,
) -> Result<PathBuf, Error> {
    let base_library_dir = base_library_dir
        .to_str()
        .ok_or(Error::InvalidPath(base_library_dir.clone()))?;
    let output_path = format!("{}/{}.%(ext)s", base_library_dir, file_name);

    let args = build_download_args(url, &output_path);
    tracing::info!("Running yt-dlp with args: {:?}", args);
    let stdout = run_ytdlp(&args).await?;

    // Parse JSON output
    let value: Value = serde_json::from_slice(&stdout)?;
    let path = value["_filename"]
        .as_str()
        .ok_or(Error::NotFound("downloaded file path".to_string()))?;

    // Replace extension with .mp3
    let final_path = PathBuf::from(match path.rsplit_once('.') {
        Some((base, _)) => format!("{}.mp3", base),
        None => format!("{}.mp3", path),
    });

    Ok(final_path)
}

fn build_download_args(url: &str, output_path: &str) -> Vec<String> {
    // default args
    let mut args = vec![
        url.to_string(),
        "--print-json".to_string(),
        "-f".to_string(),
        "bestaudio".to_string(),
        "--extract-audio".to_string(),
        "--audio-format".to_string(),
        "mp3".to_string(),
        "--audio-quality".to_string(),
        "0".to_string(),
        "--embed-thumbnail".to_string(),
        "--output".to_string(),
        output_path.to_string(),
    ];

    append_proxy_arg(&mut args);

    args
}

/// Minimal shape of a single JSON object emitted by
/// `yt-dlp "ytsearchN:<query>" --dump-json --skip-download --flat-playlist`.
/// yt-dlp emits many more fields (thumbnails, view_count, description, ...);
/// only the ones Soundome actually needs are modeled here, the rest are
/// ignored by serde.
#[derive(Debug, Deserialize)]
struct YtDlpSearchEntry {
    id: String,
    title: String,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    uploader: Option<String>,
    /// Seconds, as a float. Absent for some live streams/premieres.
    #[serde(default)]
    duration: Option<f64>,
}

/// A single YouTube search result, already narrowed down to what Soundome needs.
#[derive(Debug, Clone)]
pub struct YtDlpSearchResult {
    pub id: String,
    pub title: String,
    pub author: String,
    /// Duration in whole seconds, when yt-dlp reports one.
    pub duration: Option<i32>,
}

/// Search YouTube via yt-dlp's `ytsearchN:` pseudo-URL and return up to `limit`
/// results, without downloading anything (`--skip-download --flat-playlist`
/// keeps this to a single, fast metadata-only request per search).
///
/// This replaces the previous Invidious-based search: yt-dlp talks to YouTube
/// directly (through the shared proxy when configured), so there is no more
/// third-party instance to select or fall back to.
pub async fn search_with_ytdlp(
    query: &str,
    limit: usize,
) -> SoundomeResult<Vec<YtDlpSearchResult>> {
    let search_spec = format!("ytsearch{}:{}", limit, query);

    let mut args = vec![
        search_spec,
        "--dump-json".to_string(),
        "--skip-download".to_string(),
        "--flat-playlist".to_string(),
    ];
    append_proxy_arg(&mut args);

    tracing::info!("Running yt-dlp search with args: {:?}", args);
    let stdout = run_ytdlp(&args).await?;

    // In `--dump-json --flat-playlist` mode yt-dlp prints one JSON object per
    // line (one per search result), not a single JSON document/array.
    let results = String::from_utf8_lossy(&stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(
            |line| match serde_json::from_str::<YtDlpSearchEntry>(line) {
                Ok(entry) => Some(entry),
                Err(err) => {
                    tracing::warn!("Skipping unparsable yt-dlp search result: {}", err);
                    None
                }
            },
        )
        .map(|entry| YtDlpSearchResult {
            id: entry.id,
            title: entry.title,
            author: entry.channel.or(entry.uploader).unwrap_or_default(),
            duration: entry.duration.map(|d| d.round() as i32),
        })
        .collect();

    Ok(results)
}

/// Spawn `yt-dlp` with the given args and return its captured stdout.
/// Maps a non-zero exit code to `Error::ExitCode` carrying the captured stderr.
async fn run_ytdlp(args: &[String]) -> SoundomeResult<Vec<u8>> {
    let mut child = Command::new("yt-dlp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()?;

    // Read stdout asynchronously to prevent buffer overflow
    let mut stdout = Vec::new();
    if let Some(mut child_stdout) = child.stdout.take() {
        tokio::io::copy(&mut child_stdout, &mut stdout).await?;
    }

    // TODO: Implement timeout handling
    let exit_code = child.wait().await?;

    if !exit_code.success() {
        let mut stderr = Vec::new();
        if let Some(mut reader) = child.stderr {
            reader.read_to_end(&mut stderr).await?;
        }
        return Err(Error::ExitCode {
            code: exit_code.code().unwrap_or(1),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        });
    }

    Ok(stdout)
}

/// Append `--proxy <url>` when a proxy is configured via `Config.proxy`.
fn append_proxy_arg(args: &mut Vec<String>) {
    if let Some(proxy_url) = ProxyRotator::get().get_next_proxy() {
        tracing::info!("Using proxy for yt-dlp: {}", proxy_url);
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }
}
