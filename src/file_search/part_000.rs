use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tracing::{debug, instrument, warn};
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

    // Take stdout for streaming reads
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            warn!("Failed to capture mdfind stdout");
            let _ = child.kill();
            let _ = child.wait();
            return Vec::new();
        }
    };

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

    // Clean up the child process
    // If we stopped early (hit limit), kill the process
    if results.len() >= limit {
        let _ = child.kill();
    }
    // Wait for process to fully exit (prevents zombies)
    let _ = child.wait();

    debug!(result_count = results.len(), "Search completed");
    results
}
// ============================================================================
// Streaming Search API (for real-time UX)
// ============================================================================

/// Events emitted during streaming search
#[derive(Debug, Clone)]
pub enum SearchEvent {
    /// A new file result was found
    Result(FileResult),
    /// Search completed (either finished or cancelled)
    Done,
}
/// Cancel token for streaming searches
///
/// Set to `true` to cancel an in-flight search.
/// The search thread will check this token and stop early.
pub type CancelToken = Arc<AtomicBool>;
/// Create a new cancel token
pub fn new_cancel_token() -> CancelToken {
    Arc::new(AtomicBool::new(false))
}
