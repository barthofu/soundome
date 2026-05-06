use serde_json::Value;
use shared::{errors::Error, types::SoundomeResult};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

pub async fn _download_with_scdl(url: &str, base_library_dir: PathBuf) -> SoundomeResult<PathBuf> {
    let output_path = format!(
        "{}",
        base_library_dir
            .to_str()
            .ok_or(Error::InvalidPath(base_library_dir.clone()))?
    );

    let mut child = Command::new("scdl")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(&["-l", url, "--path", &output_path])
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
