use serde_json::Value;
use shared::{errors::Error, http::ProxyRotator};
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

    let args = build_args(url, &output_path);
    tracing::info!("Running yt-dlp with args: {:?}", args);
    let mut child = Command::new("yt-dlp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(&args)
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

fn build_args(url: &str, output_path: &str) -> Vec<String> {
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

    // proxy
    if let Some(proxy_url) = ProxyRotator::get().get_next_proxy() {
        tracing::info!("Using proxy for yt-dlp: {}", proxy_url);
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }

    args
}
