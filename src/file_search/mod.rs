//! File Search Module using macOS Spotlight (mdfind)
//!
//! This module provides file search functionality using macOS's mdfind command,
//! which interfaces with the Spotlight index for fast file searching.
//!
//! # Streaming API
//!
//! For real-time search UX, use `search_files_streaming()` with a cancel token.
//! This allows:
//! - Cancellation of in-flight searches when query changes
//! - Batched UI updates without blocking on full results
//! - Proper cleanup of mdfind processes
//!
//! # Performance Notes
//!
//! - Metadata (size, modified) is fetched per-result which can be slow
//! - For faster "time to first result", consider skipping metadata in streaming mode
//!   and hydrating it lazily for visible rows only

// --- merged from part_000.rs ---
use std::path::Path;
use std::time::UNIX_EPOCH;
use tracing::{debug, instrument};
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
/// Maximum Spotlight results collected for root launcher file rows.
pub const ROOT_FILE_SOURCE_LIMIT: usize = 24;
/// Maximum root launcher file rows rendered under the Files section.
pub const ROOT_FILE_RENDER_LIMIT: usize = 6;
/// Maximum frecency-backed recent file rows rendered on the empty root launcher.
pub const ROOT_FILE_RECENT_LIMIT: usize = ROOT_FILE_RENDER_LIMIT;
/// Maximum directory children collected for root launcher directory browsing.
pub const ROOT_FILE_BROWSE_SOURCE_LIMIT: usize = 96;
/// Maximum directory children rendered for root launcher directory browsing.
pub const ROOT_FILE_BROWSE_RENDER_LIMIT: usize = 12;
/// Minimum visible query length before root launcher file search starts.
pub const ROOT_FILE_MIN_QUERY_CHARS: usize = 3;

/// Which source currently backs the root launcher's `Files` section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootFileSectionMode {
    /// Global filename search backed by Spotlight.
    GlobalQuery,
    /// Direct child listing for an explicit directory path query.
    DirectoryBrowse,
}
/// Check if the query looks like an advanced mdfind query (with operators)
/// If so, pass it through directly; otherwise wrap as filename query
pub(crate) fn looks_like_advanced_mdquery(q: &str) -> bool {
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

/// Returns true when the root launcher should ask Spotlight for file rows.
pub fn should_search_root_files(query: &str) -> bool {
    let q = query.trim();
    q.chars().count() >= ROOT_FILE_MIN_QUERY_CHARS
        && !looks_like_advanced_mdquery(q)
        && !is_directory_path(q)
}

/// Returns true when the root launcher query is syntactically a directory browse.
///
/// This is intentionally syntax-only so grouping/ranking code can decide layout
/// without touching the filesystem. Provider code still validates existence via
/// `parse_directory_path` before collecting rows.
pub fn looks_like_root_directory_browse_query(query: &str) -> bool {
    let q = query.trim();
    !q.is_empty()
        && (q.starts_with('/')
            || q == "~"
            || q.starts_with("~/")
            || q.starts_with("./")
            || q.starts_with("../"))
        && !looks_like_advanced_mdquery(q)
}

/// Return the folder portion of a root directory-browse query without reading the filesystem.
pub fn root_directory_query_base(query: &str) -> Option<String> {
    let q = query.trim();
    if !looks_like_root_directory_browse_query(q) {
        return None;
    }
    if q == "~" || q == "~/" {
        return Some("~/".to_string());
    }
    if q.ends_with('/') {
        return Some(q.to_string());
    }
    let last_slash = q.rfind('/')?;
    Some(q[..=last_slash].to_string())
}

/// Return the provider identity for a root directory-browse query.
///
/// The visible query may include a child fragment after the final slash, but
/// the source provider is only the containing directory plus hidden-file mode.
pub fn root_directory_browse_source_key(query: &str) -> Option<(String, bool)> {
    let parsed = parse_directory_path(query)?;
    Some((parsed.directory, parsed.show_hidden))
}

/// Returns the root file section mode implied by a query's syntax.
pub fn root_file_section_mode_for_query(query: &str) -> Option<RootFileSectionMode> {
    if should_search_root_files(query) {
        Some(RootFileSectionMode::GlobalQuery)
    } else if looks_like_root_directory_browse_query(query) {
        Some(RootFileSectionMode::DirectoryBrowse)
    } else {
        None
    }
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

mod directory;
mod mdfind;
mod os_open;

pub use crate::scripts::input_detection::is_directory_path;
#[allow(unused_imports)]
pub use directory::{
    ensure_trailing_slash, expand_path, list_directory, list_directory_filtered,
    list_directory_streaming, list_directory_streaming_with_options, list_directory_with_options,
    parent_dir_display, parse_directory_path, shorten_path, ParsedDirPath,
};
pub use mdfind::{
    new_cancel_token, search_files, search_files_streaming, search_files_streaming_with_options,
    CancelToken, SearchEvent, SearchFilesStreamingOptions,
};
pub use os_open::{
    duplicate_path, move_path, move_to_trash, open_file, open_with, prompt_move_destination_dir,
    prompt_rename_target_name, quick_look, rename_path, reveal_in_finder, show_info,
};

pub fn parent_folder_search_query(path: &str) -> Option<String> {
    let parent = Path::new(path).parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    let parent = parent.to_str()?;
    Some(ensure_trailing_slash(parent))
}

/// Build a root-launcher file result from live metadata for a previously seen path.
///
/// Recent root files are frecency-backed, not search-backed, so this helper only
/// hydrates known paths and filters out app bundles that belong to app search.
pub fn file_result_from_existing_path(path: &str) -> Option<FileResult> {
    let metadata = get_file_metadata(path)?;
    if metadata.file_type == FileType::Application {
        return None;
    }

    Some(FileResult {
        path: metadata.path,
        name: metadata.name,
        size: metadata.size,
        modified: metadata.modified,
        file_type: metadata.file_type,
    })
}

/// Convert directory browse results into root-launcher file matches.
pub fn root_directory_file_matches(
    results: &[FileResult],
    child_filter: Option<&str>,
    limit: usize,
) -> Vec<crate::scripts::FileMatch> {
    let filter = child_filter
        .map(str::trim)
        .filter(|filter| !filter.is_empty());

    let Some(filter) = filter else {
        return results
            .iter()
            .take(limit)
            .enumerate()
            .map(|(rank, file)| crate::scripts::FileMatch {
                file: file.clone(),
                score: i32::MAX.saturating_sub(rank as i32),
            })
            .collect();
    };

    let q = filter.to_lowercase();
    let mut nucleo = crate::scripts::NucleoCtx::new(&q);
    let mut ranked: Vec<_> = results
        .iter()
        .filter_map(|file| {
            let stem = Path::new(&file.name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(&file.name);
            let name_score = nucleo.score(&file.name);
            let stem_score = nucleo.score(stem);
            let score = name_score.max(stem_score)?;
            let text_tier = root_file_name_relevance_tier(&file.name, &q, true);
            if text_tier < 3 {
                return None;
            }

            Some(crate::scripts::FileMatch {
                file: file.clone(),
                score: text_tier
                    .saturating_mul(ROOT_FILE_TEXT_TIER_MULTIPLIER)
                    .saturating_add(score.min(10_000) as i32),
            })
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.file.name.cmp(&b.file.name))
            .then_with(|| a.file.path.cmp(&b.file.path))
    });
    ranked.truncate(limit);
    ranked
}

const ROOT_FILE_TEXT_TIER_MULTIPLIER: i32 = 20_000;

fn root_file_name_relevance_tier(name: &str, query: &str, name_matched: bool) -> i32 {
    let name_lc = name.to_lowercase();
    let stem_lc = Path::new(name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name)
        .to_lowercase();

    if name_lc == query || stem_lc == query {
        return 6;
    }
    if name_lc.starts_with(query) || stem_lc.starts_with(query) {
        return 5;
    }
    if contains_at_root_file_boundary(&name_lc, query)
        || contains_at_root_file_boundary(&stem_lc, query)
    {
        return 4;
    }
    if name_lc.contains(query) || stem_lc.contains(query) {
        return 3;
    }
    if name_matched {
        return 2;
    }
    1
}

fn contains_at_root_file_boundary(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.match_indices(needle).any(|(idx, _)| {
        idx == 0
            || haystack[..idx]
                .chars()
                .next_back()
                .map(is_root_file_boundary_char)
                .unwrap_or(false)
    })
}

fn is_root_file_boundary_char(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '-' | '_' | '.' | '/' | '(' | ')' | '[' | ']' | '{' | '}'
    )
}

/// Rank a bounded batch of Spotlight results for display in root launcher search.
pub fn rank_root_file_results(
    results: &[FileResult],
    query: &str,
    limit: usize,
    frecency_score: impl Fn(&str) -> f64,
) -> Vec<crate::scripts::FileMatch> {
    let q = query.trim().to_lowercase();
    if q.is_empty() || limit == 0 {
        return Vec::new();
    }

    let mut nucleo = crate::scripts::NucleoCtx::new(&q);
    let mut seen = std::collections::HashSet::new();
    let mut ranked: Vec<_> = results
        .iter()
        .filter(|file| seen.insert(file.path.clone()))
        .filter(|file| file.file_type != FileType::Application)
        .filter_map(|file| {
            let name_score = nucleo.score(&file.name);
            let (score, name_matched) = match name_score {
                Some(score) => (score, true),
                None => (nucleo.score(&file.path)?, false),
            };
            let text_tier = root_file_name_relevance_tier(&file.name, &q, name_matched);
            let frecency_bonus =
                (frecency_score(&format!("file/{}", file.path)) * 100.0).min(500.0) as i32;

            Some(crate::scripts::FileMatch {
                file: file.clone(),
                score: text_tier
                    .saturating_mul(ROOT_FILE_TEXT_TIER_MULTIPLIER)
                    .saturating_add(score.min(10_000) as i32)
                    .saturating_add(frecency_bonus),
            })
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.file.name.cmp(&b.file.name))
            .then_with(|| a.file.path.cmp(&b.file.path))
    });
    ranked.truncate(limit);
    ranked
}

/// Payload for file drag-out from the mini explorer.
///
/// Stored as the GPUI drag value. When the drag starts, we also initiate
/// a native macOS drag session so the file can be dropped into Finder
/// or other apps.
#[derive(Clone, Debug)]
pub struct FileDragPayload {
    pub name: String,
}

impl FileDragPayload {
    pub fn from_result(result: &FileResult) -> Self {
        Self {
            name: result.name.clone(),
        }
    }
}

/// Render implementation for the drag preview overlay.
impl gpui::Render for FileDragPayload {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::{div, px, rgb, ParentElement, Styled};

        let theme = crate::theme::get_cached_theme();
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .bg(rgb(theme.colors.background.title_bar))
            .border_1()
            .border_color(rgb(theme.colors.ui.border))
            .text_sm()
            .text_color(rgb(theme.colors.text.primary))
            .child(self.name.clone())
    }
}

#[cfg(test)]
pub(crate) use os_open::terminal_working_directory;
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

    let file_type = detect_file_type(path_obj);

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

/// Return true when a file path supports inline thumbnail previews in file search rows.
///
/// This intentionally matches the product requirement for thumbnail-capable image
/// extensions in the list UI.
#[allow(dead_code)]
pub fn is_thumbnail_preview_supported(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);

    matches!(
        extension.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "tiff")
    )
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
    crate::formatting::format_relative_time_long(unix_timestamp)
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
    rank_file_results_nucleo(results, filter_pattern)
        .into_iter()
        .map(|(idx, score)| (idx, results[idx].clone(), score))
        .collect()
}
/// Core nucleo ranking helper returning only (index, score).
///
/// This keeps sorting/ranking allocations minimal and lets callers choose
/// whether they need owned copies or borrowed references.
fn rank_file_results_nucleo(results: &[FileResult], filter_pattern: &str) -> Vec<(usize, u32)> {
    use crate::scripts::NucleoCtx;

    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, score)))
        .collect();

    // Sort by score descending (higher = better match), then by name to
    // keep ranking deterministic when scores tie.
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| results[a.0].name.cmp(&results[b.0].name))
            .then_with(|| a.0.cmp(&b.0))
    });

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
    rank_file_results_nucleo(results, filter_pattern)
        .into_iter()
        .map(|(idx, _)| (idx, &results[idx]))
        .collect()
}
// --- merged from part_004.rs ---
#[cfg(test)]
mod tests {
    // --- merged from part_000.rs ---
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
    fn recent_file_hydration_skips_missing_paths() {
        let path = std::env::temp_dir()
            .join(format!("sk-missing-recent-file-{}", std::process::id()))
            .to_string_lossy()
            .into_owned();
        assert!(file_result_from_existing_path(&path).is_none());
    }
    #[test]
    fn recent_file_hydration_skips_app_bundles() {
        let app_dir = std::env::temp_dir().join(format!(
            "sk-recent-file-hydration-{}.app",
            std::process::id()
        ));
        std::fs::create_dir_all(&app_dir).expect("create temporary app bundle directory");
        let result = file_result_from_existing_path(&app_dir.to_string_lossy());
        let _ = std::fs::remove_dir_all(&app_dir);
        assert!(result.is_none(), "app bundles should stay in app search");
    }
    #[test]
    fn recent_file_hydration_returns_file_result_for_existing_file() {
        let file_path = std::env::temp_dir().join(format!(
            "sk-recent-file-hydration-{}.txt",
            std::process::id()
        ));
        std::fs::write(&file_path, "recent file proof").expect("write temp file");
        let result = file_result_from_existing_path(&file_path.to_string_lossy())
            .expect("hydrate existing file");
        let _ = std::fs::remove_file(&file_path);

        assert_eq!(
            result.name,
            file_path.file_name().unwrap().to_string_lossy()
        );
        assert_eq!(result.file_type, FileType::Document);
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

    #[test]
    fn root_file_search_requires_simple_name_queries() {
        assert!(!should_search_root_files(""));
        assert!(!should_search_root_files("ab"));
        assert!(should_search_root_files("abc"));
        assert!(should_search_root_files("  abc  "));
        assert!(!should_search_root_files("/Users/example"));
        assert!(!should_search_root_files("~/Documents"));
        assert!(!should_search_root_files("kMDItemFSName == 'notes.txt'"));
    }

    fn file(path: &str, name: &str, file_type: FileType) -> FileResult {
        FileResult {
            path: path.to_string(),
            name: name.to_string(),
            size: 0,
            modified: 0,
            file_type,
        }
    }

    #[test]
    fn root_directory_browse_query_accepts_child_fragments() {
        assert_eq!(
            root_file_section_mode_for_query("~/dev/"),
            Some(RootFileSectionMode::DirectoryBrowse)
        );
        assert_eq!(
            root_file_section_mode_for_query("~/dev/al"),
            Some(RootFileSectionMode::DirectoryBrowse)
        );
        assert_eq!(
            root_directory_query_base("~/dev/al"),
            Some("~/dev/".to_string())
        );
        assert_eq!(
            root_file_section_mode_for_query("fix"),
            Some(RootFileSectionMode::GlobalQuery)
        );
    }

    #[test]
    fn root_directory_browse_source_key_ignores_child_fragments() {
        let root =
            std::env::temp_dir().join(format!("script-kit-root-source-key-{}", std::process::id()));
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).expect("create temp root directory");

        let base = format!("{}/", nested.display());
        let with_fragment = format!("{base}al");

        let base_key = root_directory_browse_source_key(&base);
        let fragment_key = root_directory_browse_source_key(&with_fragment);

        assert_eq!(base_key, fragment_key);
        assert_eq!(
            base_key,
            Some((nested.to_string_lossy().into_owned(), false))
        );

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn root_directory_browse_source_key_keeps_hidden_mode() {
        let root = std::env::temp_dir().join(format!(
            ".script-kit-root-source-key-hidden-{}",
            std::process::id()
        ));
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).expect("create hidden temp root directory");

        let query = format!("{}/al", nested.display());

        assert_eq!(
            root_directory_browse_source_key(&query),
            Some((nested.to_string_lossy().into_owned(), true))
        );

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn root_directory_file_matches_preserves_provider_order_without_filter() {
        let results = vec![
            file("/tmp/beta.txt", "beta.txt", FileType::Document),
            file("/tmp/alpha.txt", "alpha.txt", FileType::Document),
        ];

        let matches = root_directory_file_matches(&results, None, 10);

        assert_eq!(
            matches
                .iter()
                .map(|entry| entry.file.name.as_str())
                .collect::<Vec<_>>(),
            vec!["beta.txt", "alpha.txt"]
        );
    }

    #[test]
    fn root_directory_file_matches_filters_by_child_name() {
        let results = vec![
            file("/tmp/beta-notes.md", "beta-notes.md", FileType::Document),
            file("/tmp/beta-folder", "beta-folder", FileType::Directory),
            file(
                "/tmp/alpha-report.md",
                "alpha-report.md",
                FileType::Document,
            ),
            file("/tmp/alpha-folder", "alpha-folder", FileType::Directory),
        ];

        let matches = root_directory_file_matches(&results, Some("al"), 10);

        assert_eq!(
            matches
                .iter()
                .map(|entry| entry.file.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha-folder", "alpha-report.md"],
            "child-fragment filtering should match and rank direct child names only"
        );
    }

    #[test]
    fn root_directory_file_matches_does_not_score_parent_paths() {
        let results = vec![
            file(
                "/tmp/alpha-parent/report.md",
                "report.md",
                FileType::Document,
            ),
            file("/tmp/other/alpha.md", "alpha.md", FileType::Document),
        ];

        let matches = root_directory_file_matches(&results, Some("alpha"), 10);

        assert_eq!(
            matches
                .iter()
                .map(|entry| entry.file.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha.md"],
            "directory child filtering should not match text that appears only in the parent path"
        );
    }

    #[test]
    fn root_file_ranking_caps_dedupes_and_skips_apps() {
        let results = vec![
            file("/tmp/fix.txt", "fix.txt", FileType::Document),
            file("/tmp/fix.txt", "fix duplicate.txt", FileType::Document),
            file("/Applications/Fix.app", "Fix.app", FileType::Application),
            file("/tmp/fix-notes.md", "fix-notes.md", FileType::Document),
            file("/tmp/prefix-fix.md", "prefix-fix.md", FileType::Document),
        ];

        let ranked = rank_root_file_results(&results, "fix", 2, |_| 0.0);

        assert_eq!(ranked.len(), 2, "render limit should cap root rows");
        assert!(
            ranked
                .iter()
                .all(|entry| entry.file.file_type != FileType::Application),
            "root search should not duplicate app launcher results"
        );
        assert_eq!(
            ranked
                .iter()
                .filter(|entry| entry.file.path == "/tmp/fix.txt")
                .count(),
            1,
            "duplicate Spotlight paths should collapse to one row"
        );
    }

    #[test]
    fn root_file_ranking_applies_frecency_to_close_matches() {
        let results = vec![
            file("/tmp/fix-alpha.txt", "fix-alpha.txt", FileType::Document),
            file("/tmp/fix-beta.txt", "fix-beta.txt", FileType::Document),
        ];

        let ranked = rank_root_file_results(&results, "fix", 2, |key| {
            if key == "file//tmp/fix-beta.txt" {
                10.0
            } else {
                0.0
            }
        });

        assert_eq!(
            ranked.first().map(|entry| entry.file.path.as_str()),
            Some("/tmp/fix-beta.txt"),
            "frecency should break close root-file ranking ties"
        );
    }

    #[test]
    fn root_file_ranking_prefers_stem_exact_over_path_only_frecency() {
        let results = vec![
            file(
                "/tmp/fix/archive/report.md",
                "report.md",
                FileType::Document,
            ),
            file("/tmp/other/fix.md", "fix.md", FileType::Document),
        ];

        let ranked = rank_root_file_results(&results, "fix", 2, |key| {
            if key == "file//tmp/fix/archive/report.md" {
                10.0
            } else {
                0.0
            }
        });

        assert_eq!(
            ranked.first().map(|entry| entry.file.path.as_str()),
            Some("/tmp/other/fix.md"),
            "filename stem exact should beat path-only matches even with frecency"
        );
    }

    #[test]
    fn root_file_ranking_prefers_filename_prefix_over_boundary_contains() {
        let results = vec![
            file("/tmp/prefix-fix.md", "prefix-fix.md", FileType::Document),
            file("/tmp/fix-notes.md", "fix-notes.md", FileType::Document),
        ];

        let ranked = rank_root_file_results(&results, "fix", 2, |_| 0.0);

        assert_eq!(
            ranked.first().map(|entry| entry.file.name.as_str()),
            Some("fix-notes.md"),
            "filename prefix should beat separator-boundary contains"
        );
    }

    #[test]
    fn root_file_ranking_prefers_exact_stem_over_filename_prefix() {
        let results = vec![
            file(
                "/tmp/notes-backup.md",
                "notes-backup.md",
                FileType::Document,
            ),
            file("/tmp/notes.md", "notes.md", FileType::Document),
            file(
                "/tmp/notes/archive/report.md",
                "report.md",
                FileType::Document,
            ),
        ];

        let ranked = rank_root_file_results(&results, "notes", 3, |_| 0.0);

        assert_eq!(
            ranked.first().map(|entry| entry.file.name.as_str()),
            Some("notes.md"),
            "exact filename stem should beat prefix and path-only matches"
        );
        assert!(
            ranked
                .iter()
                .position(|entry| entry.file.name == "notes-backup.md")
                < ranked
                    .iter()
                    .position(|entry| entry.file.name == "report.md"),
            "filename prefix should rank ahead of path-only matches"
        );
    }

    #[test]
    fn root_file_ranking_prefers_fuzzy_filename_over_path_only() {
        let results = vec![
            file("/tmp/final/report.md", "report.md", FileType::Document),
            file(
                "/tmp/other/fnl-notes.md",
                "fnl-notes.md",
                FileType::Document,
            ),
        ];

        let ranked = rank_root_file_results(&results, "fnl", 2, |_| 0.0);

        assert_eq!(
            ranked.first().map(|entry| entry.file.name.as_str()),
            Some("fnl-notes.md"),
            "fuzzy filename match should beat a path-only match"
        );
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
    fn test_is_thumbnail_preview_supported_returns_true_for_supported_extensions() {
        assert!(is_thumbnail_preview_supported("/tmp/photo.png"));
        assert!(is_thumbnail_preview_supported("/tmp/photo.JPG"));
        assert!(is_thumbnail_preview_supported("/tmp/photo.jpeg"));
        assert!(is_thumbnail_preview_supported("/tmp/animation.gif"));
        assert!(is_thumbnail_preview_supported("/tmp/icon.webp"));
        assert!(is_thumbnail_preview_supported("/tmp/logo.svg"));
        assert!(is_thumbnail_preview_supported("/tmp/picture.bmp"));
        assert!(is_thumbnail_preview_supported("/tmp/favicon.ico"));
        assert!(is_thumbnail_preview_supported("/tmp/scan.tiff"));
    }

    #[test]
    fn test_is_thumbnail_preview_supported_returns_false_for_unsupported_extensions() {
        assert!(!is_thumbnail_preview_supported("/tmp/photo.heic"));
        assert!(!is_thumbnail_preview_supported("/tmp/document.pdf"));
        assert!(!is_thumbnail_preview_supported("/tmp/README"));
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
        assert_eq!(format_relative_time(now), "just now");

        // Minutes ago
        assert_eq!(format_relative_time(now - 60), "1 minute ago");
        assert_eq!(format_relative_time(now - 120), "2 minutes ago");
        assert_eq!(format_relative_time(now - 59 * 60), "59 minutes ago");

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
    // --- merged from part_001.rs ---
    #[test]
    fn test_list_directory_limit() {
        let unique = format!(
            "script-kit-file-search-limit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let temp_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&temp_dir).expect("should create test directory");

        let nested_dir = temp_dir.join("A-dir");
        std::fs::create_dir_all(&nested_dir).expect("should create nested directory");
        std::fs::write(temp_dir.join("b.txt"), b"b").expect("should create b.txt");
        std::fs::write(temp_dir.join("a.txt"), b"a").expect("should create a.txt");
        std::fs::write(temp_dir.join("c.txt"), b"c").expect("should create c.txt");

        let results = list_directory(temp_dir.to_str().expect("utf8 temp path"), 3);
        let names: Vec<&str> = results.iter().map(|result| result.name.as_str()).collect();

        assert_eq!(results.len(), 3, "directory listing should obey limit");
        assert_eq!(
            names,
            vec!["A-dir", "a.txt", "b.txt"],
            "results should be sorted before truncation"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
    #[test]
    fn test_list_directory_zero_limit_returns_empty() {
        let tmp_dir = std::env::temp_dir();
        let results = list_directory(tmp_dir.to_str().expect("utf8 temp path"), 0);
        assert!(results.is_empty(), "limit=0 should return no results");
    }
    #[test]
    fn test_list_directory_hides_dotfiles_by_default() {
        let results = list_directory("~", 100);

        for result in &results {
            assert!(
                !result.name.starts_with('.'),
                "default listing should not include hidden files: {}",
                result.name
            );
        }
    }

    #[test]
    fn test_list_directory_with_options_can_include_dotfiles() {
        let unique = format!(
            "script-kit-file-search-hidden-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let temp_dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(temp_dir.join(".hidden-dir")).expect("should create hidden dir");
        std::fs::write(temp_dir.join(".hidden-file"), b"hidden").expect("should create dotfile");
        std::fs::write(temp_dir.join("visible-file"), b"visible").expect("should create file");

        let hidden_results =
            list_directory_with_options(temp_dir.to_str().expect("utf8 temp path"), 10, true);
        let hidden_names: Vec<&str> = hidden_results
            .iter()
            .map(|result| result.name.as_str())
            .collect();
        assert!(
            hidden_names.contains(&".hidden-dir"),
            "hidden listing should include hidden directories"
        );
        assert!(
            hidden_names.contains(&".hidden-file"),
            "hidden listing should include dotfiles"
        );

        let default_results = list_directory(temp_dir.to_str().expect("utf8 temp path"), 10);
        let default_names: Vec<&str> = default_results
            .iter()
            .map(|result| result.name.as_str())
            .collect();
        assert!(
            !default_names.contains(&".hidden-dir"),
            "default listing should still hide hidden directories"
        );
        assert!(
            !default_names.contains(&".hidden-file"),
            "default listing should still hide dotfiles"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
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
    fn test_filter_results_nucleo_empty_pattern_uses_name_tiebreaker() {
        let results = vec![
            FileResult {
                path: "/test/zeta.txt".to_string(),
                name: "zeta.txt".to_string(),
                size: 100,
                modified: 0,
                file_type: FileType::Document,
            },
            FileResult {
                path: "/test/alpha.txt".to_string(),
                name: "alpha.txt".to_string(),
                size: 200,
                modified: 0,
                file_type: FileType::Document,
            },
        ];

        let filtered = filter_results_nucleo_simple(&results, "");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].1.name, "alpha.txt");
        assert_eq!(filtered[1].1.name, "zeta.txt");
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
    fn parent_folder_search_query_returns_trailing_slashed_parent() {
        assert_eq!(
            parent_folder_search_query("/tmp/projects/readme.md"),
            Some("/tmp/projects/".to_string())
        );
    }
    #[test]
    fn parent_folder_search_query_handles_root_parent() {
        assert_eq!(parent_folder_search_query("/hosts"), Some("/".to_string()));
    }
    #[test]
    fn parent_folder_search_query_rejects_relative_leaf_without_parent() {
        assert_eq!(parent_folder_search_query("readme.md"), None);
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
    #[test]
    fn test_terminal_working_directory_uses_directory_path_when_is_dir() {
        let resolved = terminal_working_directory("/tmp/projects", true);
        assert_eq!(resolved, "/tmp/projects");
    }
    #[test]
    fn test_terminal_working_directory_uses_parent_for_file_paths() {
        let resolved = terminal_working_directory("/tmp/projects/readme.md", false);
        assert_eq!(resolved, "/tmp/projects");
    }
    #[test]
    fn test_terminal_working_directory_falls_back_to_current_dir_without_parent() {
        let resolved = terminal_working_directory("readme.md", false);
        assert_eq!(resolved, ".");
    }
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_move_to_trash_returns_explicit_unsupported_error_on_non_macos() {
        let error = move_to_trash("/tmp/projects/readme.md").unwrap_err();
        assert!(
            error.contains("only supported on macOS"),
            "error should explain platform limitation, got: {}",
            error
        );
    }
}
