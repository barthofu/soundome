//! Runtime-swappable `yt-dlp` binary provisioning.
//!
//! By default Soundome shells out to whatever `yt-dlp` is on `PATH` (the
//! version baked into the Docker image at build time). Because YouTube
//! frequently ships countermeasures that only get fixed in very recent
//! (sometimes nightly) yt-dlp releases, waiting for an image rebuild/release
//! to pick up a fix is too slow in practice.
//!
//! `[downloader.ytdlp].binary_url` lets an operator point Soundome at a
//! specific static yt-dlp binary release asset. At startup, Soundome:
//! - resolves a cache path derived from a hash of the URL, under
//!   `{database_dir}/bin/` (persisted across restarts via the existing data
//!   volume),
//! - reuses the cached file if it is already present (no network call),
//! - otherwise downloads it, optionally verifies a `sha256` checksum, marks
//!   it executable, and atomically installs it into the cache,
//! - falls back to the plain `"yt-dlp"` command (resolved via `PATH`) if
//!   anything above fails, logging the error rather than aborting boot.
//!
//! The resolved path is only computed once, at boot: bumping to a newer
//! build requires changing `binary_url` and restarting the process, not a
//! live hot-swap while running.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

use crate::{errors::Error, types::SoundomeResult};
use config::Config;

pub static GLOBAL_YTDLP_BINARY: OnceLock<String> = OnceLock::new();

const DEFAULT_BINARY: &str = "yt-dlp";

/// Resolves and, if needed, downloads the configured yt-dlp binary, then
/// stores the resulting command/path in the global slot returned by
/// [`path`]. Must be called once at boot, from an async context (it performs
/// network I/O). Never fails: on any error it logs and falls back to
/// `"yt-dlp"` from `PATH`.
pub async fn init() {
    let resolved = resolve().await.unwrap_or_else(|e| {
        tracing::error!(
            "Failed to provision custom yt-dlp binary, falling back to \"{}\" from PATH: {}",
            DEFAULT_BINARY,
            e
        );
        DEFAULT_BINARY.to_string()
    });

    tracing::info!("Using yt-dlp binary: {}", resolved);

    if GLOBAL_YTDLP_BINARY.set(resolved).is_err() {
        tracing::warn!("yt-dlp binary path was already initialized, ignoring second init() call");
    }
}

/// Returns the resolved yt-dlp binary path/command to use. Falls back to
/// `"yt-dlp"` if [`init`] was never called (e.g. in tests).
pub fn path() -> &'static str {
    GLOBAL_YTDLP_BINARY
        .get()
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_BINARY)
}

async fn resolve() -> SoundomeResult<String> {
    let ytdlp_config = &Config::get().downloader.ytdlp;

    let Some(binary_url) = ytdlp_config.binary_url.as_ref() else {
        return Ok(DEFAULT_BINARY.to_string());
    };

    if ytdlp_config.sha256.is_none() {
        tracing::warn!(
            "downloader.ytdlp.binary_url is set without a sha256 checksum; the download will not be verified"
        );
    }

    let cache_dir = bin_cache_dir()?;
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| Error::Custom(format!("failed to create yt-dlp cache dir: {}", e)))?;

    let cache_path = cache_dir.join(cache_file_name(binary_url));

    if cache_path.is_file() {
        match verify_executable(&cache_path).await {
            Ok(()) => {
                tracing::info!(
                    "Using cached custom yt-dlp binary at {} (url unchanged: {})",
                    cache_path.display(),
                    binary_url
                );
                return Ok(cache_path.to_string_lossy().into_owned());
            }
            Err(e) => {
                // The cached binary is present but can't actually run (e.g. it
                // was left over from a build with a different libc/arch, or a
                // previous crash left a corrupt file). Drop it and fall
                // through to a fresh download below instead of getting stuck
                // reusing a broken cache entry forever.
                tracing::warn!(
                    "Cached yt-dlp binary at {} failed a smoke test, re-downloading: {}",
                    cache_path.display(),
                    e
                );
                let _ = std::fs::remove_file(&cache_path);
            }
        }
    }

    tracing::info!(
        "Downloading custom yt-dlp binary from {} to {}",
        binary_url,
        cache_path.display()
    );

    let bytes = crate::libs::http::HttpClientBuilder::get_reqwest_client()?
        .get(binary_url)
        .send()
        .await
        .map_err(|e| Error::Network(format!("failed to download yt-dlp binary: {}", e)))?
        .error_for_status()
        .map_err(|e| Error::Network(format!("yt-dlp binary download returned an error: {}", e)))?
        .bytes()
        .await
        .map_err(|e| Error::Network(format!("failed to read yt-dlp binary response: {}", e)))?;

    if let Some(expected_sha256) = &ytdlp_config.sha256 {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual = hex::encode(hasher.finalize());

        if !actual.eq_ignore_ascii_case(expected_sha256) {
            return Err(Error::Custom(format!(
                "yt-dlp binary checksum mismatch: expected {}, got {}",
                expected_sha256, actual
            )));
        }
    }

    // Write to a temp file in the same directory then rename, so a crash
    // mid-download never leaves a corrupt file at the final cache path.
    let tmp_path = cache_dir.join(format!("{}.download", cache_file_name(binary_url)));
    std::fs::write(&tmp_path, &bytes)
        .map_err(|e| Error::Custom(format!("failed to write yt-dlp binary: {}", e)))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::Custom(format!("failed to chmod yt-dlp binary: {}", e)))?;
    }

    std::fs::rename(&tmp_path, &cache_path)
        .map_err(|e| Error::Custom(format!("failed to install yt-dlp binary: {}", e)))?;

    if let Err(e) = verify_executable(&cache_path).await {
        // Don't leave a known-broken binary around to be reused (and fail
        // the exact same way) on the next boot.
        let _ = std::fs::remove_file(&cache_path);
        return Err(e);
    }

    Ok(cache_path.to_string_lossy().into_owned())
}

/// Runs `<path> --version` as a smoke test to confirm the binary can
/// actually be executed on this system before relying on it, since a
/// download succeeding is not sufficient proof of that: a mismatched
/// architecture or libc (e.g. a glibc-linked "yt-dlp_linux" release asset on
/// this Alpine/musl-based image, instead of a "yt-dlp_musllinux" build)
/// produces a file that exists on disk but that the kernel refuses to exec,
/// which confusingly surfaces as `ENOENT` ("No such file or directory") the
/// first time something tries to spawn it — not at download time.
async fn verify_executable(path: &Path) -> SoundomeResult<()> {
    let output = tokio::process::Command::new(path)
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(Error::Custom(format!(
            "yt-dlp binary at {} exited with status {:?} when running --version: {}",
            path.display(),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::Custom(format!(
            "yt-dlp binary at {} could not be executed (\"No such file or directory\", but the \
             file exists): this usually means it was built for a different libc/architecture than \
             this container image — e.g. a glibc-linked \"yt-dlp_linux\" release asset used on an \
             Alpine/musl-based image instead of a \"yt-dlp_musllinux\"/\"yt-dlp_musllinux_aarch64\" \
             build. Original error: {}",
            path.display(),
            e
        ))),
        Err(e) => Err(Error::Custom(format!(
            "yt-dlp binary at {} could not be executed: {}",
            path.display(),
            e
        ))),
    }
}

/// Cache directory: a `bin/` subdirectory next to the SQLite database file,
/// so it lives on the same already-persisted volume without requiring any
/// extra mount configuration.
fn bin_cache_dir() -> SoundomeResult<PathBuf> {
    let db_url = &Config::get().database.url;
    let db_path = Path::new(db_url);
    let parent = db_path.parent().filter(|p| !p.as_os_str().is_empty());

    Ok(match parent {
        Some(parent) => parent.join("bin"),
        None => PathBuf::from("bin"),
    })
}

/// Derives a stable, filesystem-safe cache file name from the binary URL, so
/// that changing the URL (e.g. bumping to a newer nightly) results in a
/// fresh download, while restarts with the same URL reuse the cached file.
fn cache_file_name(binary_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(binary_url.as_bytes());
    let digest = hex::encode(hasher.finalize());
    format!("yt-dlp-{}", &digest[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_file_name_is_stable_for_same_url() {
        let a = cache_file_name("https://example.com/yt-dlp_musllinux");
        let b = cache_file_name("https://example.com/yt-dlp_musllinux");
        assert_eq!(a, b);
    }

    #[test]
    fn cache_file_name_differs_for_different_urls() {
        let a = cache_file_name("https://example.com/v1/yt-dlp_musllinux");
        let b = cache_file_name("https://example.com/v2/yt-dlp_musllinux");
        assert_ne!(a, b);
    }

    #[test]
    fn path_falls_back_to_default_when_uninitialized() {
        // Note: this only holds true if init() hasn't run in this process yet.
        // Safe in isolation; other tests in this file don't call init().
        if GLOBAL_YTDLP_BINARY.get().is_none() {
            assert_eq!(path(), DEFAULT_BINARY);
        }
    }
}
