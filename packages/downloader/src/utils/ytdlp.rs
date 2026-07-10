use serde::Deserialize;
use serde_json::Value;
use shared::{errors::Error, http::ProxyRotator, types::SoundomeResult};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::{io::AsyncReadExt, process::Command};

/// Max attempts (including the first) for transient failures such as
/// rate limiting / bot-detection 403s from YouTube. These are known to be
/// intermittent: the same URL can fail on one run and succeed on the next
/// (see docs/operations/youtube-search-configuration.md).
const MAX_ATTEMPTS: u32 = 3;
const RETRY_BASE_DELAY: Duration = Duration::from_secs(2);

pub async fn download_with_ytdlp(
    url: &str,
    file_name: &str,
    base_library_dir: PathBuf,
) -> Result<PathBuf, Error> {
    let base_library_dir = base_library_dir
        .to_str()
        .ok_or(Error::InvalidPath(base_library_dir.clone()))?;
    let output_path = format!("{}/{}.%(ext)s", base_library_dir, file_name);

    let stdout = run_ytdlp_with_retry(|| build_download_args(url, &output_path)).await?;

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

    let stdout = run_ytdlp_with_retry(|| {
        let mut args = vec![
            search_spec.clone(),
            "--dump-json".to_string(),
            "--skip-download".to_string(),
            "--flat-playlist".to_string(),
        ];
        append_proxy_arg(&mut args);
        args
    })
    .await?;

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

/// Runs `yt-dlp` via `run_ytdlp`, retrying on transient failures (rate
/// limiting / bot-detection 403s) with a short backoff.
///
/// `build_args` is called fresh on every attempt so that a rotating proxy
/// (see `ProxyRotator`) can pick a different upstream IP on retry instead of
/// repeating the same request that just got rate-limited.
async fn run_ytdlp_with_retry<F>(mut build_args: F) -> SoundomeResult<Vec<u8>>
where
    F: FnMut() -> Vec<String>,
{
    let mut attempt = 1;
    loop {
        let args = build_args();
        tracing::info!("Running yt-dlp with args (attempt {}): {:?}", attempt, args);

        match run_ytdlp(&args).await {
            Ok(stdout) => return Ok(stdout),
            Err(Error::ExitCode { code, stderr })
                if attempt < MAX_ATTEMPTS && is_transient_error(&stderr) =>
            {
                let delay = RETRY_BASE_DELAY * attempt;
                tracing::warn!(
                    "yt-dlp failed with a transient error (exit code {}), retrying in {:?} (attempt {}/{}): {}",
                    code,
                    delay,
                    attempt,
                    MAX_ATTEMPTS,
                    stderr.trim()
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

/// Heuristic: does this yt-dlp stderr look like a transient rate-limit / bot
/// detection failure rather than a permanent one (removed video, DRM, region
/// lock, etc.)? These are known to be intermittent for the exact same URL
/// (see docs/operations/youtube-search-configuration.md).
fn is_transient_error(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("403")
        || lower.contains("429")
        || lower.contains("too many requests")
        || lower.contains("rate-limit")
        || lower.contains("rate limit")
        || lower.contains("sign in to confirm")
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
