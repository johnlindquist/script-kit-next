//! Screenshot-to-file bridge for Tab AI harness context injection.
//!
//! Captures the focused window screenshot to a temporary PNG file so the
//! harness backend can read the image natively via file path — no base64
//! data is pasted into the PTY.

use anyhow::{Context, Result};

/// Maximum number of screenshot files to retain in the temp directory.
pub const TAB_AI_SCREENSHOT_MAX_KEEP: usize = 10;

/// Filename prefix used for Tab AI screenshot temp files.
const TAB_AI_SCREENSHOT_PREFIX: &str = "tab-ai-screenshot-";

/// Result of writing a focused window screenshot to a temp file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabAiScreenshotFile {
    /// Absolute path to the PNG file on disk.
    pub path: String,
    /// Width of the captured image in pixels.
    pub width: u32,
    /// Height of the captured image in pixels.
    pub height: u32,
    /// Title of the captured window.
    pub title: String,
    /// True if the capture fell back to the first available window
    /// (no focused window was detected).
    pub used_fallback: bool,
}

/// Capture a screenshot of the focused window and write it to a temp file.
///
/// Returns `Ok(None)` if no suitable window was found. Returns `Ok(Some(...))`
/// on success with the absolute path, dimensions, and metadata. Runs
/// [`cleanup_old_tab_ai_screenshot_files`] after a successful write.
pub fn capture_tab_ai_focused_window_screenshot_file() -> Result<Option<TabAiScreenshotFile>> {
    let capture = match crate::platform::capture_focused_window_screenshot() {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(
                event = "tab_ai_screenshot_capture_failed",
                error = %e,
            );
            return Ok(None);
        }
    };

    if capture.png_data.is_empty() {
        return Ok(None);
    }

    let tmp_dir = screenshot_tmp_dir()?;
    std::fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create screenshot tmp dir: {}", tmp_dir.display()))?;

    let filename = format!(
        "{}{}-{}.png",
        TAB_AI_SCREENSHOT_PREFIX,
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
        std::process::id(),
    );
    let file_path = tmp_dir.join(&filename);

    std::fs::write(&file_path, &capture.png_data)
        .with_context(|| format!("failed to write screenshot: {}", file_path.display()))?;

    let abs_path = file_path.to_string_lossy().into_owned();

    tracing::debug!(
        event = "tab_ai_screenshot_file_written",
        path = %abs_path,
        width = capture.width,
        height = capture.height,
        title = %capture.window_title,
        used_fallback = capture.used_fallback,
        bytes = capture.png_data.len(),
    );

    // Best-effort cleanup — don't fail the capture if cleanup errors
    if let Err(e) = cleanup_old_tab_ai_screenshot_files(TAB_AI_SCREENSHOT_MAX_KEEP) {
        tracing::warn!(
            event = "tab_ai_screenshot_cleanup_failed",
            error = %e,
        );
    }

    Ok(Some(TabAiScreenshotFile {
        path: abs_path,
        width: capture.width,
        height: capture.height,
        title: capture.window_title,
        used_fallback: capture.used_fallback,
    }))
}

/// Remove old Tab AI screenshot files, keeping at most `max_keep` newest files.
pub fn cleanup_old_tab_ai_screenshot_files(max_keep: usize) -> Result<()> {
    let tmp_dir = screenshot_tmp_dir()?;
    if !tmp_dir.exists() {
        return Ok(());
    }

    let mut screenshot_files: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();

    let entries = std::fs::read_dir(&tmp_dir)
        .with_context(|| format!("failed to read screenshot tmp dir: {}", tmp_dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with(TAB_AI_SCREENSHOT_PREFIX) && name_str.ends_with(".png") {
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            screenshot_files.push((entry.path(), modified));
        }
    }

    if screenshot_files.len() <= max_keep {
        return Ok(());
    }

    // Sort newest first
    screenshot_files.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove everything beyond max_keep
    for (path, _) in screenshot_files.iter().skip(max_keep) {
        if let Err(e) = std::fs::remove_file(path) {
            tracing::debug!(
                event = "tab_ai_screenshot_cleanup_file_failed",
                path = %path.display(),
                error = %e,
            );
        }
    }

    Ok(())
}

/// Return the directory for Tab AI screenshot temp files.
///
/// Uses `~/.scriptkit/tmp/` to keep temp files alongside other Script Kit data.
fn screenshot_tmp_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".scriptkit").join("tmp"))
}

/// Return the directory for Tab AI screenshot temp files, using a custom root.
///
/// This is exposed for tests that need to control the directory.
pub fn cleanup_old_tab_ai_screenshot_files_in_dir(
    dir: &std::path::Path,
    max_keep: usize,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut screenshot_files: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read screenshot dir: {}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with(TAB_AI_SCREENSHOT_PREFIX) && name_str.ends_with(".png") {
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            screenshot_files.push((entry.path(), modified));
        }
    }

    if screenshot_files.len() <= max_keep {
        return Ok(());
    }

    // Sort newest first
    screenshot_files.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove everything beyond max_keep
    for (path, _) in screenshot_files.iter().skip(max_keep) {
        if let Err(e) = std::fs::remove_file(path) {
            tracing::debug!(
                event = "tab_ai_screenshot_cleanup_file_failed",
                path = %path.display(),
                error = %e,
            );
        }
    }

    Ok(())
}

/// The screenshot filename prefix, exposed for tests.
pub fn tab_ai_screenshot_prefix() -> &'static str {
    TAB_AI_SCREENSHOT_PREFIX
}
