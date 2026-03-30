//! Parakeet model download and extraction.
//!
//! Downloads the Parakeet ONNX model archive from a remote server and
//! extracts it to the Script Kit models directory.  Supports resumable
//! downloads and reports progress via a callback.

use crate::dictation::transcription::{
    resolve_default_model_path, PARAKEET_MODEL_ARCHIVE_SIZE, PARAKEET_MODEL_URL,
};
use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Progress information emitted during model download.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Total expected size in bytes (may be 0 if unknown).
    pub total: u64,
}

impl DownloadProgress {
    /// Percentage complete (0–100), or 0 when total is unknown.
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        ((self.downloaded as f64 / self.total as f64) * 100.0).min(100.0) as u8
    }
}

/// Download phase reported to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadPhase {
    /// HTTP download in progress.
    Downloading,
    /// Archive extraction in progress.
    Extracting,
    /// Download and extraction complete.
    Complete,
    /// Download was cancelled by the caller.
    Cancelled,
    /// An error occurred.
    Failed(String),
}

/// Download and extract the Parakeet model to the default models directory.
///
/// - `on_progress` is called periodically with download progress.
/// - `cancel` can be set to `true` to abort the download.
///
/// Returns the path to the extracted model directory on success.
pub fn download_parakeet_model(
    on_progress: impl Fn(DownloadPhase, DownloadProgress) + Send,
    cancel: Arc<AtomicBool>,
) -> Result<PathBuf> {
    let model_dir = resolve_default_model_path();
    let models_parent = model_dir
        .parent()
        .context("cannot resolve models parent directory")?;

    // Ensure models directory exists.
    std::fs::create_dir_all(models_parent).with_context(|| {
        format!(
            "failed to create models directory: {}",
            models_parent.display()
        )
    })?;

    let partial_path = models_parent.join("parakeet-v3-int8.tar.gz.partial");
    let extracting_path = models_parent.join("parakeet-tdt-0.6b-v3-int8.extracting");

    // Clean up stale extraction directory from a previous interrupted run.
    if extracting_path.exists() {
        let _ = std::fs::remove_dir_all(&extracting_path);
    }

    // Phase 1: Download
    download_archive(
        PARAKEET_MODEL_URL,
        &partial_path,
        PARAKEET_MODEL_ARCHIVE_SIZE,
        &on_progress,
        &cancel,
    )?;

    if cancel.load(Ordering::Relaxed) {
        on_progress(
            DownloadPhase::Cancelled,
            DownloadProgress {
                downloaded: 0,
                total: 0,
            },
        );
        return Err(anyhow::anyhow!("model download cancelled"));
    }

    // Phase 2: Extract
    on_progress(
        DownloadPhase::Extracting,
        DownloadProgress {
            downloaded: PARAKEET_MODEL_ARCHIVE_SIZE,
            total: PARAKEET_MODEL_ARCHIVE_SIZE,
        },
    );

    extract_tar_gz(&partial_path, &extracting_path, &model_dir)?;

    // Clean up the archive.
    let _ = std::fs::remove_file(&partial_path);

    on_progress(
        DownloadPhase::Complete,
        DownloadProgress {
            downloaded: PARAKEET_MODEL_ARCHIVE_SIZE,
            total: PARAKEET_MODEL_ARCHIVE_SIZE,
        },
    );

    tracing::info!(
        category = "DICTATION",
        model_dir = %model_dir.display(),
        "Parakeet model download and extraction complete"
    );

    Ok(model_dir)
}

/// Download the archive file with resume support.
fn download_archive(
    url: &str,
    partial_path: &Path,
    expected_size: u64,
    on_progress: &impl Fn(DownloadPhase, DownloadProgress),
    cancel: &Arc<AtomicBool>,
) -> Result<()> {
    let resume_from = if partial_path.exists() {
        std::fs::metadata(partial_path)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    // If already fully downloaded, skip.
    if resume_from >= expected_size && expected_size > 0 {
        tracing::info!(
            category = "DICTATION",
            "Parakeet model archive already fully downloaded"
        );
        return Ok(());
    }

    tracing::info!(
        category = "DICTATION",
        url,
        resume_from,
        expected_size,
        "Starting Parakeet model download"
    );

    let agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent();

    let mut request = agent.get(url);
    if resume_from > 0 {
        request = request.header("Range", &format!("bytes={resume_from}-"));
    }

    let response = request.call().context("HTTP request failed")?;
    let status = response.status().as_u16();

    // If server doesn't support range requests, restart.
    let actual_resume = if resume_from > 0 && status == 200 {
        tracing::warn!(
            category = "DICTATION",
            "Server returned 200 instead of 206, restarting download"
        );
        0
    } else if status == 206 {
        resume_from
    } else if status == 200 {
        0
    } else {
        anyhow::bail!("unexpected HTTP status {status} downloading model");
    };

    let mut file = if actual_resume > 0 {
        std::fs::OpenOptions::new()
            .append(true)
            .open(partial_path)
            .with_context(|| format!("failed to open partial file: {}", partial_path.display()))?
    } else {
        std::fs::File::create(partial_path)
            .with_context(|| format!("failed to create partial file: {}", partial_path.display()))?
    };

    let mut reader = response.into_body().into_reader();
    let mut downloaded = actual_resume;
    let mut buf = vec![0u8; 64 * 1024]; // 64 KB chunks

    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let n = reader
            .read(&mut buf)
            .context("failed to read from HTTP response")?;
        if n == 0 {
            break;
        }

        std::io::Write::write_all(&mut file, &buf[..n])
            .context("failed to write to partial file")?;

        downloaded += n as u64;

        on_progress(
            DownloadPhase::Downloading,
            DownloadProgress {
                downloaded,
                total: expected_size,
            },
        );
    }

    // Validate final size.
    if expected_size > 0 && downloaded != expected_size {
        anyhow::bail!("download size mismatch: got {downloaded} bytes, expected {expected_size}");
    }

    Ok(())
}

/// Extract a `.tar.gz` archive, promoting a single inner directory to the
/// final model path (matching the vercel-voice extraction behavior).
fn extract_tar_gz(archive_path: &Path, extracting_dir: &Path, final_dir: &Path) -> Result<()> {
    tracing::info!(
        category = "DICTATION",
        archive = %archive_path.display(),
        dest = %final_dir.display(),
        "Extracting Parakeet model archive"
    );

    std::fs::create_dir_all(extracting_dir).with_context(|| {
        format!(
            "failed to create extraction directory: {}",
            extracting_dir.display()
        )
    })?;

    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("failed to open archive: {}", archive_path.display()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(extracting_dir)
        .with_context(|| format!("failed to extract archive to {}", extracting_dir.display()))?;

    // Check if the archive contained a single top-level directory.
    // If so, promote it to the final path (e.g. archive contains
    // `parakeet-tdt-0.6b-v3-int8/` → move that to `final_dir`).
    let entries: Vec<_> = std::fs::read_dir(extracting_dir)
        .context("failed to read extraction directory")?
        .filter_map(|e| e.ok())
        .collect();

    if entries.len() == 1 && entries[0].path().is_dir() {
        let inner = entries[0].path();
        // Remove any stale final_dir first.
        if final_dir.exists() {
            std::fs::remove_dir_all(final_dir).with_context(|| {
                format!("failed to remove stale model dir: {}", final_dir.display())
            })?;
        }
        std::fs::rename(&inner, final_dir).with_context(|| {
            format!(
                "failed to rename {} to {}",
                inner.display(),
                final_dir.display()
            )
        })?;
    } else {
        // Multiple entries — rename the whole extraction dir.
        if final_dir.exists() {
            std::fs::remove_dir_all(final_dir).with_context(|| {
                format!("failed to remove stale model dir: {}", final_dir.display())
            })?;
        }
        std::fs::rename(extracting_dir, final_dir).with_context(|| {
            format!(
                "failed to rename {} to {}",
                extracting_dir.display(),
                final_dir.display()
            )
        })?;
    }

    // Clean up extraction dir if it still exists (single-dir case).
    if extracting_dir.exists() {
        let _ = std::fs::remove_dir_all(extracting_dir);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_progress_percentage() {
        let p = DownloadProgress {
            downloaded: 250,
            total: 500,
        };
        assert_eq!(p.percentage(), 50);

        let p = DownloadProgress {
            downloaded: 500,
            total: 500,
        };
        assert_eq!(p.percentage(), 100);

        let p = DownloadProgress {
            downloaded: 0,
            total: 0,
        };
        assert_eq!(p.percentage(), 0);
    }
}
