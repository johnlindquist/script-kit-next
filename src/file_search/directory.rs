use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::UNIX_EPOCH;

use tracing::{debug, instrument, warn};

use super::mdfind::{CancelToken, SearchEvent};
use super::{detect_file_type, FileResult, FileType};

/// Internal cap to prevent runaway directory listings
const MAX_DIRECTORY_ENTRIES: usize = 5000;

/// Streaming directory listing: yields results as they're read.
///
/// Similar to `search_files_streaming` but for directory contents.
/// Useful for large directories where you want progressive loading.
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `cancel` - Cancel token
/// * `skip_metadata` - If true, skip stat() calls (size/modified = 0)
/// * `on_event` - Callback for each result
#[instrument(skip_all, fields(dir_path = %dir_path, skip_metadata = skip_metadata))]
pub fn list_directory_streaming<F>(
    dir_path: &str,
    cancel: CancelToken,
    skip_metadata: bool,
    mut on_event: F,
) where
    F: FnMut(SearchEvent),
{
    // Expand the path
    let expanded = match expand_path(dir_path) {
        Some(p) => p,
        None => {
            debug!("Failed to expand path: {}", dir_path);
            on_event(SearchEvent::Done);
            return;
        }
    };

    let path = Path::new(&expanded);
    if !path.is_dir() {
        debug!("Path is not a directory: {}", expanded);
        on_event(SearchEvent::Done);
        return;
    }

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "Failed to read directory: {}", expanded);
            on_event(SearchEvent::Done);
            return;
        }
    };

    let mut count = 0usize;

    for entry in entries.flatten() {
        // Check cancellation
        if cancel.load(Ordering::Relaxed) {
            debug!("Directory listing cancelled");
            break;
        }

        // Internal cap
        if count >= MAX_DIRECTORY_ENTRIES {
            debug!("Hit internal cap {}", MAX_DIRECTORY_ENTRIES);
            break;
        }

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

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let (size, modified) = if skip_metadata {
            (0, 0)
        } else {
            std::fs::metadata(&entry_path)
                .map(|m| {
                    (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0))
        };

        let file_type = detect_file_type(&entry_path);

        on_event(SearchEvent::Result(FileResult {
            path: path_str,
            name,
            size,
            modified,
            file_type,
        }));
        count += 1;
    }

    debug!(result_count = count, "Directory listing completed");
    on_event(SearchEvent::Done);
}

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

/// List contents of a directory
///
/// Returns files and directories sorted with directories first, then by name.
/// Handles ~ expansion and relative paths.
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `limit` - Maximum number of results to return (clamped to internal cap)
///
/// # Returns
/// Vector of FileResult structs for directory contents
#[instrument(skip_all, fields(dir_path = %dir_path, limit = limit))]
pub fn list_directory(dir_path: &str, limit: usize) -> Vec<FileResult> {
    debug!("Starting directory listing");

    let effective_limit = limit.min(MAX_DIRECTORY_ENTRIES);

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

    results.truncate(effective_limit);

    debug!(result_count = results.len(), "Directory listing completed");
    results
}

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
    if !crate::scripts::input_detection::is_directory_path(trimmed) {
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
    // First get additional entries so filtering can still return enough matches.
    let mut results = list_directory(dir_path, limit.saturating_mul(2));

    // Apply filter if present
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        results.retain(|r| r.name.to_lowercase().contains(&filter_lower));
    }

    // Apply limit after filtering
    results.truncate(limit);
    results
}
