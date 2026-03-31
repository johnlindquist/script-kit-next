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

/// Format a byte count as a human-readable string (e.g. "142.5 MB").
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

/// Format a download speed as a human-readable string (e.g. "12.3 MB/s").
pub fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return "-- MB/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Render a fixed-width textual progress bar.
///
/// Example: `format_progress_bar(50, 10)` -> `"█████░░░░░"`
pub fn format_progress_bar(percentage: u8, width: usize) -> String {
    let clamped = percentage.min(100) as usize;
    let filled = (clamped * width) / 100;
    let empty = width.saturating_sub(filled);
    format!(
        "{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty)
    )
}

/// Render the shared progress summary used by both the prompt and HUD.
///
/// Example output:
/// `[█████░░░░░] 50% · 256.0 MB/512.0 MB · 8.0 MB/s · ETA 32s`
pub fn format_progress_summary(
    percentage: u8,
    downloaded_bytes: u64,
    total_bytes: u64,
    speed_bytes_per_sec: u64,
    eta_seconds: Option<u64>,
) -> String {
    let bar = format_progress_bar(percentage, 10);
    let dl = format_bytes(downloaded_bytes);
    let total = format_bytes(total_bytes);
    let speed = format_speed(speed_bytes_per_sec);
    let eta = format_eta(eta_seconds);
    format!("[{bar}] {percentage}% · {dl}/{total} · {speed} · {eta}")
}

/// Estimate remaining time from total bytes and current speed.
///
/// Returns:
/// - `None` when total size is unknown or speed is zero
/// - `Some(0)` when the transfer is complete
/// - `Some(n)` for `n` seconds otherwise
pub fn estimate_eta_seconds(progress: DownloadProgress, bytes_per_sec: u64) -> Option<u64> {
    if progress.total == 0 || bytes_per_sec == 0 {
        return None;
    }
    if progress.downloaded >= progress.total {
        return Some(0);
    }
    let remaining = progress.total.saturating_sub(progress.downloaded);
    Some(remaining.div_ceil(bytes_per_sec))
}

/// Format ETA text for the download prompt and HUD.
///
/// Examples:
/// - `None` → `"ETA --"`
/// - `Some(0)` → `"ETA <1s"`
/// - `Some(15)` → `"ETA 15s"`
/// - `Some(75)` → `"ETA 1m 15s"`
/// - `Some(3672)` → `"ETA 1h 1m"`
pub fn format_eta(seconds: Option<u64>) -> String {
    let Some(seconds) = seconds else {
        return "ETA --".to_string();
    };
    if seconds == 0 {
        return "ETA <1s".to_string();
    }
    if seconds < 60 {
        return format!("ETA {seconds}s");
    }
    if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        return if secs == 0 {
            format!("ETA {minutes}m")
        } else {
            format!("ETA {minutes}m {secs}s")
        };
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if minutes == 0 {
        format!("ETA {hours}h")
    } else {
        format!("ETA {hours}h {minutes}m")
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
    fn format_progress_bar_half() {
        assert_eq!(format_progress_bar(50, 10), "█████░░░░░");
    }

    #[test]
    fn format_progress_bar_boundaries() {
        assert_eq!(format_progress_bar(0, 10), "░░░░░░░░░░");
        assert_eq!(format_progress_bar(100, 10), "██████████");
        // Clamp above 100
        assert_eq!(format_progress_bar(200, 10), "██████████");
        // Zero width
        assert_eq!(format_progress_bar(50, 0), "");
    }

    #[test]
    fn format_progress_summary_output() {
        let line = format_progress_summary(
            50,
            256 * 1024 * 1024,
            512 * 1024 * 1024,
            8 * 1024 * 1024,
            Some(32),
        );
        assert!(line.starts_with("[█████░░░░░] 50%"));
        assert!(line.contains("256.0 MB/512.0 MB"));
        assert!(line.contains("8.0 MB/s"));
        assert!(line.contains("ETA 32s"));
    }

    #[test]
    fn estimate_eta_complete_returns_zero() {
        let p = DownloadProgress {
            downloaded: 500,
            total: 500,
        };
        assert_eq!(estimate_eta_seconds(p, 100), Some(0));
    }

    #[test]
    fn estimate_eta_unknown_total_returns_none() {
        let p = DownloadProgress {
            downloaded: 100,
            total: 0,
        };
        assert_eq!(estimate_eta_seconds(p, 100), None);
    }

    #[test]
    fn estimate_eta_zero_speed_returns_none() {
        let p = DownloadProgress {
            downloaded: 100,
            total: 500,
        };
        assert_eq!(estimate_eta_seconds(p, 0), None);
    }

    #[test]
    fn estimate_eta_in_progress() {
        let p = DownloadProgress {
            downloaded: 250,
            total: 500,
        };
        // 250 remaining / 50 bps = 5 seconds
        assert_eq!(estimate_eta_seconds(p, 50), Some(5));
    }

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
