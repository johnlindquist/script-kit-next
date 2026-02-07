fn terminal_working_directory(path: &str, is_dir: bool) -> String {
    if is_dir {
        path.to_string()
    } else {
        std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    }
}
/// Open a terminal window at the target path.
///
/// Returns the resolved working directory used to launch the terminal.
pub fn open_in_terminal(path: &str, is_dir: bool) -> Result<String, String> {
    let dir_path = terminal_working_directory(path, is_dir);

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let escaped_dir_path = crate::utils::escape_applescript_string(&dir_path);
        let script = format!(
            r#"tell application "Terminal"
                do script "cd " & quoted form of "{}"
                activate
            end tell"#,
            escaped_dir_path
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
        Ok(dir_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        let _ = is_dir;
        Err("Open in Terminal is currently only supported on macOS".to_string())
    }
}
/// Move a path to Trash.
pub fn move_to_trash(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let escaped_path = crate::utils::escape_applescript_string(path);
        let script = format!(
            r#"tell application "Finder"
                delete POSIX file "{}"
            end tell"#,
            escaped_path
        );

        let mut child = Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to spawn trash command: {}", e))?;

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for trash command: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Trash command exited with status: {}", status))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("Move to Trash is currently only supported on macOS".to_string())
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
            crate::utils::escape_applescript_string(path)
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
            crate::utils::escape_applescript_string(path)
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
