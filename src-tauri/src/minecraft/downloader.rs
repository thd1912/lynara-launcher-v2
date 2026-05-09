//! Downloader — async file downloader with SHA-1 verification and progress reporting.
//!
//! Single primitive: `download_file(url, dest, expected_sha1, progress_tx)`.
//! Used by `installer.rs` to download libraries, client jar, asset index, and asset objects.

use futures_util::StreamExt;
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::minecraft::{manifest, McError};

/// HTTP client shared across the install process.
/// Built lazily on first use to avoid global state issues.
fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("LynaraLauncher/2.0")
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to build HTTP client")
}

/// Downloads a file, verifies SHA-1 if provided, reports incremental bytes via channel.
///
/// - If `dest` already exists with the right SHA-1, skips the download (returns Ok immediately).
/// - On SHA-1 mismatch after download, deletes the file and returns an error.
/// - Streams the response body directly to disk (low memory usage).
pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: Option<&str>,
    progress_tx: &mpsc::UnboundedSender<u64>,
) -> Result<(), McError> {
    // Skip if file already exists with valid SHA-1
    if let Some(sha1) = expected_sha1 {
        if dest.exists() && manifest::verify_file_sha1(dest, sha1) {
            // File already downloaded and valid — count its size as "done"
            if let Ok(meta) = std::fs::metadata(dest) {
                let _ = progress_tx.send(meta.len());
            }
            return Ok(());
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Download to a temp file, atomic move on success
    let tmp_path = dest.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp_path).await?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(McError::Network)?
        .error_for_status()
        .map_err(McError::Network)?;

    let mut hasher = Sha1::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(McError::Network)?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        let _ = progress_tx.send(chunk.len() as u64);
    }
    file.flush().await?;
    drop(file);

    // Verify SHA-1 if expected
    if let Some(expected) = expected_sha1 {
        let computed = hex::encode(hasher.finalize());
        if !computed.eq_ignore_ascii_case(expected) {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(McError::Custom(format!(
                "SHA-1 mismatch for {}: expected {}, got {}",
                url, expected, computed
            )));
        }
    }

    // Atomic move into place
    if dest.exists() {
        tokio::fs::remove_file(dest).await?;
    }
    tokio::fs::rename(&tmp_path, dest).await?;

    Ok(())
}

/// Build a fresh HTTP client (call once per install batch and reuse).
pub fn http_client() -> reqwest::Client {
    build_client()
}
