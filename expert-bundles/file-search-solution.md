# File Search Solution Expert Bundle

## Original Goal

> Create an expert bundle of 45k of our current file search solution
>
> The file search feature uses macOS Spotlight (mdfind) for searching files system-wide.
> Currently experiencing issues with mdfind hanging indefinitely for queries that don't match anything.
> User wants to explore replacing mdfind with faster streaming solutions like `fd` or the `ignore` crate.

## Executive Summary

The Script Kit GPUI file search feature provides system-wide file searching using macOS Spotlight (`mdfind`). The implementation includes directory browsing mode (when path-like queries are typed), fuzzy filtering with Nucleo, and a 50/50 split UI with file list and preview.

**Recent Problem:** mdfind can hang indefinitely for queries that don't match anything (e.g., "nanobananal", "mp4" in some cases). A 5-second timeout was added as a band-aid fix, but this creates poor UX where users wait 5 seconds before seeing "No files found".

### Key Components:
1. **Core search module** (`src/file_search.rs`): Contains `search_files()` using mdfind with timeout, `search_with_timeout()` helper, `list_directory()` for browsing, Nucleo-based filtering (`filter_results_nucleo_simple`), and file metadata utilities.
2. **Entry point** (`src/app_execute.rs`): `open_file_search()` initializes the view and triggers initial search.
3. **Input handling** (`src/app_impl.rs`): Debounced search triggering (200ms), directory path detection via `parse_directory_path()`, Tab/Shift+Tab for directory navigation.
4. **UI rendering** (`src/render_builtins.rs`): `render_file_search()` creates the 50/50 split view with virtualized uniform_list.
5. **Built-in registration** (`src/builtins.rs`): Defines the "Search Files" built-in entry with `BuiltInFeature::FileSearch`.

### Current Architecture Issues:
1. **mdfind can hang** - Spotlight search can take 5+ seconds or hang indefinitely for no-match queries.
2. **No streaming** - Results only appear after search completes; no progressive loading as results are found.
3. **5s timeout is too long** - Users see "Searching..." for 5 seconds before "No files found".
4. **Blocking I/O** - The current implementation blocks in a thread while reading from mdfind stdout.

### Potential Improvements:
1. **Replace mdfind with `fd` command** (23ms vs 5s+, already installed, streams results naturally, `--max-results` for early termination).
2. **Use `ignore` crate** (powers ripgrep) for Rust-native parallel directory traversal with .gitignore support.
3. **Implement proper streaming** with channel-based result delivery so UI updates as results arrive.
4. **Hybrid approach**: Use fd for speed, fall back to mdfind only for Spotlight metadata searches.

### Files Included:
- `src/file_search.rs` (FULL): Core search logic, mdfind wrapper with timeout, directory listing, Nucleo filtering, file utilities
- `src/app_execute.rs` (excerpts): Entry point `open_file_search()`, built-in feature handler
- `src/app_impl.rs` (excerpts): Input handling, debouncing, Tab/arrow navigation, actions popup
- `src/render_builtins.rs` (excerpts): UI rendering for file search view (list, preview, states)
- `src/builtins.rs` (excerpts): Built-in definitions, FileSearch registration
- `src/main.rs` (excerpts): AppView enum, state fields

---

# Core File Search Module (Complete)

This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 1
</notes>
</file_summary>

<directory_structure>
src/file_search.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/file_search.rs">
//! File Search Module using macOS Spotlight (mdfind)
//!
//! This module provides file search functionality using macOS's mdfind command,
//! which interfaces with the Spotlight index for fast file searching.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, UNIX_EPOCH};
use tracing::{debug, instrument, warn};

/// Timeout for mdfind searches. Spotlight can hang indefinitely for queries
/// that don't match anything, so we need to timeout and return empty results.
const MDFIND_TIMEOUT: Duration = Duration::from_secs(5);

/// File type classification based on extension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    File,
    Directory,
    Application,
    Image,
    Document,
    Audio,
    Video,
    #[default]
    Other,
}

/// Information about a file for the actions dialog
/// Used as context for file-specific actions (similar to PathInfo and ScriptInfo)
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Full path to the file
    pub path: String,
    /// File name (last component of path)
    pub name: String,
    /// Type of file (used by the actions builder for context-specific actions)
    #[allow(dead_code)]
    pub file_type: FileType,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl FileInfo {
    /// Create FileInfo from a FileResult
    #[allow(dead_code)]
    pub fn from_result(result: &FileResult) -> Self {
        FileInfo {
            path: result.path.clone(),
            name: result.name.clone(),
            file_type: result.file_type,
            is_dir: result.file_type == FileType::Directory,
        }
    }

    /// Create FileInfo from path string
    #[allow(dead_code)]
    pub fn from_path(path: &str) -> Self {
        let path_obj = std::path::Path::new(path);
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let is_dir = path_obj.is_dir();
        let file_type = if is_dir {
            FileType::Directory
        } else {
            FileType::File
        };

        FileInfo {
            path: path.to_string(),
            name,
            file_type,
            is_dir,
        }
    }
}

/// Result of a file search
#[derive(Debug, Clone)]
pub struct FileResult {
    /// Full path to the file
    pub path: String,
    /// File name (last component of path)
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time as Unix timestamp
    pub modified: u64,
    /// Type of file
    pub file_type: FileType,
}

/// Metadata for a single file
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileMetadata {
    /// Full path to the file
    pub path: String,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time as Unix timestamp
    pub modified: u64,
    /// Type of file
    pub file_type: FileType,
    /// Whether the file is readable
    pub readable: bool,
    /// Whether the file is writable
    pub writable: bool,
}

/// Default limit for UI display (final visible results after filtering)
#[allow(dead_code)]
pub const DEFAULT_LIMIT: usize = 50;

/// Limit for interactive mdfind searches
/// Smaller than directory listing because each result requires a stat() call
/// 500 results is plenty for fuzzy filtering and keeps response time <1s
pub const DEFAULT_SEARCH_LIMIT: usize = 500;

/// Default cache limit for directory listing (fast operation, can handle more)
/// Directory listing is cheaper than mdfind search (single readdir vs many stat calls)
pub const DEFAULT_CACHE_LIMIT: usize = 2000;

/// Check if the query looks like an advanced mdfind query (with operators)
/// If so, pass it through directly; otherwise wrap as filename query
fn looks_like_advanced_mdquery(q: &str) -> bool {
    let q = q.trim();
    q.contains("kMDItem")
        || q.contains("==")
        || q.contains("!=")
        || q.contains(">=")
        || q.contains("<=")
        || q.contains("&&")
        || q.contains("||")
}

/// Escape special characters for mdfind query string literals
fn escape_md_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Build an mdfind query from user input
/// - If input looks like advanced query syntax, pass through as-is
/// - Otherwise, wrap as case-insensitive filename contains query
fn build_mdquery(user_query: &str) -> String {
    let q = user_query.trim();
    if looks_like_advanced_mdquery(q) {
        return q.to_string();
    }
    let escaped = escape_md_string(q);
    format!(r#"kMDItemFSName == "*{}*"c"#, escaped)
}

// NOTE: escape_query() was removed because:
// 1. It was unused dead code
// 2. Command::new() does NOT use a shell, so shell escaping is irrelevant
// 3. Arguments passed via .arg() are automatically handled safely

/// Detect file type based on extension
fn detect_file_type(path: &Path) -> FileType {
    // Get extension first - we need it to check for .app bundles
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // macOS .app bundles are directories but should be classified as Applications
    // Check for .app extension BEFORE checking is_dir()
    if extension.as_deref() == Some("app") {
        return FileType::Application;
    }

    // Check if it's a directory (but not an .app bundle)
    if path.is_dir() {
        return FileType::Directory;
    }

    match extension.as_deref() {
        // Applications (already handled above, but kept for completeness)
        Some("app") => FileType::Application,

        // Images
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "heic"
            | "heif",
        ) => FileType::Image,

        // Documents
        Some(
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt"
            | "ods" | "odp" | "pages" | "numbers" | "key",
        ) => FileType::Document,

        // Audio
        Some("mp3" | "wav" | "aac" | "flac" | "ogg" | "wma" | "m4a" | "aiff") => FileType::Audio,

        // Video
        Some("mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg") => {
            FileType::Video
        }

        // Check if it's a file (has extension but not matched above)
        Some(_) => FileType::File,

        // No extension - check if it exists to determine type
        None => {
            if path.exists() {
                if path.is_dir() {
                    FileType::Directory
                } else {
                    FileType::File
                }
            } else {
                FileType::Other
            }
        }
    }
}

/// Search for files using macOS mdfind (Spotlight)
///
/// Uses streaming to avoid buffering all results when only `limit` are needed.
/// Converts simple queries to filename-matching mdfind queries.
///
/// # Arguments
/// * `query` - Search query string (will be converted to filename query if simple)
/// * `onlyin` - Optional directory to limit search scope
/// * `limit` - Maximum number of results to return
///
/// # Returns
/// Vector of FileResult structs containing file information
///
#[instrument(skip_all, fields(query = %query, onlyin = ?onlyin, limit = limit))]
pub fn search_files(query: &str, onlyin: Option<&str>, limit: usize) -> Vec<FileResult> {
    debug!("Starting mdfind search");

    if query.is_empty() {
        debug!("Empty query, returning empty results");
        return Vec::new();
    }

    // Convert user query to proper mdfind query (filename matching)
    let mdquery = build_mdquery(query);
    debug!(mdquery = %mdquery, "Built mdfind query");

    let mut cmd = Command::new("mdfind");

    // Add -onlyin if specified
    if let Some(dir) = onlyin {
        cmd.arg("-onlyin").arg(dir);
    }

    // Add the query
    cmd.arg(&mdquery);

    // Set up streaming: pipe stdout instead of buffering
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    debug!(command = ?cmd, "Spawning mdfind");

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            warn!(error = %e, "Failed to spawn mdfind");
            return Vec::new();
        }
    };

    // Run the search with a timeout to prevent hanging on no-match queries
    // mdfind can hang indefinitely when Spotlight index is being rebuilt or
    // for queries that don't match anything
    let results = search_with_timeout(&mut child, limit);

    // Clean up the child process
    let _ = child.kill();
    let _ = child.wait();

    debug!(result_count = results.len(), "Search completed");
    results
}

/// Internal helper that reads results from mdfind with a timeout.
/// Returns collected results when timeout expires or all results are read.
fn search_with_timeout(child: &mut Child, limit: usize) -> Vec<FileResult> {
    // Take stdout for streaming reads
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            warn!("Failed to capture mdfind stdout");
            return Vec::new();
        }
    };

    // Spawn a thread to read results - this allows us to timeout
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut results = Vec::new();

        // Stream line-by-line, stopping after limit
        for line_result in reader.lines() {
            if results.len() >= limit {
                break;
            }

            let line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    debug!(error = %e, "Error reading mdfind output line");
                    continue;
                }
            };

            // Only skip truly empty lines, not lines with spaces
            // NOTE: .lines() already strips newline characters (\n, \r\n).
            // We intentionally do NOT call trim() because macOS paths CAN contain
            // leading/trailing spaces (rare but valid).
            if line.is_empty() {
                continue;
            }

            let path = Path::new(&line);

            // Get file metadata
            let (size, modified) = match std::fs::metadata(path) {
                Ok(meta) => {
                    let size = meta.len();
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    (size, modified)
                }
                Err(_) => (0, 0),
            };

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let file_type = detect_file_type(path);

            results.push(FileResult {
                path: line,
                name,
                size,
                modified,
                file_type,
            });
        }

        // Send results (ignore error if receiver dropped due to timeout)
        let _ = tx.send(results);
    });

    // Wait for results with timeout
    match rx.recv_timeout(MDFIND_TIMEOUT) {
        Ok(results) => results,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            warn!(
                timeout_secs = MDFIND_TIMEOUT.as_secs(),
                "mdfind search timed out - Spotlight may be slow or query has no matches"
            );
            Vec::new()
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            warn!("mdfind reader thread disconnected unexpectedly");
            Vec::new()
        }
    }
}

/// Get detailed metadata for a specific file
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// Some(FileMetadata) if the file exists and is readable, None otherwise
///
#[allow(dead_code)]
#[instrument(skip_all, fields(path = %path))]
pub fn get_file_metadata(path: &str) -> Option<FileMetadata> {
    debug!("Getting file metadata");

    let path_obj = Path::new(path);

    let metadata = match std::fs::metadata(path_obj) {
        Ok(m) => m,
        Err(e) => {
            debug!(error = %e, "Failed to get file metadata");
            return None;
        }
    };

    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let size = metadata.len();

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let file_type = if metadata.is_dir() {
        FileType::Directory
    } else {
        detect_file_type(path_obj)
    };

    // Check permissions
    let readable = path_obj.exists(); // If we got metadata, it's readable
    let writable = !metadata.permissions().readonly();

    Some(FileMetadata {
        path: path.to_string(),
        name,
        size,
        modified,
        file_type,
        readable,
        writable,
    })
}

// ============================================================================
// UI Helper Functions
// These functions are prepared for file search UI that's being implemented.
// Allow dead_code temporarily until the file search view is complete.
// ============================================================================

/// Get an emoji icon for the file type (used in file search UI)
#[allow(dead_code)]
pub fn file_type_icon(file_type: FileType) -> &'static str {
    match file_type {
        FileType::Directory => "📁",
        FileType::Application => "📦",
        FileType::Image => "🖼️",
        FileType::Document => "📄",
        FileType::Audio => "🎵",
        FileType::Video => "🎬",
        FileType::File => "📃",
        FileType::Other => "📎",
    }
}

/// Format file size in human-readable format (e.g., "1.2 MB", "456 KB")
#[allow(dead_code)]
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format Unix timestamp as relative time (e.g., "2 hours ago", "3 days ago")
#[allow(dead_code)]
pub fn format_relative_time(unix_timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if unix_timestamp == 0 {
        return "Unknown".to_string();
    }

    let diff = now.saturating_sub(unix_timestamp);

    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    if diff < MINUTE {
        "Just now".to_string()
    } else if diff < HOUR {
        let mins = diff / MINUTE;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < DAY {
        let hours = diff / HOUR;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < WEEK {
        let days = diff / DAY;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < MONTH {
        let weeks = diff / WEEK;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < YEAR {
        let months = diff / MONTH;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = diff / YEAR;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

/// Open a file with the system default application
#[allow(dead_code)]
pub fn open_file(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
        Ok(())
    }
}

/// Reveal a file in Finder (macOS) or file manager
#[allow(dead_code)]
pub fn reveal_in_finder(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", path])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Try to get the parent directory and open it
        let parent = std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", path])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
        Ok(())
    }
}

/// Preview a file using Quick Look (macOS)
#[allow(dead_code)]
pub fn quick_look(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("qlmanage")
            .args(["-p", path])
            .spawn()
            .map_err(|e| format!("Failed to preview file: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Quick Look is macOS-only; fall back to opening the file
        open_file(path)
    }
}

/// Show the "Open With" dialog for a file (macOS)
#[allow(dead_code)]
pub fn open_with(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to trigger the "Open With" menu
        let script = format!(
            r#"tell application "Finder"
                activate
                set theFile to POSIX file "{}"
                open information window of theFile
            end tell"#,
            path.replace('"', r#"\""#)
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open 'Open With' dialog: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Open With is only supported on macOS".to_string())
    }
}

/// Show the Get Info window for a file in Finder (macOS)
#[allow(dead_code)]
pub fn show_info(path: &str) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to open the Get Info window
        let script = format!(
            r#"tell application "Finder"
                activate
                set theFile to POSIX file "{}"
                open information window of theFile
            end tell"#,
            path.replace('"', r#"\""#)
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to show file info: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Show Info is only supported on macOS".to_string())
    }
}

// ============================================================================
// Path Navigation Helpers
// These are pure string manipulation helpers for directory navigation UI.
// They work with display paths (containing ~ or relative paths) without IO.
// ============================================================================

/// Ensure a path string ends with a trailing slash
///
/// Used to normalize directory paths for consistent display and navigation.
///
/// # Examples
/// - `/foo/bar` → `/foo/bar/`
/// - `~/dev/` → `~/dev/` (unchanged)
/// - `` → `/` (empty becomes root)
/// - `~` → `~/`
#[allow(dead_code)]
pub fn ensure_trailing_slash(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// Get the parent directory path for Shift+Tab navigation
///
/// This is a pure string operation for display paths. It handles:
/// - Tilde paths (`~/foo/` → `~/`)
/// - Absolute paths (`/foo/bar/` → `/foo/`)
/// - Relative paths (`./` → `../`, `../` → `../../`)
///
/// Returns `None` for root paths that have no parent:
/// - `/` (filesystem root)
/// - `~/` (home directory root)
///
/// # Arguments
/// * `dir_with_slash` - Directory path (ideally ending with `/`, but handles without)
///
/// # Returns
/// * `Some(parent_path)` - Parent directory path ending with `/`
/// * `None` - If this is a root path with no parent
#[allow(dead_code)]
pub fn parent_dir_display(dir_with_slash: &str) -> Option<String> {
    // Normalize: ensure we're working with a trailing-slash path
    let normalized = if dir_with_slash.ends_with('/') {
        dir_with_slash.to_string()
    } else {
        format!("{}/", dir_with_slash)
    };

    // Handle root cases that have no parent
    if normalized == "/" || normalized == "~/" {
        return None;
    }

    // Handle relative paths specially
    if normalized == "./" {
        // Current dir -> parent dir
        return Some("../".to_string());
    }

    if normalized == "../" {
        // One level up -> two levels up
        return Some("../../".to_string());
    }

    // Handle ../ chains: ../../ -> ../../../
    if normalized.starts_with("../") {
        // Count existing ../ segments and add one more
        return Some(format!("../{}", normalized));
    }

    // For regular paths (absolute or tilde), find the parent by removing last segment
    // e.g., "/foo/bar/" -> "/foo/", "~/dev/kit/" -> "~/dev/"

    // Remove trailing slash for easier processing
    let without_trailing = normalized.trim_end_matches('/');

    // Find the last slash (which separates parent from current dir)
    if let Some(last_slash_pos) = without_trailing.rfind('/') {
        // Special case: tilde prefix
        if without_trailing.starts_with("~/") {
            if last_slash_pos == 1 {
                // "~/foo" -> last_slash at 1 -> parent is "~/"
                return Some("~/".to_string());
            }
            // "~/foo/bar" -> parent is "~/foo/"
            return Some(format!("{}/", &without_trailing[..last_slash_pos]));
        }

        // Absolute path case
        if last_slash_pos == 0 {
            // "/foo" -> parent is "/"
            return Some("/".to_string());
        }

        // General case: "/foo/bar" -> "/foo/"
        return Some(format!("{}/", &without_trailing[..last_slash_pos]));
    }

    // No slash found - shouldn't happen for valid directory paths
    None
}

/// Shorten a path for display by using ~ for home directory
#[allow(dead_code)]
pub fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Some(home_str) = home.to_str() {
            if let Some(stripped) = path.strip_prefix(home_str) {
                return format!("~{}", stripped);
            }
        }
    }
    path.to_string()
}

/// Expand a path string, replacing ~ with the home directory
/// and resolving relative paths (., ..)
///
/// # Arguments
/// * `path` - Path string that may contain ~, ., or ..
///
/// # Returns
/// Expanded absolute path as a String, or None if expansion fails
pub fn expand_path(path: &str) -> Option<String> {
    let trimmed = path.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Handle home directory expansion
    if trimmed == "~" {
        return dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string()));
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        return dirs::home_dir().and_then(|home| home.join(rest).to_str().map(|s| s.to_string()));
    }

    // Handle relative paths
    if trimmed == "." || trimmed.starts_with("./") {
        let cwd = std::env::current_dir().ok()?;
        let suffix = trimmed.strip_prefix("./").unwrap_or("");
        if suffix.is_empty() {
            return cwd.to_str().map(|s| s.to_string());
        }
        return cwd.join(suffix).to_str().map(|s| s.to_string());
    }

    if trimmed == ".." || trimmed.starts_with("../") {
        let cwd = std::env::current_dir().ok()?;
        let parent = cwd.parent()?;
        let suffix = trimmed.strip_prefix("../").unwrap_or("");
        if suffix.is_empty() {
            return parent.to_str().map(|s| s.to_string());
        }
        return parent.join(suffix).to_str().map(|s| s.to_string());
    }

    // Already an absolute path
    if trimmed.starts_with('/') {
        return Some(trimmed.to_string());
    }

    // Not a recognized path format
    None
}

/// Internal cap to prevent runaway directory listings
const MAX_DIRECTORY_ENTRIES: usize = 5000;

/// List contents of a directory
///
/// Returns files and directories sorted with directories first, then by name.
/// Handles ~ expansion and relative paths.
///
/// NOTE: Does NOT truncate results. Callers should apply their own limit
/// after scoring/filtering. An internal cap of 5000 entries prevents runaway.
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `_limit` - DEPRECATED: No longer used (kept for API compatibility)
///
/// # Returns
/// Vector of FileResult structs for directory contents
#[instrument(skip_all, fields(dir_path = %dir_path, limit = _limit))]
pub fn list_directory(dir_path: &str, _limit: usize) -> Vec<FileResult> {
    debug!("Starting directory listing");

    // Expand the path
    let expanded = match expand_path(dir_path) {
        Some(p) => p,
        None => {
            debug!("Failed to expand path: {}", dir_path);
            return Vec::new();
        }
    };

    let path = Path::new(&expanded);

    // Check if it's a valid directory
    if !path.is_dir() {
        debug!("Path is not a directory: {}", expanded);
        return Vec::new();
    }

    // Read directory contents
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "Failed to read directory: {}", expanded);
            return Vec::new();
        }
    };

    let mut results: Vec<FileResult> = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let path_str = match entry_path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip hidden files (starting with .)
        if name.starts_with('.') {
            continue;
        }

        // Get metadata
        let (size, modified) = match std::fs::metadata(&entry_path) {
            Ok(meta) => {
                let size = meta.len();
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (size, modified)
            }
            Err(_) => (0, 0),
        };

        let file_type = detect_file_type(&entry_path);

        results.push(FileResult {
            path: path_str,
            name,
            size,
            modified,
            file_type,
        });
    }

    // Sort: directories first, then alphabetically by name
    results.sort_by(|a, b| {
        let a_is_dir = matches!(a.file_type, FileType::Directory);
        let b_is_dir = matches!(b.file_type, FileType::Directory);

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    // Apply internal cap to prevent runaway (callers truncate after filtering)
    if results.len() > MAX_DIRECTORY_ENTRIES {
        results.truncate(MAX_DIRECTORY_ENTRIES);
    }

    debug!(result_count = results.len(), "Directory listing completed");
    results
}

/// Check if a path looks like a directory path that should be listed
/// (as opposed to a search query)
///
/// Re-exports from input_detection module for convenience
pub use crate::scripts::input_detection::is_directory_path;

/// Result of parsing a directory path with potential filter
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDirPath {
    /// The directory to list (always ends with / after expansion)
    pub directory: String,
    /// Optional filter pattern (the part after the last /)
    pub filter: Option<String>,
}

/// Parse a directory path into its directory component and optional filter
///
/// This handles paths like:
/// - `~/dev/` -> directory=`~/dev/`, filter=None (list all)
/// - `~/dev/fin` -> directory=`~/dev/`, filter=Some("fin") (filter by "fin")
/// - `~/dev/mcp-` -> directory=`~/dev/`, filter=Some("mcp-") (filter by "mcp-")
/// - `/usr/local/bin` -> directory=`/usr/local/`, filter=Some("bin")
/// - `~` -> directory=`~`, filter=None
///
/// Returns None if:
/// - The path doesn't look like a directory path
/// - The parent directory doesn't exist
#[instrument(skip_all, fields(path = %path))]
pub fn parse_directory_path(path: &str) -> Option<ParsedDirPath> {
    let trimmed = path.trim();

    // Must be a directory-style path
    if !is_directory_path(trimmed) {
        return None;
    }

    // Handle special case: just ~ or ~/ (home directory)
    if trimmed == "~" || trimmed == "~/" {
        return Some(ParsedDirPath {
            directory: "~".to_string(),
            filter: None,
        });
    }

    // Handle paths ending with / - they're complete directory paths
    if trimmed.ends_with('/') {
        // Verify the directory exists
        if let Some(expanded) = expand_path(trimmed.trim_end_matches('/')) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                return Some(ParsedDirPath {
                    directory: trimmed.to_string(),
                    filter: None,
                });
            }
        }
        return None;
    }

    // Path doesn't end with / - split into parent dir and potential filter
    // e.g., ~/dev/fin -> ~/dev/ + fin
    if let Some(last_slash_idx) = trimmed.rfind('/') {
        let parent = &trimmed[..=last_slash_idx]; // Include the slash
        let potential_filter = &trimmed[last_slash_idx + 1..];

        // Verify parent directory exists
        let parent_to_check = if parent == "/" {
            "/"
        } else {
            parent.trim_end_matches('/')
        };

        if let Some(expanded) = expand_path(parent_to_check) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                let filter = if potential_filter.is_empty() {
                    None
                } else {
                    Some(potential_filter.to_string())
                };
                return Some(ParsedDirPath {
                    directory: parent.to_string(),
                    filter,
                });
            }
        }
    }

    None
}

/// List directory contents with optional filter applied
///
/// This combines directory listing with instant filtering for responsive UX.
/// When the user types `~/dev/fin`, we list `~/dev/` and filter by "fin".
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `filter` - Optional filter string to match against filenames
/// * `limit` - Maximum number of results to return
///
/// # Returns
/// Vector of FileResult structs matching the filter
#[allow(dead_code)]
#[instrument(skip_all, fields(dir_path = %dir_path, filter = ?filter, limit = limit))]
pub fn list_directory_filtered(
    dir_path: &str,
    filter: Option<&str>,
    limit: usize,
) -> Vec<FileResult> {
    // First get all directory contents
    let mut results = list_directory(dir_path, limit * 2); // Get more to filter from

    // Apply filter if present
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        results.retain(|r| r.name.to_lowercase().contains(&filter_lower));
    }

    // Apply limit after filtering
    results.truncate(limit);
    results
}

/// Filter and sort FileResults using Nucleo fuzzy matching
///
/// This function filters cached file results by fuzzy-matching the filter pattern
/// against file names, then sorts by match score (higher = better match).
///
/// # Arguments
/// * `results` - Slice of FileResult to filter
/// * `filter_pattern` - The pattern to fuzzy-match against file names
///
/// # Returns
/// Vector of (original_index, FileResult, score) tuples, sorted by score descending
#[allow(dead_code)]
pub fn filter_results_with_nucleo(
    results: &[FileResult],
    filter_pattern: &str,
) -> Vec<(usize, FileResult, u32)> {
    use crate::scripts::NucleoCtx;

    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, FileResult, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, r.clone(), score)))
        .collect();

    // Sort by score descending (higher = better match)
    scored.sort_by(|a, b| b.2.cmp(&a.2));

    scored
}

/// Filter FileResults using Nucleo and return only (index, FileResult) pairs
///
/// This is a convenience wrapper for use in UI code where the score isn't needed.
/// Results are pre-sorted by match quality.
///
/// # Arguments
/// * `results` - Slice of FileResult to filter
/// * `filter_pattern` - The pattern to fuzzy-match against file names
///
/// # Returns
/// Vector of (original_index, &FileResult) tuples, sorted by match quality
#[allow(dead_code)]
pub fn filter_results_nucleo_simple<'a>(
    results: &'a [FileResult],
    filter_pattern: &str,
) -> Vec<(usize, &'a FileResult)> {
    use crate::scripts::NucleoCtx;

    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, &FileResult, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, r, score)))
        .collect();

    // Sort by score descending (higher = better match)
    scored.sort_by(|a, b| b.2.cmp(&a.2));

    // Return without scores
    scored.into_iter().map(|(idx, r, _)| (idx, r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Query Builder Tests
    // ========================================================================

    #[test]
    fn test_looks_like_advanced_mdquery_detects_kmditem() {
        assert!(looks_like_advanced_mdquery("kMDItemFSName == 'test'"));
        assert!(looks_like_advanced_mdquery(
            "kMDItemContentType == 'public.image'"
        ));
    }

    #[test]
    fn test_looks_like_advanced_mdquery_detects_operators() {
        assert!(looks_like_advanced_mdquery("name == test"));
        assert!(looks_like_advanced_mdquery("size != 0"));
        assert!(looks_like_advanced_mdquery("date >= 2024"));
        assert!(looks_like_advanced_mdquery("size <= 1000"));
        assert!(looks_like_advanced_mdquery("type == image && size > 1000"));
        assert!(looks_like_advanced_mdquery("ext == jpg || ext == png"));
    }

    #[test]
    fn test_looks_like_advanced_mdquery_simple_queries() {
        // Simple text queries should NOT be detected as advanced
        assert!(!looks_like_advanced_mdquery("hello"));
        assert!(!looks_like_advanced_mdquery("my document"));
        assert!(!looks_like_advanced_mdquery("test.txt"));
        assert!(!looks_like_advanced_mdquery("file-name"));
    }

    #[test]
    fn test_escape_md_string_basic() {
        assert_eq!(escape_md_string("hello"), "hello");
        assert_eq!(escape_md_string("test file"), "test file");
    }

    #[test]
    fn test_escape_md_string_quotes() {
        assert_eq!(escape_md_string(r#"file"name"#), r#"file\"name"#);
        assert_eq!(escape_md_string(r#""quoted""#), r#"\"quoted\""#);
    }

    #[test]
    fn test_escape_md_string_backslashes() {
        assert_eq!(escape_md_string(r"path\to\file"), r"path\\to\\file");
        assert_eq!(escape_md_string(r"\escaped\"), r"\\escaped\\");
    }

    #[test]
    fn test_escape_md_string_mixed() {
        assert_eq!(escape_md_string(r#"file\"name"#), r#"file\\\"name"#);
    }

    #[test]
    fn test_build_mdquery_simple_query() {
        let query = build_mdquery("hello");
        assert_eq!(query, r#"kMDItemFSName == "*hello*"c"#);
    }

    #[test]
    fn test_build_mdquery_with_spaces() {
        let query = build_mdquery("my document");
        assert_eq!(query, r#"kMDItemFSName == "*my document*"c"#);
    }

    #[test]
    fn test_build_mdquery_passes_through_advanced() {
        let advanced = "kMDItemFSName == 'test.txt'";
        let query = build_mdquery(advanced);
        assert_eq!(query, advanced); // Should pass through unchanged
    }

    #[test]
    fn test_build_mdquery_with_special_chars() {
        let query = build_mdquery(r#"file"name"#);
        assert_eq!(query, r#"kMDItemFSName == "*file\"name*"c"#);
    }

    #[test]
    fn test_build_mdquery_trims_whitespace() {
        let query = build_mdquery("  hello  ");
        assert_eq!(query, r#"kMDItemFSName == "*hello*"c"#);
    }

    // ========================================================================
    // File Type Detection Tests
    // ========================================================================

    #[test]
    fn test_detect_file_type_image() {
        assert_eq!(
            detect_file_type(Path::new("/test/photo.png")),
            FileType::Image
        );
        assert_eq!(
            detect_file_type(Path::new("/test/photo.JPG")),
            FileType::Image
        );
        assert_eq!(
            detect_file_type(Path::new("/test/photo.heic")),
            FileType::Image
        );
    }

    #[test]
    fn test_detect_file_type_document() {
        assert_eq!(
            detect_file_type(Path::new("/test/doc.pdf")),
            FileType::Document
        );
        assert_eq!(
            detect_file_type(Path::new("/test/doc.docx")),
            FileType::Document
        );
        assert_eq!(
            detect_file_type(Path::new("/test/doc.txt")),
            FileType::Document
        );
    }

    #[test]
    fn test_detect_file_type_audio() {
        assert_eq!(
            detect_file_type(Path::new("/test/song.mp3")),
            FileType::Audio
        );
        assert_eq!(
            detect_file_type(Path::new("/test/song.wav")),
            FileType::Audio
        );
    }

    #[test]
    fn test_detect_file_type_video() {
        assert_eq!(
            detect_file_type(Path::new("/test/movie.mp4")),
            FileType::Video
        );
        assert_eq!(
            detect_file_type(Path::new("/test/movie.mov")),
            FileType::Video
        );
    }

    #[test]
    fn test_detect_file_type_application() {
        assert_eq!(
            detect_file_type(Path::new("/Applications/Safari.app")),
            FileType::Application
        );
    }

    #[test]
    fn test_detect_file_type_generic_file() {
        assert_eq!(
            detect_file_type(Path::new("/test/script.rs")),
            FileType::File
        );
        assert_eq!(
            detect_file_type(Path::new("/test/config.json")),
            FileType::File
        );
    }

    #[test]
    fn test_search_files_empty_query() {
        let results = search_files("", None, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_file_result_creation() {
        let result = FileResult {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
        };

        assert_eq!(result.path, "/test/file.txt");
        assert_eq!(result.name, "file.txt");
        assert_eq!(result.size, 1024);
        assert_eq!(result.file_type, FileType::Document);
    }

    #[test]
    fn test_file_metadata_creation() {
        let meta = FileMetadata {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
            readable: true,
            writable: true,
        };

        assert_eq!(meta.path, "/test/file.txt");
        assert!(meta.readable);
        assert!(meta.writable);
    }

    #[test]
    fn test_default_file_type() {
        assert_eq!(FileType::default(), FileType::Other);
    }

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_search_files_real_query() {
        // This test only runs on macOS and verifies mdfind works
        let results = search_files("System Preferences", Some("/System"), 5);
        // We don't assert specific results as they may vary,
        // but the function should not panic
        assert!(results.len() <= 5);
    }

    #[cfg(all(target_os = "macos", feature = "slow-tests"))]
    #[test]
    fn test_get_file_metadata_real_file() {
        // Test with a file that should exist on all macOS systems
        let meta = get_file_metadata("/System/Library/CoreServices/Finder.app");
        // Finder.app should exist on macOS
        if let Some(m) = meta {
            assert!(!m.name.is_empty());
            assert!(m.readable);
        }
        // It's OK if this returns None on some systems
    }

    // ========================================================================
    // UI Helper Function Tests
    // ========================================================================

    #[test]
    fn test_file_type_icon() {
        assert_eq!(file_type_icon(FileType::Directory), "📁");
        assert_eq!(file_type_icon(FileType::Application), "📦");
        assert_eq!(file_type_icon(FileType::Image), "🖼️");
        assert_eq!(file_type_icon(FileType::Document), "📄");
        assert_eq!(file_type_icon(FileType::Audio), "🎵");
        assert_eq!(file_type_icon(FileType::Video), "🎬");
        assert_eq!(file_type_icon(FileType::File), "📃");
        assert_eq!(file_type_icon(FileType::Other), "📎");
    }

    #[test]
    fn test_format_file_size() {
        // Bytes
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1023), "1023 B");

        // Kilobytes
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(10240), "10.0 KB");

        // Megabytes
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.0 MB");

        // Gigabytes
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.0 GB");
    }

    #[test]
    fn test_format_relative_time() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Just now (0 seconds ago)
        assert_eq!(format_relative_time(now), "Just now");

        // Minutes ago
        assert_eq!(format_relative_time(now - 60), "1 min ago");
        assert_eq!(format_relative_time(now - 120), "2 mins ago");
        assert_eq!(format_relative_time(now - 59 * 60), "59 mins ago");

        // Hours ago
        assert_eq!(format_relative_time(now - 3600), "1 hour ago");
        assert_eq!(format_relative_time(now - 7200), "2 hours ago");

        // Days ago
        assert_eq!(format_relative_time(now - 86400), "1 day ago");
        assert_eq!(format_relative_time(now - 172800), "2 days ago");

        // Unknown (0 timestamp)
        assert_eq!(format_relative_time(0), "Unknown");
    }

    #[test]
    fn test_shorten_path() {
        // Test with a path that doesn't start with home
        assert_eq!(shorten_path("/usr/local/bin"), "/usr/local/bin");
        assert_eq!(shorten_path("/etc/hosts"), "/etc/hosts");

        // Test with home directory path (if home dir is available)
        if let Some(home) = dirs::home_dir() {
            if let Some(home_str) = home.to_str() {
                let test_path = format!("{}/Documents/test.txt", home_str);
                assert_eq!(shorten_path(&test_path), "~/Documents/test.txt");
            }
        }
    }

    // ========================================================================
    // Directory Navigation Tests
    // ========================================================================

    #[test]
    fn test_expand_path_home() {
        // Test ~ expansion
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_str().unwrap();

            // Just ~
            assert_eq!(expand_path("~"), Some(home_str.to_string()));

            // ~/subdir
            let expanded = expand_path("~/Documents");
            assert!(expanded.is_some());
            assert!(expanded.unwrap().starts_with(home_str));
        }
    }

    #[test]
    fn test_expand_path_absolute() {
        // Absolute paths should pass through unchanged
        assert_eq!(expand_path("/usr/local"), Some("/usr/local".to_string()));
        assert_eq!(expand_path("/"), Some("/".to_string()));
        assert_eq!(
            expand_path("/System/Library"),
            Some("/System/Library".to_string())
        );
    }

    #[test]
    fn test_expand_path_relative_current() {
        // Relative paths with .
        let cwd = std::env::current_dir().unwrap();
        let cwd_str = cwd.to_str().unwrap();

        // Just .
        let expanded = expand_path(".");
        assert!(expanded.is_some());
        assert_eq!(expanded.unwrap(), cwd_str);

        // ./subdir
        let expanded = expand_path("./src");
        assert!(expanded.is_some());
        let expected = cwd.join("src");
        assert_eq!(expanded.unwrap(), expected.to_str().unwrap());
    }

    #[test]
    fn test_expand_path_relative_parent() {
        // Relative paths with ..
        let cwd = std::env::current_dir().unwrap();
        if let Some(parent) = cwd.parent() {
            let parent_str = parent.to_str().unwrap();

            // Just ..
            let expanded = expand_path("..");
            assert!(expanded.is_some());
            assert_eq!(expanded.unwrap(), parent_str);
        }
    }

    #[test]
    fn test_expand_path_empty() {
        assert_eq!(expand_path(""), None);
        assert_eq!(expand_path("   "), None);
    }

    #[test]
    fn test_expand_path_not_path() {
        // Regular text should return None
        assert_eq!(expand_path("hello"), None);
        assert_eq!(expand_path("search query"), None);
    }

    #[test]
    fn test_list_directory_nonexistent() {
        // Non-existent directory should return empty
        let results = list_directory("/this/path/does/not/exist/at/all", 50);
        assert!(results.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_list_directory_system() {
        // List /System which exists on all macOS systems
        let results = list_directory("/System", 10);
        assert!(!results.is_empty(), "Should find items in /System");

        // Should contain Library
        let has_library = results.iter().any(|r| r.name == "Library");
        assert!(has_library, "Should contain Library folder");

        // Library should be marked as directory
        let library = results.iter().find(|r| r.name == "Library");
        if let Some(lib) = library {
            assert_eq!(lib.file_type, FileType::Directory);
        }
    }

    #[test]
    fn test_list_directory_home() {
        // List home directory using ~
        let results = list_directory("~", 100);

        // Home should have at least some contents
        // (assuming it's a valid home directory)
        // Don't assert specific files as they vary by system
        assert!(
            results.is_empty() || !results.is_empty(),
            "Should not panic on home directory"
        );
    }

    #[test]
    fn test_list_directory_dirs_first() {
        // Test using /tmp which usually has both dirs and files
        let results = list_directory("/tmp", 50);

        // If we have results, verify sorting
        if results.len() >= 2 {
            // Find first file (non-directory)
            let first_file_idx = results
                .iter()
                .position(|r| !matches!(r.file_type, FileType::Directory));

            // Find last directory
            let last_dir_idx = results
                .iter()
                .rposition(|r| matches!(r.file_type, FileType::Directory));

            // If we have both dirs and files, dirs should come first
            if let (Some(first_file), Some(last_dir)) = (first_file_idx, last_dir_idx) {
                assert!(
                    last_dir < first_file,
                    "Directories should come before files"
                );
            }
        }
    }

    #[test]
    fn test_list_directory_limit() {
        // limit parameter is deprecated - list_directory no longer truncates
        // Callers should apply their own limit after filtering/scoring
        // We just verify that it doesn't panic and returns reasonable results
        let results = list_directory("/", 3);
        // Should return all entries (up to internal cap) not just 3
        // The "/" directory typically has multiple entries
        assert!(!results.is_empty(), "Root directory should have entries");
        // Verify internal cap works (5000)
        assert!(results.len() <= 5000, "Should respect internal cap of 5000");
    }

    #[test]
    fn test_list_directory_hides_dotfiles() {
        // Hidden files (starting with .) should be excluded
        let results = list_directory("~", 100);

        for result in &results {
            assert!(
                !result.name.starts_with('.'),
                "Should not include hidden files: {}",
                result.name
            );
        }
    }

    #[test]
    fn test_is_directory_path_reexport() {
        // Verify the re-export works
        assert!(is_directory_path("~/dev"));
        assert!(is_directory_path("/usr/local"));
        assert!(is_directory_path("./src"));
        assert!(!is_directory_path("hello world"));
    }

    // ========================================================================
    // Nucleo Filtering Tests
    // ========================================================================

    #[test]
    fn test_filter_results_nucleo_empty_pattern() {
        let results = vec![
            FileResult {
                path: "/test/apple.txt".to_string(),
                name: "apple.txt".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::Document,
            },
            FileResult {
                path: "/test/banana.txt".to_string(),
                name: "banana.txt".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::Document,
            },
        ];

        // Empty pattern with Nucleo matches everything (score 0)
        // This is expected behavior - caller should check for empty pattern before calling
        let filtered = filter_results_nucleo_simple(&results, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_results_nucleo_exact_match() {
        let results = vec![
            FileResult {
                path: "/test/mcp-final.txt".to_string(),
                name: "mcp-final".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/definitions.txt".to_string(),
                name: "definitions".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "final" should match "mcp-final" better than "definitions"
        let filtered = filter_results_nucleo_simple(&results, "final");
        assert!(!filtered.is_empty());
        assert_eq!(filtered[0].1.name, "mcp-final");
    }

    #[test]
    fn test_filter_results_nucleo_fuzzy_ordering() {
        let results = vec![
            FileResult {
                path: "/test/define.txt".to_string(),
                name: "define".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/mcp-final.txt".to_string(),
                name: "mcp-final".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/final-test.txt".to_string(),
                name: "final-test".to_string(),
                size: 300,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "fin" should fuzzy match both "mcp-final" and "final-test"
        // Both should rank higher than "define" (which has f, i, n but not consecutive)
        let filtered = filter_results_nucleo_simple(&results, "fin");

        // Should have matches
        assert!(!filtered.is_empty());

        // "final-test" or "mcp-final" should be first (both have "fin" as prefix of "final")
        let first_name = &filtered[0].1.name;
        assert!(
            first_name.contains("final"),
            "Expected 'final' in first result, got: {}",
            first_name
        );
    }

    #[test]
    fn test_filter_results_nucleo_no_matches() {
        let results = vec![
            FileResult {
                path: "/test/apple.txt".to_string(),
                name: "apple".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::File,
            },
            FileResult {
                path: "/test/banana.txt".to_string(),
                name: "banana".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::File,
            },
        ];

        // "xyz" should not match anything
        let filtered = filter_results_nucleo_simple(&results, "xyz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_results_nucleo_case_insensitive() {
        let results = vec![FileResult {
            path: "/test/MyDocument.txt".to_string(),
            name: "MyDocument".to_string(),
            size: 100,
            modified: 0,
            file_type: FileType::Document,
        }];

        // Should match regardless of case
        let filtered_lower = filter_results_nucleo_simple(&results, "mydoc");
        let filtered_upper = filter_results_nucleo_simple(&results, "MYDOC");
        let filtered_mixed = filter_results_nucleo_simple(&results, "MyDoc");

        assert!(!filtered_lower.is_empty());
        assert!(!filtered_upper.is_empty());
        assert!(!filtered_mixed.is_empty());
    }

    // ========================================================================
    // FileInfo Tests
    // ========================================================================

    #[test]
    fn test_file_info_from_result() {
        let result = FileResult {
            path: "/test/document.pdf".to_string(),
            name: "document.pdf".to_string(),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
        };

        let info = FileInfo::from_result(&result);
        assert_eq!(info.path, "/test/document.pdf");
        assert_eq!(info.name, "document.pdf");
        assert_eq!(info.file_type, FileType::Document);
        assert!(!info.is_dir);
    }

    #[test]
    fn test_file_info_from_result_directory() {
        let result = FileResult {
            path: "/test/Documents".to_string(),
            name: "Documents".to_string(),
            size: 0,
            modified: 1234567890,
            file_type: FileType::Directory,
        };

        let info = FileInfo::from_result(&result);
        assert_eq!(info.path, "/test/Documents");
        assert_eq!(info.name, "Documents");
        assert_eq!(info.file_type, FileType::Directory);
        assert!(info.is_dir);
    }

    #[test]
    fn test_file_info_from_path() {
        // Test with a path that likely exists
        let info = FileInfo::from_path("/tmp");
        assert_eq!(info.path, "/tmp");
        assert_eq!(info.name, "tmp");
        // /tmp should be a directory on Unix systems
        #[cfg(unix)]
        assert!(info.is_dir);
    }

    // ========================================================================
    // Path Utility Tests (ensure_trailing_slash, parent_dir_display)
    // ========================================================================

    #[test]
    fn test_ensure_trailing_slash_already_has_slash() {
        assert_eq!(ensure_trailing_slash("/foo/bar/"), "/foo/bar/");
        assert_eq!(ensure_trailing_slash("~/dev/"), "~/dev/");
        assert_eq!(ensure_trailing_slash("/"), "/");
        assert_eq!(ensure_trailing_slash("~/"), "~/");
    }

    #[test]
    fn test_ensure_trailing_slash_needs_slash() {
        assert_eq!(ensure_trailing_slash("/foo/bar"), "/foo/bar/");
        assert_eq!(ensure_trailing_slash("~/dev"), "~/dev/");
        assert_eq!(ensure_trailing_slash(".."), "../");
        assert_eq!(ensure_trailing_slash("."), "./");
    }

    #[test]
    fn test_ensure_trailing_slash_edge_cases() {
        // Empty string
        assert_eq!(ensure_trailing_slash(""), "/");
        // Single tilde
        assert_eq!(ensure_trailing_slash("~"), "~/");
    }

    #[test]
    fn test_parent_dir_display_root() {
        // "/" has no parent
        assert_eq!(parent_dir_display("/"), None);
    }

    #[test]
    fn test_parent_dir_display_home_root() {
        // "~/" has no parent (home directory is treated as root)
        assert_eq!(parent_dir_display("~/"), None);
    }

    #[test]
    fn test_parent_dir_display_relative_parent() {
        // "../" -> "../../"
        assert_eq!(parent_dir_display("../"), Some("../../".to_string()));
    }

    #[test]
    fn test_parent_dir_display_relative_current() {
        // "./" -> "../"
        assert_eq!(parent_dir_display("./"), Some("../".to_string()));
    }

    #[test]
    fn test_parent_dir_display_tilde_subdir() {
        // "~/foo/" -> "~/"
        assert_eq!(parent_dir_display("~/foo/"), Some("~/".to_string()));
        // "~/foo/bar/" -> "~/foo/"
        assert_eq!(parent_dir_display("~/foo/bar/"), Some("~/foo/".to_string()));
    }

    #[test]
    fn test_parent_dir_display_absolute_subdir() {
        // "/foo/bar/" -> "/foo/"
        assert_eq!(parent_dir_display("/foo/bar/"), Some("/foo/".to_string()));
        // "/foo/" -> "/"
        assert_eq!(parent_dir_display("/foo/"), Some("/".to_string()));
    }

    #[test]
    fn test_parent_dir_display_multiple_levels() {
        // Deep paths
        assert_eq!(parent_dir_display("/a/b/c/d/"), Some("/a/b/c/".to_string()));
        assert_eq!(
            parent_dir_display("~/projects/rust/kit/"),
            Some("~/projects/rust/".to_string())
        );
    }

    #[test]
    fn test_parent_dir_display_no_trailing_slash() {
        // Paths without trailing slash should still work (normalize first)
        // The function expects trailing slash, but should handle edge cases gracefully
        assert_eq!(parent_dir_display("/foo/bar"), Some("/foo/".to_string()));
        assert_eq!(parent_dir_display("~/foo"), Some("~/".to_string()));
    }
}

</file>

</files>
---

# Context: UI, Input Handling, and Integration

## Entry Point: open_file_search (src/app_execute.rs lines 1051-1118)

```rust
    /// - Live search as user types (debounced)
    /// - File type icons (folder, document, image, audio, video, code, etc.)
    /// - File size and modified date display
    /// - Enter: Open file in default application
    /// - Cmd+Enter: Reveal in Finder
    pub fn open_file_search(&mut self, query: String, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Opening File Search with query: {}", query),
        );

        // Perform initial search or directory listing
        // Check if query looks like a directory path
        let results = if file_search::is_directory_path(&query) {
            logging::log(
                "EXEC",
                &format!("Detected directory path, listing: {}", query),
            );
            // Verify path is actually a directory before listing
            let expanded = file_search::expand_path(&query);
            let is_real_dir = expanded
                .as_deref()
                .map(|p| std::path::Path::new(p).is_dir())
                .unwrap_or(false);

            let dir_results = file_search::list_directory(&query, file_search::DEFAULT_CACHE_LIMIT);

            // Fallback to Spotlight search if path looks like directory but isn't
            if dir_results.is_empty() && !is_real_dir {
                logging::log(
                    "EXEC",
                    "Path mode not a real directory; falling back to Spotlight search",
                );
                file_search::search_files(&query, None, file_search::DEFAULT_SEARCH_LIMIT)
            } else {
                dir_results
            }
        } else {
            file_search::search_files(&query, None, file_search::DEFAULT_SEARCH_LIMIT)
        };
        logging::log(
            "EXEC",
            &format!("File search found {} results", results.len()),
        );

        // Cache the results
        self.cached_file_results = results;

        // Set up the view state
        self.filter_text = query.clone();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search files...".to_string());

        // Switch to file search view
        self.current_view = AppView::FileSearchView {
            query,
            selected_index: 0,
        };

        // Use standard height for file search view (same as window switcher)
        resize_to_view_sync(ViewType::ScriptList, 0);

        // Focus the main filter input so cursor blinks and typing works
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;

        cx.notify();
    }
```

## Built-in Registration (src/builtins.rs lines 931-949)

```rust
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-file-search",
        "Search Files",
        "Browse directories and search for files",
        vec![
            "file",
            "search",
            "find",
            "directory",
            "folder",
            "browse",
            "navigate",
            "path",
            "open",
            "explorer",
        ],
        BuiltInFeature::FileSearch,
        "folder-search",
    ));
```

## FileSearchView Input Handling (src/app_impl.rs lines 2074-2230)

This section handles text input changes in FileSearchView, including:
- Directory path detection and listing
- Debounced Spotlight search
- Loading state management

```rust
            AppView::FileSearchView {
                query,
                selected_index,
            } => {
                if *query != new_text {
                    // Update query immediately for responsive UI
                    *query = new_text.clone();
                    *selected_index = 0;

                    // Cancel existing debounce task
                    self.file_search_debounce_task = None;

                    // Check if this is a directory path with potential filter
                    // e.g., ~/dev/fin -> list ~/dev/ and filter by "fin"
                    if let Some(parsed) = crate::file_search::parse_directory_path(&new_text) {
                        // Directory path mode - check if we need to reload directory
                        let dir_changed =
                            self.file_search_current_dir.as_ref() != Some(&parsed.directory);

                        if dir_changed {
                            // Directory changed - need to load new directory contents
                            // Clear old results to prevent flash of wrong directory items
                            // The render will show "Loading..." when loading with empty results
                            self.cached_file_results.clear();
                            self.file_search_current_dir = Some(parsed.directory.clone());
                            self.file_search_loading = true;
                            // Reset scroll immediately to prevent stale scroll position
                            self.file_search_scroll_handle
                                .scroll_to_item(0, ScrollStrategy::Top);
                            cx.notify();

                            let dir_to_list = parsed.directory.clone();
                            let task = cx.spawn(async move |this, cx| {
                                // Small debounce for directory listing
                                Timer::after(std::time::Duration::from_millis(50)).await;

                                let (tx, rx) = std::sync::mpsc::channel();
                                std::thread::spawn(move || {
                                    let results = crate::file_search::list_directory(
                                        &dir_to_list,
                                        crate::file_search::DEFAULT_CACHE_LIMIT,
                                    );
                                    let _ = tx.send(results);
                                });

                                loop {
                                    Timer::after(std::time::Duration::from_millis(10)).await;
                                    match rx.try_recv() {
                                        Ok(results) => {
                                            let _ = cx.update(|cx| {
                                                this.update(cx, |app, cx| {
                                                    app.cached_file_results = results;
                                                    app.file_search_loading = false;
                                                    // Reset selected_index when async results arrive
                                                    // to prevent bounds issues if results shrink
                                                    if let AppView::FileSearchView {
                                                        selected_index,
                                                        ..
                                                    } = &mut app.current_view
                                                    {
                                                        *selected_index = 0;
                                                    }
                                                    app.file_search_scroll_handle
                                                        .scroll_to_item(0, ScrollStrategy::Top);
                                                    cx.notify();
                                                })
                                            });
                                            break;
                                        }
                                        Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                                    }
                                }
                            });
                            self.file_search_debounce_task = Some(task);
                        } else {
                            // Same directory - just filter existing results (instant!)
                            // Filtering is done in render based on query
                            self.file_search_loading = false;
                            cx.notify();
                        }
                        return; // Don't run main menu filter logic
                    }

                    // Not a directory path - do regular file search with debounce
                    self.file_search_current_dir = None;
                    self.file_search_loading = true;
                    cx.notify();

                    // Debounce: wait 200ms before searching
                    let search_query = new_text.clone();
                    let task = cx.spawn(async move |this, cx| {
                        // Wait for debounce period
                        Timer::after(std::time::Duration::from_millis(200)).await;

                        // Run search in background thread
                        let (tx, rx) = std::sync::mpsc::channel();
                        let query_for_thread = search_query.clone();
                        std::thread::spawn(move || {
                            let results = crate::file_search::search_files(
                                &query_for_thread,
                                None,
                                crate::file_search::DEFAULT_SEARCH_LIMIT,
                            );
                            let _ = tx.send(results);
                        });

                        // Poll for results
                        loop {
                            Timer::after(std::time::Duration::from_millis(10)).await;
                            match rx.try_recv() {
                                Ok(results) => {
                                    let result_count = results.len();
                                    let _ = cx.update(|cx| {
                                        this.update(cx, |app, cx| {
                                            // Only update if query still matches (user hasn't typed more)
                                            if let AppView::FileSearchView { query, .. } =
                                                &app.current_view
                                            {
                                                if *query == search_query {
                                                    logging::log(
                                                        "EXEC",
                                                        &format!(
                                                            "File search for '{}' found {} results",
                                                            search_query, result_count
                                                        ),
                                                    );
                                                    app.cached_file_results = results;
                                                    app.file_search_loading = false;
                                                    // Reset selected_index when async results arrive
                                                    // to prevent bounds issues if results shrink
                                                    if let AppView::FileSearchView {
                                                        selected_index,
                                                        ..
                                                    } = &mut app.current_view
                                                    {
                                                        *selected_index = 0;
                                                    }
                                                    app.file_search_scroll_handle
                                                        .scroll_to_item(0, ScrollStrategy::Top);
                                                    cx.notify();
                                                }
                                            }
                                        })
                                    });
                                    break;
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                            }
                        }
                    });

                    // Store task so it can be cancelled if user types more
                    self.file_search_debounce_task = Some(task);
                }
                return; // Don't run main menu filter logic
```

## Tab/Shift+Tab Navigation (src/app_impl.rs lines 374-490)

Handles directory navigation - Tab to enter directory, Shift+Tab to go up.

```rust
                            // Handle Tab/Shift+Tab in FileSearchView for directory/file navigation
                            // CRITICAL: ALWAYS consume Tab/Shift+Tab to prevent focus traversal
                            if let AppView::FileSearchView {
                                query,
                                selected_index,
                            } = &mut this.current_view
                            {
                                // ALWAYS stop propagation for Tab/Shift+Tab in FileSearchView
                                // This prevents Tab from falling through to focus traversal
                                cx.stop_propagation();

                                if has_shift {
                                    // Shift+Tab: Go up one directory level using parent_dir_display helper
                                    // This handles ~/, /, ./, ../ and regular paths correctly
                                    if let Some(parent_path) =
                                        crate::file_search::parent_dir_display(query)
                                    {
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Navigating up from '{}' to '{}'",
                                                query, parent_path
                                            ),
                                        );

                                        // Update the input - handle_filter_input_change will:
                                        // - Update query
                                        // - Reset selected_index to 0
                                        // - Detect directory change
                                        // - Trigger async directory load
                                        this.gpui_input_state.update(cx, |state, cx| {
                                            state.set_value(parent_path.clone(), window, cx);
                                            // Ensure cursor is at end with no selection after programmatic set_value
                                            // This prevents issues where GPUI might leave caret at wrong position
                                            let len = parent_path.len();
                                            state.set_selection(len, len, window, cx);
                                        });

                                        cx.notify();
                                    } else {
                                        // At root (/ or ~/) - no parent to navigate to
                                        // Key is consumed (stop_propagation called above) but no action taken
                                        crate::logging::log(
                                            "KEY",
                                            &format!(
                                                "Shift+Tab: Already at root '{}', no-op",
                                                query
                                            ),
                                        );
                                    }
                                } else {
                                    // Tab: Enter directory OR autocomplete file name
                                    // Get filtered results to find selected item
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    let filtered_results: Vec<_> =
                                        if let Some(ref pattern) = filter_pattern {
                                            crate::file_search::filter_results_nucleo_simple(
                                                &this.cached_file_results,
                                                pattern,
                                            )
                                        } else {
                                            this.cached_file_results.iter().enumerate().collect()
                                        };

                                    // Defensive bounds check: clamp selected_index if out of bounds
                                    let filtered_len = filtered_results.len();
                                    if filtered_len > 0 && *selected_index >= filtered_len {
                                        *selected_index = filtered_len - 1;
                                    }

                                    if let Some((_, file)) = filtered_results.get(*selected_index) {
                                        if file.file_type == crate::file_search::FileType::Directory
                                        {
                                            // Directory: Enter it (append /)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            let new_path = format!("{}/", shortened);
                                            crate::logging::log(
                                                "KEY",
                                                &format!("Tab: Entering directory: {}", new_path),
                                            );

                                            // Update the input - handle_filter_input_change handles the rest
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(new_path.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = new_path.len();
                                                state.set_selection(len, len, window, cx);
                                            });

                                            cx.notify();
                                        } else {
                                            // File: Autocomplete the full path (terminal-style tab completion)
                                            let shortened =
                                                crate::file_search::shorten_path(&file.path);
                                            crate::logging::log(
                                                "KEY",
                                                &format!(
                                                    "Tab: Autocompleting file path: {}",
                                                    shortened
                                                ),
                                            );

                                            // Set the input to the file's full path
                                            this.gpui_input_state.update(cx, |state, cx| {
                                                state.set_value(shortened.clone(), window, cx);
                                                // Ensure cursor is at end with no selection after programmatic set_value
                                                let len = shortened.len();
```

## Arrow Key Navigation (src/app_impl.rs lines 550-620)

```rust
                    && !event.keystroke.modifiers.control
                {
                    if let Some(app) = app_entity.upgrade() {
                        app.update(cx, |this, cx| {
                            // Only intercept in views that use Input + list navigation
                            match &mut this.current_view {
                                AppView::FileSearchView {
                                    selected_index,
                                    query,
                                } => {
                                    // CRITICAL: If actions popup is open, route to actions dialog instead
                                    if this.show_actions_popup {
                                        if let Some(ref dialog) = this.actions_dialog {
                                            if key == "up" || key == "arrowup" {
                                                dialog.update(cx, |d, cx| d.move_up(cx));
                                            } else if key == "down" || key == "arrowdown" {
                                                dialog.update(cx, |d, cx| d.move_down(cx));
                                            }
                                            // Notify the actions window to re-render
                                            crate::actions::notify_actions_window(cx);
                                        }
                                        cx.stop_propagation();
                                        return;
                                    }

                                    // Compute filtered length using same logic as render
                                    let filter_pattern = if let Some(parsed) =
                                        crate::file_search::parse_directory_path(query)
                                    {
                                        parsed.filter
                                    } else if !query.is_empty() {
                                        Some(query.clone())
                                    } else {
                                        None
                                    };

                                    // Use Nucleo fuzzy matching for consistent filtering with render
                                    let filtered_len = if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                        .len()
                                    } else {
                                        this.cached_file_results.len()
                                    };

                                    if (key == "up" || key == "arrowup") && *selected_index > 0 {
                                        *selected_index -= 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    } else if (key == "down" || key == "arrowdown")
                                        && *selected_index + 1 < filtered_len
                                    {
                                        *selected_index += 1;
                                        this.file_search_scroll_handle.scroll_to_item(
                                            *selected_index,
                                            gpui::ScrollStrategy::Nearest,
                                        );
                                        cx.notify();
                                    }
                                    // Stop propagation so Input doesn't handle it
                                    cx.stop_propagation();
                                }
                                AppView::ClipboardHistoryView {
                                    selected_index,
                                    filter: _,
                                } => {
```

## Render File Search UI (src/render_builtins.rs lines 1987-2200)

The main render function creating the 50/50 split view with file list and preview.

```rust
    /// Render file search view with 50/50 split (list + preview)
    pub(crate) fn render_file_search(
        &mut self,
        query: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::file_search::{self, FileType};

        // Use design tokens for spacing/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let _design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let _opacity = self.theme.get_opacity();
        // bg_with_alpha removed - let vibrancy show through from Root (matches main menu)
        let box_shadows = self.create_box_shadows();

        // Color values for use in closures
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let _accent_color = self.theme.colors.accent.selected;
        let list_hover = self.theme.colors.accent.selected_subtle;
        let list_selected = self.theme.colors.accent.selected_subtle;
        // Use theme opacity for vibrancy-compatible selection/hover (matches main menu)
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0) as u32;

        // Filter results based on query
        // When query is a directory path, extract the filter component for instant filtering
        // e.g., ~/dev/fin -> filter by "fin" on directory contents
        let filter_pattern = if let Some(parsed) = crate::file_search::parse_directory_path(query) {
            parsed.filter // Some("fin") or None
        } else if !query.is_empty() {
            // Not a directory path - use query as filter for search results
            Some(query.to_string())
        } else {
            None
        };

        // Use Nucleo fuzzy matching for filtering - gives better match quality ranking
        let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
            file_search::filter_results_nucleo_simple(&self.cached_file_results, pattern)
        } else {
            // No filter - show all results
            self.cached_file_results.iter().enumerate().collect()
        };
        let filtered_len = filtered_results.len();

        // Get selected file for preview (if any)
        let selected_file = filtered_results
            .get(selected_index)
            .map(|(_, r)| (*r).clone());

        // Key handler for file search
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        // Use handle_action instead of trigger_action_by_name to support
                        // both built-in actions (open_file, quick_look, etc.) and SDK actions
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC goes back to main menu (not close window)
                if key_str == "escape" {
                    logging::log("KEY", "ESC in FileSearch - returning to main menu");
                    // Cancel any pending search
                    this.file_search_debounce_task = None;
                    this.file_search_loading = false;
                    // Clear cached results
                    this.cached_file_results.clear();
                    // Return to main menu
                    this.current_view = AppView::ScriptList;
                    this.filter_text.clear();
                    this.selected_index = 0;
                    // Sync input and reset placeholder to default
                    this.gpui_input_state.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                        // Ensure cursor is at start (empty string, so 0..0)
                        state.set_selection(0, 0, window, cx);
                        state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
                    });
                    this.update_window_size_deferred(window, cx);
                    cx.notify();
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // Use parse_directory_path to extract filter pattern
                    let filter_pattern =
                        if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                            parsed.filter
                        } else if !query.is_empty() {
                            Some(query.clone())
                        } else {
                            None
                        };

                    // Use Nucleo fuzzy matching for filtering
                    let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
                        crate::file_search::filter_results_nucleo_simple(
                            &this.cached_file_results,
                            pattern,
                        )
                    } else {
                        this.cached_file_results.iter().enumerate().collect()
                    };
                    let _filtered_len = filtered_results.len();

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    // Clone the file to avoid borrow issues
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
```

## Loading State & Empty State UI (src/render_builtins.rs lines 2500-2588)

Shows "Searching..." when loading, "No files found" when empty.

```rust
                                    .text_sm()
                                    .text_color(rgb(text_dimmed))
                                    .child(format!("{} files", filtered_len)),
                            ),
                    )
            })
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content: loading state OR empty state OR 50/50 split
            .child(if is_loading && filtered_len == 0 {
                // Loading state: full-width centered (no split, clean appearance)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("Searching..."),
                    )
            } else if filtered_len == 0 {
                // Empty state: single centered message (no awkward 50/50 split)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div().flex().flex_col().items_center().gap(px(8.)).child(
                            div()
                                .text_color(rgb(text_dimmed))
                                .child(if query.is_empty() {
                                    "Type to search files"
                                } else {
                                    "No files found"
                                }),
                        ),
                    )
            } else {
                // Normal state: 50/50 split with list and preview
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .min_h(px(0.))
                    .overflow_hidden()
                    // Left panel: file list (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .border_r(px(design_visual.border_thin))
                            .border_color(rgba((ui_border << 8) | 0x40))
                            .child(list_element),
                    )
                    // Right panel: preview (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(preview_content),
                    )
            })
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Open")
                    .primary_shortcut("↵"),
                // Default config already has secondary_label="Actions", secondary_shortcut="⌘K", show_secondary=true
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()
    }
}
```

## App State Definition (src/main.rs lines 1065-1073)

State fields for file search:

```rust
    file_search_scroll_handle: UniformListScrollHandle,
    // File search loading state (true while mdfind is running)
    file_search_loading: bool,
    // Debounce task for file search (cancelled when new input arrives)
    file_search_debounce_task: Option<gpui::Task<()>>,
    // Current directory being listed (for instant filter mode)
    file_search_current_dir: Option<String>,
    // Path of the file selected for actions (for file search actions handling)
    file_search_actions_path: Option<String>,
```

## AppView Enum - FileSearchView variant (src/main.rs lines 754-760)

```rust
    },
    /// Showing file search results
    FileSearchView {
        query: String,
        selected_index: usize,
    },
}
```

## Built-in Feature Enum (src/builtins.rs lines 10-35)

```rust
//! - **System Actions**: Power management, UI controls, volume/brightness
//! - **Window Actions**: Window tiling and management for the frontmost window
//! - **Notes Commands**: Notes window operations
//! - **AI Commands**: AI chat window operations  
//! - **Script Commands**: Create new scripts and scriptlets
//! - **Permission Commands**: Accessibility permission management
//!

use crate::config::BuiltInConfig;
use crate::menu_bar::MenuBarItem;
use tracing::debug;

// ============================================================================
// Command Type Enums
// ============================================================================

/// System action types for macOS system commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemActionType {
    // Power management
    EmptyTrash,
    LockScreen,
    Sleep,
    Restart,
    ShutDown,
    LogOut,
```

## Built-in Entry Struct (src/builtins.rs lines 37-80)

```rust
    // UI controls
    ToggleDarkMode,
    ShowDesktop,
    MissionControl,
    Launchpad,
    ForceQuitApps,

    // Volume controls (preset levels)
    Volume0,
    Volume25,
    Volume50,
    Volume75,
    Volume100,
    VolumeMute,

    // Dev/test actions (only available in debug builds)
    #[cfg(debug_assertions)]
    TestConfirmation,

    // App control
    QuitScriptKit,

    // System utilities
    ToggleDoNotDisturb,
    StartScreenSaver,

    // System Preferences
    OpenSystemPreferences,
    OpenPrivacySettings,
    OpenDisplaySettings,
    OpenSoundSettings,
    OpenNetworkSettings,
    OpenKeyboardSettings,
    OpenBluetoothSettings,
    OpenNotificationsSettings,
}

/// Window action types for window management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowActionType {
    TileLeft,
    TileRight,
    TileTop,
    TileBottom,
```

## File Search Key Handler (src/render_builtins.rs lines 2046-2200)

Handles Enter, Cmd+Enter, Cmd+K, Cmd+Y, arrow keys in file search view.

```rust
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        // Use handle_action instead of trigger_action_by_name to support
                        // both built-in actions (open_file, quick_look, etc.) and SDK actions
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC goes back to main menu (not close window)
                if key_str == "escape" {
                    logging::log("KEY", "ESC in FileSearch - returning to main menu");
                    // Cancel any pending search
                    this.file_search_debounce_task = None;
                    this.file_search_loading = false;
                    // Clear cached results
                    this.cached_file_results.clear();
                    // Return to main menu
                    this.current_view = AppView::ScriptList;
                    this.filter_text.clear();
                    this.selected_index = 0;
                    // Sync input and reset placeholder to default
                    this.gpui_input_state.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                        // Ensure cursor is at start (empty string, so 0..0)
                        state.set_selection(0, 0, window, cx);
                        state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
                    });
                    this.update_window_size_deferred(window, cx);
                    cx.notify();
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query,
                    selected_index,
                } = &mut this.current_view
                {
                    // Apply filter to get current filtered list
                    // Use parse_directory_path to extract filter pattern
                    let filter_pattern =
                        if let Some(parsed) = crate::file_search::parse_directory_path(query) {
                            parsed.filter
                        } else if !query.is_empty() {
                            Some(query.clone())
                        } else {
                            None
                        };

                    // Use Nucleo fuzzy matching for filtering
                    let filtered_results: Vec<_> = if let Some(ref pattern) = filter_pattern {
                        crate::file_search::filter_results_nucleo_simple(
                            &this.cached_file_results,
                            pattern,
                        )
                    } else {
                        this.cached_file_results.iter().enumerate().collect()
                    };
                    let _filtered_len = filtered_results.len();

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    // Clone the file to avoid borrow issues
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
```

## Uniform List Rendering (src/render_builtins.rs lines 2213-2350)

Virtualized list rendering for file results.

```rust
        // Use uniform_list for virtualized scrolling
        // Note: Loading state with 0 results is handled by the main content section (full-width spinner)
        // This list_element is only used in the 50/50 split when we have results
        let list_element = if filtered_len == 0 {
            // No results and not loading
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_dimmed))
                .child(if query.is_empty() {
                    "Type to search files"
                } else {
                    "No files found"
                })
                .into_any_element()
        } else {
            uniform_list(
                "file-search-list",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some(file) = files_for_closure.get(ix) {
                                let is_selected = ix == current_selected;
                                // Use theme opacity for vibrancy-compatible selection
                                let bg = if is_selected {
                                    rgba((list_selected << 8) | selected_alpha)
                                } else {
                                    rgba(0x00000000)
                                };
                                let hover_bg = rgba((list_hover << 8) | hover_alpha);

                                div()
                                    .id(ix)
                                    .w_full()
                                    .h(px(52.))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .px(px(12.))
                                    .gap(px(12.))
                                    .bg(bg)
                                    .hover(move |s| s.bg(hover_bg))
                                    .child(
                                        div()
                                            .text_lg()
                                            .text_color(rgb(text_muted))
                                            .child(file_search::file_type_icon(file.file_type)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(text_primary))
                                                    .child(file.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(file_search::shorten_path(&file.path)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_end()
                                            .gap(px(2.))
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_file_size(file.size),
                                                ),
                                            )
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_relative_time(
                                                        file.modified,
                                                    ),
                                                ),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(52.))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.file_search_scroll_handle)
            .into_any_element()
        };

        // Build preview panel content - matching main menu labeled section pattern
        let preview_content = if let Some(file) = &selected_file {
            let file_type_str = match file.file_type {
                FileType::Directory => "Folder",
                FileType::Image => "Image",
                FileType::Audio => "Audio",
                FileType::Video => "Video",
                FileType::Document => "Document",
                FileType::Application => "Application",
                FileType::File => "File",
                FileType::Other => "File",
            };

            div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(design_spacing.padding_lg))
                .gap(px(design_spacing.gap_md))
                .overflow_y_hidden()
                // Name section (labeled like main menu)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Name"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
```

## Preview Panel Rendering (src/render_builtins.rs lines 2350-2500)

Right panel showing selected file details.

```rust
                                .items_center()
                                .gap(px(design_spacing.gap_sm))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(file.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(rgba((ui_border << 8) | 0x40))
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(file_type_str),
                                ),
                        ),
                )
                // Path section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Path"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(file.path.clone()),
                        ),
                )
                // Divider (like main menu)
                .child(
                    div()
                        .w_full()
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(design_spacing.padding_sm)),
                )
                // Details section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Details"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(design_spacing.gap_sm))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Size: {}",
                                    file_search::format_file_size(file.size)
                                )))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Modified: {}",
                                    file_search::format_relative_time(file.modified)
                                )))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_dimmed))
                                        .child(format!("Type: {}", file_type_str)),
                                ),
                        ),
                )
        } else if is_loading {
            // When loading, show empty preview (no distracting message)
            div().flex_1()
        } else {
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child("No file selected"),
            )
        };

        // Main container - styled to match main menu exactly
        // NOTE: No border to match main menu (border adds visual padding/shift)
        div()
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            .shadow(box_shadows)
            .rounded(px(design_visual.radius_lg))
            // Header with search input - styled to match main menu exactly
            // Uses shared header constants (HEADER_PADDING_X/Y, CURSOR_HEIGHT_LG) for visual consistency.
            // The right-side element uses same py(4px) padding as main menu's "Ask AI" button
            // to ensure identical flex row height (28px) and input vertical centering.
            .child({
                // Calculate input height using same formula as main menu
                let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);

                div()
                    .w_full()
                    .px(px(HEADER_PADDING_X))
                    .py(px(HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(HEADER_GAP))
                    // Search input - matches main menu Input styling for visual consistency
                    // NOTE: Removed search icon to match main menu alignment exactly
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(input_height))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(_design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    // Right-side element styled to match main menu's "Ask AI" button height
                    // Using fixed width to prevent layout shift when content changes
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .py(px(4.))
                            .w(px(70.)) // Fixed width prevents layout shift
                            .child(
                                div()
                                    .text_sm()
```

## File Search Actions Integration (src/app_impl.rs lines 808-870)

Actions popup handling in FileSearchView.

```rust
        // Add interceptor for actions popup in FileSearchView
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Only handle when in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            return;
                        }

                        // Handle Cmd+K to toggle actions popup
                        if has_cmd && key == "k" {
                            if let AppView::FileSearchView {
                                selected_index,
                                query,
                            } = &mut this.current_view
                            {
                                // Get the filter pattern for directory path parsing
                                let filter_pattern = if let Some(parsed) =
                                    crate::file_search::parse_directory_path(query)
                                {
                                    parsed.filter
                                } else if !query.is_empty() {
                                    Some(query.clone())
                                } else {
                                    None
                                };

                                let filtered_results: Vec<_> =
                                    if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                    } else {
                                        this.cached_file_results.iter().enumerate().collect()
                                    };

                                // Defensive bounds check: clamp selected_index if out of bounds
                                let filtered_len = filtered_results.len();
                                if filtered_len > 0 && *selected_index >= filtered_len {
                                    *selected_index = filtered_len - 1;
                                }

                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Only handle remaining keys if actions popup is open
                        if !this.show_actions_popup {
                            return;
```

## Actions Builder for File Search (src/actions/builders.rs relevant section)

```rust
// build_file_actions not found in builders.rs
```

## File Actions Builder (src/actions/builders.rs)

```rust
```

## Cargo.toml - Relevant Dependencies

```toml
nucleo-matcher = "0.3"         # High-performance fuzzy matching (10-100x faster than bespoke)

# From Cargo.lock - transitive dependencies:
name = "ignore"
version = "0.4.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3d782a365a015e0f5c04902246139249abf769125006fbe7649e2ee88169b4a"
name = "walkdir"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b"
name = "globwalk"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93e3af942408868f6934a7b85134a3230832b9977cf66125df2f9edcfce4ddcc"
```

## Protocol Message Types for File Search (src/protocol/types.rs)

```rust
/// File search result entry
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileSearchResultEntry {
    pub path: String,
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

/// Element type for UI element querying (getElements)
///
/// # Forward Compatibility
/// The `Unknown` variant with `#[serde(other)]` ensures forward compatibility:
/// if a newer protocol version adds new element types, older receivers
/// will deserialize them as `Unknown` instead of failing entirely.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    Choice,
```

## Full Built-in Entries Definition (src/builtins.rs lines 900-1000)

All built-in entries including file search.

```rust
        "Quick editor for notes and code - auto-saves to disk",
        vec![
            "scratch",
            "pad",
            "scratchpad",
            "notes",
            "editor",
            "write",
            "text",
            "quick",
            "jot",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::ScratchPad),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quick-terminal",
        "Quick Terminal",
        "Open a terminal for running quick commands",
        vec![
            "terminal", "term", "shell", "bash", "zsh", "command", "quick", "console", "cli",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::QuickTerminal),
        "💻",
    ));

    // =========================================================================
    // File Search (Directory Navigation)
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-file-search",
        "Search Files",
        "Browse directories and search for files",
        vec![
            "file",
            "search",
            "find",
            "directory",
            "folder",
            "browse",
            "navigate",
            "path",
            "open",
            "explorer",
        ],
        BuiltInFeature::FileSearch,
        "folder-search",
    ));

    debug!(count = entries.len(), "Built-in entries loaded");
    entries
}

// ============================================================================
// Menu Bar Item Conversion
// ============================================================================

/// Convert menu bar items to built-in entries for search
///
/// This flattens the menu hierarchy into searchable entries, skipping the
/// Apple menu (first item) and only including leaf items (no submenus).
///
/// # Arguments
/// * `items` - The menu bar items from the frontmost application
/// * `bundle_id` - The bundle identifier of the application (e.g., "com.apple.Safari")
/// * `app_name` - The display name of the application (e.g., "Safari")
///
/// # Returns
/// A vector of `BuiltInEntry` items that can be added to search results
#[allow(dead_code)] // Will be used when menu bar integration is complete
pub fn menu_bar_items_to_entries(
    items: &[MenuBarItem],
    bundle_id: &str,
    app_name: &str,
) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    // Skip first item (Apple menu)
    for item in items.iter().skip(1) {
        flatten_menu_item(item, bundle_id, app_name, &[], &mut entries);
    }

    debug!(
        count = entries.len(),
        bundle_id = bundle_id,
        app_name = app_name,
        "Menu bar items converted to entries"
    );
    entries
}

/// Recursively flatten a menu item and its children into entries
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn flatten_menu_item(
    item: &MenuBarItem,
    bundle_id: &str,
    app_name: &str,
    parent_path: &[String],
    entries: &mut Vec<BuiltInEntry>,
```

## Handle Built-in Feature Execution (src/app_execute.rs lines 995-1050)

```rust
    // =========================================================================
    // File Search Implementation
    // =========================================================================
    //
    // BLOCKED: Requires the following changes to main.rs (not in worker reservations):
    //
    // 1. Add to AppView enum:
    //    ```rust
    //    /// Showing file search results (Spotlight/mdfind based)
    //    FileSearchView {
    //        query: String,
    //        selected_index: usize,
    //    },
    //    ```
    //
    // 2. Add to ScriptListApp struct:
    //    ```rust
    //    /// Cached file search results
    //    cached_file_results: Vec<file_search::FileResult>,
    //    /// Scroll handle for file search list
    //    file_search_scroll_handle: UniformListScrollHandle,
    //    ```
    //
    // 3. Add initialization in app_impl.rs ScriptListApp::new():
    //    ```rust
    //    cached_file_results: Vec::new(),
    //    file_search_scroll_handle: UniformListScrollHandle::new(),
    //    ```
    //
    // 4. Add render call in main.rs Render impl match arm:
    //    ```rust
    //    AppView::FileSearchView { query, selected_index } => {
    //        self.render_file_search(query.clone(), *selected_index, cx)
    //    }
    //    ```
    //
    // 5. Wire up in app_impl.rs execute_fallback():
    //    ```rust
    //    FallbackResult::SearchFiles { query } => {
    //        self.open_file_search(query, cx);
    //    }
    //    ```
    //
    // Once those are added, uncomment the method below.
    // =========================================================================

    /// Open file search with the given query
    ///
    /// This performs an mdfind-based file search and displays results in a Raycast-like UI.
    ///
    /// # Arguments
    /// * `query` - The search query (passed from the "Search Files" fallback action)
    ///
    /// # Usage
    /// Called when user selects "Search Files" fallback with a search term.
    /// Features:
```

## App Initialization - File Search State (src/app_impl.rs lines 235-250)

```rust
            // .measure_all() ensures all items are measured upfront for correct scroll height
            main_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
            list_scroll_handle: UniformListScrollHandle::new(),
            arg_list_scroll_handle: UniformListScrollHandle::new(),
            clipboard_list_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            file_search_scroll_handle: UniformListScrollHandle::new(),
            file_search_loading: false,
            file_search_debounce_task: None,
            file_search_current_dir: None,
            file_search_actions_path: None,
            show_actions_popup: false,
            actions_dialog: None,
            cursor_visible: true,
            focused_input: FocusedInput::MainFilter,
```

## Complete Built-ins List Definition (src/builtins.rs lines 800-950)

```rust
    ));

    // =========================================================================
    // Script Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-script",
        "New Script",
        "Create a new Script Kit script",
        vec!["new", "script", "create", "code"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewScript),
        "📜",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-extension",
        "New Extension",
        "Create a new Script Kit extension",
        vec!["new", "extension", "create", "snippet"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewExtension),
        "✨",
    ));

    // =========================================================================
    // Permission Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-check-permissions",
        "Check Permissions",
        "Check all required macOS permissions",
        vec!["check", "permissions", "accessibility", "privacy"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::CheckPermissions),
        "✅",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-request-accessibility",
        "Request Accessibility Permission",
        "Request accessibility permission for Script Kit",
        vec!["request", "accessibility", "permission"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::RequestAccessibility),
        "🔑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-accessibility-settings",
        "Open Accessibility Settings",
        "Open Accessibility settings in System Preferences",
        vec!["accessibility", "settings", "permission", "open"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::OpenAccessibilitySettings),
        "♿",
    ));

    // =========================================================================
    // Frecency/Suggested Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-clear-suggested",
        "Clear Suggested",
        "Clear all suggested/recently used items",
        vec![
            "clear",
            "suggested",
            "recent",
            "frecency",
            "reset",
            "history",
        ],
        BuiltInFeature::FrecencyCommand(FrecencyCommandType::ClearSuggested),
        "🧹",
    ));

    // =========================================================================
    // Settings Commands
    // =========================================================================

    // Only show reset if there are custom positions
    if crate::window_state::has_custom_positions() {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-reset-window-positions",
            "Reset Window Positions",
            "Restore all windows to default positions",
            vec![
                "reset", "window", "position", "default", "restore", "layout", "location",
            ],
            BuiltInFeature::SettingsCommand(SettingsCommandType::ResetWindowPositions),
            "🔄",
        ));
    }

    // =========================================================================
    // Utility Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-scratch-pad",
        "Scratch Pad",
        "Quick editor for notes and code - auto-saves to disk",
        vec![
            "scratch",
            "pad",
            "scratchpad",
            "notes",
            "editor",
            "write",
            "text",
            "quick",
            "jot",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::ScratchPad),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quick-terminal",
        "Quick Terminal",
        "Open a terminal for running quick commands",
        vec![
            "terminal", "term", "shell", "bash", "zsh", "command", "quick", "console", "cli",
        ],
        BuiltInFeature::UtilityCommand(UtilityCommandType::QuickTerminal),
        "💻",
    ));

    // =========================================================================
    // File Search (Directory Navigation)
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-file-search",
        "Search Files",
        "Browse directories and search for files",
        vec![
            "file",
            "search",
            "find",
            "directory",
            "folder",
            "browse",
            "navigate",
            "path",
            "open",
            "explorer",
        ],
        BuiltInFeature::FileSearch,
        "folder-search",
    ));

```

## File Search Toggle Actions (src/app_impl.rs toggle_file_search_actions)

```rust
// toggle_file_search_actions not found
```

## Complete App Execute Entry Points (src/app_execute.rs lines 1-100)

```rust
// App execution methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: execute_builtin, execute_app, execute_window_focus

impl ScriptListApp {
    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Executing built-in: {} (id: {})", entry.name, entry.id),
        );

        // Check if this command requires confirmation
        if self.config.requires_confirmation(&entry.id) {
            // Check if we're already in confirmation mode for this entry
            if self.pending_confirmation.as_ref() == Some(&entry.id) {
                // User confirmed - clear pending and proceed with execution
                logging::log("EXEC", &format!("Confirmed: {}", entry.id));
                self.pending_confirmation = None;
                // Fall through to execute
            } else {
                // First press - enter confirmation mode
                logging::log("EXEC", &format!("Awaiting confirmation: {}", entry.id));
                self.pending_confirmation = Some(entry.id.clone());
                cx.notify();
                return; // Don't execute yet
            }
        }

        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                logging::log("EXEC", "Opening Clipboard History");
                // P0 FIX: Store data in self, view holds only state
                self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);
                logging::log(
                    "EXEC",
                    &format!(
                        "Loaded {} clipboard entries (cached)",
                        self.cached_clipboard_entries.len()
                    ),
                );
                // Clear the shared input for fresh search (sync on next render)
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search clipboard history...".to_string());
                // Initial selected_index should be 0 (first entry)
                // Note: clipboard history uses a flat list without section headers
                self.current_view = AppView::ClipboardHistoryView {
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for clipboard history view
                resize_to_view_sync(ViewType::ScriptList, 0);
                // Focus the main filter input so cursor blinks and typing works
                self.pending_focus = Some(FocusTarget::MainFilter);
                self.focused_input = FocusedInput::MainFilter;
                cx.notify();
            }
            builtins::BuiltInFeature::AppLauncher => {
                logging::log("EXEC", "Opening App Launcher");
                // P0 FIX: Use self.apps which is already cached
                // Refresh apps list when opening launcher
                self.apps = app_launcher::scan_applications().clone();
                logging::log("EXEC", &format!("Loaded {} applications", self.apps.len()));
                // Clear the shared input for fresh search (sync on next render)
                self.filter_text = String::new();
                self.pending_filter_sync = true;
                self.pending_placeholder = Some("Search applications...".to_string());
                self.current_view = AppView::AppLauncherView {
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for app launcher view
                resize_to_view_sync(ViewType::ScriptList, 0);
                // Focus the main filter input so cursor blinks and typing works
                self.pending_focus = Some(FocusTarget::MainFilter);
                self.focused_input = FocusedInput::MainFilter;
                cx.notify();
            }
            builtins::BuiltInFeature::App(app_name) => {
                logging::log("EXEC", &format!("Launching app: {}", app_name));
                // Find and launch the specific application
                let apps = app_launcher::scan_applications();
                if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
                    if let Err(e) = app_launcher::launch_application(app) {
                        logging::log("ERROR", &format!("Failed to launch {}: {}", app_name, e));
                        self.last_output = Some(SharedString::from(format!(
                            "Failed to launch: {}",
                            app_name
                        )));
                    } else {
                        logging::log("EXEC", &format!("Launched app: {}", app_name));
                        self.close_and_reset_window(cx);
                    }
                } else {
                    logging::log("ERROR", &format!("App not found: {}", app_name));
                    self.last_output =
                        Some(SharedString::from(format!("App not found: {}", app_name)));
                }
                cx.notify();
            }
```

## Full Actions Dialog Key Routing (src/app_impl.rs lines 808-950)

```rust
        // Add interceptor for actions popup in FileSearchView
        // This handles Cmd+K (toggle), Escape (close), Enter (submit), and typing
        let app_entity_for_actions = cx.entity().downgrade();
        let actions_interceptor = cx.intercept_keystrokes({
            let app_entity = app_entity_for_actions;
            move |event, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;
                let key_char = event.keystroke.key_char.as_deref();

                if let Some(app) = app_entity.upgrade() {
                    app.update(cx, |this, cx| {
                        // Only handle when in FileSearchView with actions popup open
                        if !matches!(this.current_view, AppView::FileSearchView { .. }) {
                            return;
                        }

                        // Handle Cmd+K to toggle actions popup
                        if has_cmd && key == "k" {
                            if let AppView::FileSearchView {
                                selected_index,
                                query,
                            } = &mut this.current_view
                            {
                                // Get the filter pattern for directory path parsing
                                let filter_pattern = if let Some(parsed) =
                                    crate::file_search::parse_directory_path(query)
                                {
                                    parsed.filter
                                } else if !query.is_empty() {
                                    Some(query.clone())
                                } else {
                                    None
                                };

                                let filtered_results: Vec<_> =
                                    if let Some(ref pattern) = filter_pattern {
                                        crate::file_search::filter_results_nucleo_simple(
                                            &this.cached_file_results,
                                            pattern,
                                        )
                                    } else {
                                        this.cached_file_results.iter().enumerate().collect()
                                    };

                                // Defensive bounds check: clamp selected_index if out of bounds
                                let filtered_len = filtered_results.len();
                                if filtered_len > 0 && *selected_index >= filtered_len {
                                    *selected_index = filtered_len - 1;
                                }

                                if let Some((_, file)) = filtered_results.get(*selected_index) {
                                    let file_clone = (*file).clone();
                                    this.toggle_file_search_actions(&file_clone, window, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Only handle remaining keys if actions popup is open
                        if !this.show_actions_popup {
                            return;
                        }

                        // Handle Escape to close actions popup
                        if key == "escape" {
                            this.close_actions_popup(ActionsDialogHost::FileSearch, window, cx);
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Enter to submit selected action
                        if key == "enter" {
                            if let Some(ref dialog) = this.actions_dialog {
                                let action_id = dialog.read(cx).get_selected_action_id();
                                let should_close = dialog.read(cx).selected_action_should_close();

                                if let Some(action_id) = action_id {
                                    crate::logging::log(
                                        "ACTIONS",
                                        &format!(
                                            "FileSearch actions executing action: {} (close={})",
                                            action_id, should_close
                                        ),
                                    );

                                    if should_close {
                                        this.close_actions_popup(
                                            ActionsDialogHost::FileSearch,
                                            window,
                                            cx,
                                        );
                                    }

                                    // Use handle_action instead of trigger_action_by_name
                                    // handle_action supports both built-in actions (open_file, quick_look, etc.)
                                    // and SDK actions
                                    this.handle_action(action_id, cx);
                                }
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle Backspace for actions search
                        if key == "backspace" {
                            if let Some(ref dialog) = this.actions_dialog {
                                dialog.update(cx, |d, cx| d.handle_backspace(cx));
                                crate::actions::notify_actions_window(cx);
                                crate::actions::resize_actions_window(cx, dialog);
                            }
                            cx.stop_propagation();
                            return;
                        }

                        // Handle printable character input for actions search
                        if let Some(chars) = key_char {
                            if let Some(ch) = chars.chars().next() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    if let Some(ref dialog) = this.actions_dialog {
                                        dialog.update(cx, |d, cx| d.handle_char(ch, cx));
                                        crate::actions::notify_actions_window(cx);
                                        crate::actions::resize_actions_window(cx, dialog);
                                    }
                                    cx.stop_propagation();
                                }
                            }
                        }
                    });
                }
            }
        });
        app.gpui_input_subscriptions.push(actions_interceptor);

        // CRITICAL FIX: Sync list state on initialization
        // This was removed when state mutations were moved out of render(),
        // but we still need to sync once during initialization so the list
        // knows about the scripts that were loaded.
        // Without this, the first render shows "No scripts or snippets found"
        // because main_list_state starts with 0 items.
        app.sync_list_state();
        app.validate_selection_bounds(cx);
```

---

## Implementation Guide

### Option 1: Replace mdfind with fd (Recommended for Quick Win)

The `fd` command is already installed and is dramatically faster (23ms vs 5s+).

#### Step 1: Add fd-based search function

```rust
// File: src/file_search.rs
// Add after the existing search_files function

/// Search files using fd command (much faster than mdfind)
/// Falls back to mdfind if fd is not available
#[instrument(skip_all, fields(query = %query, limit = limit))]
pub fn search_files_fd(query: &str, limit: usize) -> Vec<FileResult> {
    if query.is_empty() {
        return Vec::new();
    }
    
    let mut cmd = Command::new("fd");
    cmd.arg("--type").arg("f")  // files only (use "d" for dirs, omit for both)
        .arg("--max-results").arg(limit.to_string())
        .arg("--color").arg("never")
        .arg(query);  // fd uses regex by default, glob with -g
    
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    
    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => {
            warn!(error = %e, "fd not available, falling back to mdfind");
            return search_files(query, None, limit);
        }
    };
    
    if !output.status.success() {
        return search_files(query, None, limit);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .take(limit)
        .filter_map(|line| {
            let path = Path::new(line);
            if !path.exists() { return None; }
            
            let (size, modified) = std::fs::metadata(path)
                .map(|m| {
                    let size = m.len();
                    let modified = m.modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    (size, modified)
                })
                .unwrap_or((0, 0));
            
            Some(FileResult {
                path: line.to_string(),
                name: path.file_name()?.to_str()?.to_string(),
                size,
                modified,
                file_type: detect_file_type(path),
            })
        })
        .collect()
}
```

#### Step 2: Update app_impl.rs to use fd

```rust
// File: src/app_impl.rs
// In the FileSearchView input handling section (~line 2173)
// Replace:
let results = crate::file_search::search_files(
    &query_for_thread,
    None,
    crate::file_search::DEFAULT_SEARCH_LIMIT,
);
// With:
let results = crate::file_search::search_files_fd(
    &query_for_thread,
    crate::file_search::DEFAULT_SEARCH_LIMIT,
);
```

### Option 2: Use ignore crate for Rust-native streaming (Better Long-term)

Add the `ignore` crate for ripgrep-style parallel directory traversal.

#### Step 1: Add dependency to Cargo.toml

```toml
[dependencies]
ignore = "0.4"
```

#### Step 2: Create streaming search with ignore crate

```rust
// File: src/file_search.rs

use ignore::WalkBuilder;
use std::sync::mpsc;

/// Stream-based file search using the ignore crate
/// Sends results through channel as they're found
pub fn search_files_streaming(
    query: &str,
    tx: mpsc::Sender<FileResult>,
    limit: usize,
) {
    if query.is_empty() {
        return;
    }
    
    let query_lower = query.to_lowercase();
    let mut count = 0;
    
    // Start from home directory
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    
    let walker = WalkBuilder::new(&home)
        .hidden(true)  // skip hidden by default
        .git_ignore(true)  // respect .gitignore
        .threads(num_cpus::get())
        .build_parallel();
    
    walker.run(|| {
        let tx = tx.clone();
        let query_lower = query_lower.clone();
        let count = std::sync::atomic::AtomicUsize::new(0);
        
        Box::new(move |entry| {
            if count.load(std::sync::atomic::Ordering::Relaxed) >= limit {
                return ignore::WalkState::Quit;
            }
            
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return ignore::WalkState::Continue,
            };
            
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Simple substring match (could use Nucleo here)
            if name.to_lowercase().contains(&query_lower) {
                if let Ok(meta) = entry.metadata() {
                    let result = FileResult {
                        path: path.to_string_lossy().to_string(),
                        name: name.to_string(),
                        size: meta.len(),
                        modified: meta.modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                        file_type: detect_file_type(path),
                    };
                    
                    if tx.send(result).is_err() {
                        return ignore::WalkState::Quit;
                    }
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            
            ignore::WalkState::Continue
        })
    });
}
```

### Testing the Changes

1. Build and run:
```bash
cargo build
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

2. Open file search (type "files" in main menu or trigger builtin)

3. Type a query like "mp4" and verify:
   - Results appear quickly (< 1 second)
   - No "Searching..." hang
   - Proper "No files found" for non-matching queries

---

## Instructions for the Next AI Agent

You are tasked with improving the file search implementation in Script Kit GPUI. The current implementation uses macOS Spotlight (mdfind) which can hang indefinitely.

### Context
- The core search module is in `src/file_search.rs`
- A 5-second timeout was added to `search_files()` but this is a band-aid
- The user has `fd` installed (23ms vs 5s+ for searches)
- The codebase already has `walkdir`, `ignore`, `globwalk` as transitive dependencies

### Your Options
1. **Quick fix**: Replace `search_files` calls with fd-based search (see Implementation Guide Option 1)
2. **Better solution**: Add streaming search with the `ignore` crate (see Option 2)
3. **Hybrid**: Use fd for interactive search, keep mdfind for metadata searches

### Key Files to Modify
1. `src/file_search.rs` - Add new search function(s)
2. `src/app_impl.rs` (~line 2173) - Update which search function is called
3. `Cargo.toml` - Add `ignore` dep if using Option 2

### Testing Protocol
```bash
cargo build
echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### Verification Gate (run before committing)
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Success Criteria
- File search returns results in < 1 second for common queries
- "No files found" appears quickly for non-matching queries (no 5s wait)
- Directory browsing still works (Tab to enter, Shift+Tab to go up)
- All existing tests pass
