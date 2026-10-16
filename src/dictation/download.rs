//! Dictation model downloads.
//!
//! Downloads single-file GGUF speech models into the Script Kit models
//! directory. Transfers resume via HTTP Range, every file is checked against
//! its published SHA-256 before it is moved into place, and progress is
//! reported via a callback.

use crate::dictation::transcription::{
    resolve_default_model_path, resolve_whisper_model_path, PARAKEET_MODEL_SHA256,
    PARAKEET_MODEL_SIZE, PARAKEET_MODEL_URL, WHISPER_MODEL_SHA256, WHISPER_MODEL_SIZE,
    WHISPER_MODEL_URL,
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
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
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

/// Classify a download error into a user-friendly, actionable message.
///
/// Maps common failure modes (network, disk space, permissions, server
/// errors) to short guidance strings.  Unknown errors pass through
/// verbatim so no information is lost.
pub fn classify_download_error(error: &anyhow::Error) -> String {
    let text = error.to_string();
    let lower = text.to_ascii_lowercase();

    if lower.contains("http request failed")
        || lower.contains("dns")
        || lower.contains("timed out")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("connection aborted")
        || lower.contains("network")
    {
        return "Network issue while downloading the dictation model. Check your connection and retry.".to_string();
    }

    if lower.contains("no space")
        || lower.contains("disk full")
        || lower.contains("failed to write to partial file")
    {
        return "Not enough disk space to store the dictation model. Free some space and retry."
            .to_string();
    }

    if lower.contains("permission denied")
        || lower.contains("failed to create models directory")
        || lower.contains("failed to create partial file")
    {
        return "Script Kit could not write the dictation model to disk. Check folder permissions and retry.".to_string();
    }

    if lower.contains("unexpected http status") {
        return format!("The dictation model server returned an unexpected response: {text}");
    }

    text
}

/// Download phase reported to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadPhase {
    /// HTTP download in progress.
    Downloading,
    /// Checking the downloaded file against its published SHA-256.
    Verifying,
    /// Download and verification complete.
    Complete,
    /// Download was cancelled by the caller.
    Cancelled,
    /// An error occurred.
    Failed(String),
}

/// A single-file model download: where it comes from, where it goes, and
/// how to check it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFileSpec {
    pub url: &'static str,
    pub size: u64,
    /// Lowercase hex SHA-256 of the complete file.
    pub sha256: &'static str,
}

/// Download spec for the Parakeet model file.
pub const PARAKEET_MODEL_SPEC: ModelFileSpec = ModelFileSpec {
    url: PARAKEET_MODEL_URL,
    size: PARAKEET_MODEL_SIZE,
    sha256: PARAKEET_MODEL_SHA256,
};

/// Download spec for the Whisper Medium model file.
pub const WHISPER_MODEL_SPEC: ModelFileSpec = ModelFileSpec {
    url: WHISPER_MODEL_URL,
    size: WHISPER_MODEL_SIZE,
    sha256: WHISPER_MODEL_SHA256,
};

/// Download the Parakeet model file to the models directory.
pub fn download_parakeet_model(
    on_progress: impl Fn(DownloadPhase, DownloadProgress) + Send,
    cancel: Arc<AtomicBool>,
) -> Result<PathBuf> {
    download_model_file(
        &PARAKEET_MODEL_SPEC,
        &resolve_default_model_path(),
        on_progress,
        cancel,
    )
}

/// Download the Whisper Medium model file to the models directory.
pub fn download_whisper_model(
    on_progress: impl Fn(DownloadPhase, DownloadProgress) + Send,
    cancel: Arc<AtomicBool>,
) -> Result<PathBuf> {
    download_model_file(
        &WHISPER_MODEL_SPEC,
        &resolve_whisper_model_path(),
        on_progress,
        cancel,
    )
}

/// Path of the resumable partial download that sits beside `model_path`.
pub fn partial_download_path(model_path: &Path) -> PathBuf {
    let mut name = model_path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(".partial");
    model_path.with_file_name(name)
}

/// Size of the resumable partial download for `model_path`, when present.
pub fn partial_download_size(model_path: &Path) -> Option<u64> {
    let len = std::fs::metadata(partial_download_path(model_path))
        .ok()?
        .len();
    (len > 0).then_some(len)
}

/// Download one model file: resumable transfer into a `.partial` sibling,
/// SHA-256 check, then an atomic rename into place.
///
/// A partial whose hash does not match is deleted so the next attempt starts
/// clean instead of resuming corrupt bytes.
pub fn download_model_file(
    spec: &ModelFileSpec,
    model_path: &Path,
    on_progress: impl Fn(DownloadPhase, DownloadProgress) + Send,
    cancel: Arc<AtomicBool>,
) -> Result<PathBuf> {
    let models_parent = model_path
        .parent()
        .context("cannot resolve models parent directory")?;
    std::fs::create_dir_all(models_parent).with_context(|| {
        format!(
            "failed to create models directory: {}",
            models_parent.display()
        )
    })?;

    let partial_path = partial_download_path(model_path);
    download_archive(spec.url, &partial_path, spec.size, &on_progress, &cancel)?;

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

    on_progress(
        DownloadPhase::Verifying,
        DownloadProgress {
            downloaded: spec.size,
            total: spec.size,
        },
    );
    let actual = sha256_file_hex(&partial_path)?;
    if !actual.eq_ignore_ascii_case(spec.sha256) {
        let _ = std::fs::remove_file(&partial_path);
        anyhow::bail!(
            "downloaded model failed its checksum (expected {}, got {actual}); the partial file was removed, try again",
            spec.sha256
        );
    }

    std::fs::rename(&partial_path, model_path).with_context(|| {
        format!(
            "failed to move downloaded model into place at {}",
            model_path.display()
        )
    })?;

    on_progress(
        DownloadPhase::Complete,
        DownloadProgress {
            downloaded: spec.size,
            total: spec.size,
        },
    );

    tracing::info!(
        category = "DICTATION",
        model_path = %model_path.display(),
        "Dictation model download complete and verified"
    );

    Ok(model_path.to_path_buf())
}

/// Lowercase hex SHA-256 of a file, streamed in 1 MiB blocks.
pub fn sha256_file_hex(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("failed to open {} for verification", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("failed to read {} for verification", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Download a file with resume support.
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
            "Dictation model file already fully downloaded"
        );
        return Ok(());
    }

    tracing::info!(
        category = "DICTATION",
        url,
        resume_from,
        expected_size,
        "Starting dictation model download"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_download_path_is_a_sibling_with_partial_suffix() {
        let path = Path::new("/models/parakeet-tdt-0.6b-v3-Q8_0.gguf");
        assert_eq!(
            partial_download_path(path),
            PathBuf::from("/models/parakeet-tdt-0.6b-v3-Q8_0.gguf.partial")
        );
    }

    #[test]
    fn sha256_file_hex_matches_known_digest() {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(file.path(), b"abc").expect("write");
        assert_eq!(
            sha256_file_hex(file.path()).expect("hash"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// Network check of the real download path: seeds a `.partial` with all
    /// but the last 4 MiB of a local copy of the Parakeet GGUF, then lets
    /// `download_model_file` resume over HTTP Range (through the host's
    /// redirect), verify the SHA-256 and rename into place.
    #[test]
    #[ignore = "network: set SK_DICTATION_SEED_GGUF to a local parakeet-tdt-0.6b-v3-Q8_0.gguf"]
    fn download_resumes_partial_and_verifies_checksum() {
        let Ok(seed) = std::env::var("SK_DICTATION_SEED_GGUF") else {
            eprintln!("skipping: SK_DICTATION_SEED_GGUF not set");
            return;
        };
        let dir = tempfile::TempDir::new().expect("temp dir");
        let model_path = dir.path().join("parakeet-tdt-0.6b-v3-Q8_0.gguf");
        let keep = PARAKEET_MODEL_SIZE - 4 * 1024 * 1024;
        {
            use std::io::Write as _;
            let mut src = std::fs::File::open(&seed).expect("open seed");
            let mut dst =
                std::fs::File::create(partial_download_path(&model_path)).expect("partial");
            let mut remaining = keep;
            let mut buf = vec![0u8; 1024 * 1024];
            while remaining > 0 {
                let want = remaining.min(buf.len() as u64) as usize;
                src.read_exact(&mut buf[..want]).expect("read seed");
                dst.write_all(&buf[..want]).expect("write partial");
                remaining -= want as u64;
            }
        }
        let phases = std::sync::Mutex::new(Vec::new());
        let first_progress = std::sync::Mutex::new(None);
        let path = download_model_file(
            &PARAKEET_MODEL_SPEC,
            &model_path,
            |phase, progress| {
                if phase == DownloadPhase::Downloading {
                    first_progress
                        .lock()
                        .expect("lock")
                        .get_or_insert(progress.downloaded);
                }
                let mut phases = phases.lock().expect("lock");
                if phases.last() != Some(&phase) {
                    phases.push(phase);
                }
            },
            Arc::new(AtomicBool::new(false)),
        )
        .expect("download");
        assert_eq!(path, model_path);
        assert!(model_path.is_file());
        assert!(!partial_download_path(&model_path).exists());
        let first = first_progress.lock().expect("lock").unwrap_or_default();
        assert!(
            first > keep,
            "download must resume from the partial, first progress {first}"
        );
        assert_eq!(
            *phases.lock().expect("lock"),
            vec![
                DownloadPhase::Downloading,
                DownloadPhase::Verifying,
                DownloadPhase::Complete
            ]
        );
    }

    #[test]
    fn checksum_mismatch_removes_partial() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let model_path = dir.path().join("tiny.gguf");
        std::fs::write(partial_download_path(&model_path), b"abc").expect("partial");
        let spec = ModelFileSpec {
            url: "https://invalid.example/never-fetched",
            size: 3,
            sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        };
        let error = download_model_file(
            &spec,
            &model_path,
            |_, _| {},
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("checksum must fail");
        assert!(error.to_string().contains("checksum"), "{error}");
        assert!(!partial_download_path(&model_path).exists());
        assert!(!model_path.exists());
    }

    #[test]
    fn model_specs_match_catalog_constants() {
        assert_eq!(PARAKEET_MODEL_SPEC.size, PARAKEET_MODEL_SIZE);
        assert_eq!(WHISPER_MODEL_SPEC.size, WHISPER_MODEL_SIZE);
        assert_ne!(PARAKEET_MODEL_SPEC.sha256, WHISPER_MODEL_SPEC.sha256);
    }

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
    fn classify_network_errors() {
        let err = anyhow::anyhow!("HTTP request failed: dns error");
        assert!(classify_download_error(&err).contains("Network issue"));

        let err = anyhow::anyhow!("connection timed out after 30s");
        assert!(classify_download_error(&err).contains("Network issue"));

        let err = anyhow::anyhow!("connection reset by peer");
        assert!(classify_download_error(&err).contains("Network issue"));

        let err = anyhow::anyhow!("connection refused");
        assert!(classify_download_error(&err).contains("Network issue"));

        let err = anyhow::anyhow!("connection aborted");
        assert!(classify_download_error(&err).contains("Network issue"));

        let err = anyhow::anyhow!("network unreachable");
        assert!(classify_download_error(&err).contains("Network issue"));
    }

    #[test]
    fn classify_disk_space_errors() {
        let err = anyhow::anyhow!("no space left on device");
        assert!(classify_download_error(&err).contains("disk space"));

        let err = anyhow::anyhow!("disk full");
        assert!(classify_download_error(&err).contains("disk space"));

        let err = anyhow::anyhow!("failed to write to partial file");
        assert!(classify_download_error(&err).contains("disk space"));
    }

    #[test]
    fn classify_permission_errors() {
        let err = anyhow::anyhow!("permission denied");
        assert!(classify_download_error(&err).contains("permissions"));

        let err = anyhow::anyhow!("failed to create models directory: /foo/bar");
        assert!(classify_download_error(&err).contains("permissions"));

        let err = anyhow::anyhow!("failed to create partial file: /foo/bar.partial");
        assert!(classify_download_error(&err).contains("permissions"));
    }

    #[test]
    fn classify_http_status_errors() {
        let err = anyhow::anyhow!("unexpected HTTP status 503 downloading model");
        let msg = classify_download_error(&err);
        assert!(msg.contains("unexpected response"));
        assert!(msg.contains("503"));
    }

    #[test]
    fn classify_unknown_errors_pass_through() {
        let err = anyhow::anyhow!("something completely unexpected happened");
        assert_eq!(
            classify_download_error(&err),
            "something completely unexpected happened"
        );
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
