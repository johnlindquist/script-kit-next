//! Scriptlet cache module for tracking per-file scriptlet state with change detection.
//!
//! This module provides:
//! - `CachedScriptlet`: Lightweight struct tracking scriptlet registration metadata
//! - `CachedScriptletFile`: Per-file cache with mtime for staleness detection
//! - `ScriptletCache`: HashMap-based cache for all scriptlet files
//! - `ScriptletDiff`: Diff result identifying what changed between old and new scriptlets
//!
//! # Usage
//!
//! ```rust,ignore
//! use script_kit_gpui::scriptlet_cache::{ScriptletCache, CachedScriptlet, diff_scriptlets};
//!
//! let mut cache = ScriptletCache::new();
//!
//! // Add a file's scriptlets
//! let scriptlets = vec![
//!     CachedScriptlet::new("My Snippet", Some("cmd+shift+m"), None, None, "/path/to/file.md#my-snippet"),
//! ];
//! cache.update_file("/path/to/file.md", mtime, scriptlets);
//!
//! // Check if file is stale
//! if cache.is_stale("/path/to/file.md", current_mtime) {
//!     let old = cache.get_scriptlets("/path/to/file.md").unwrap_or_default();
//!     let new = load_scriptlets_from_file("/path/to/file.md");
//!     let diff = diff_scriptlets(&old, &new);
//!     // Apply diff to hotkeys/expand_manager
//! }
//! ```

// --- merged from part_000.rs ---
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
// Re-export validation types from scriptlets module for convenience
pub use crate::scriptlets::{ScriptletParseResult, ScriptletValidationError};
/// File fingerprint for robust staleness detection.
///
/// Using both mtime AND size catches more real changes than mtime alone:
/// - mtime alone can miss edits within the same timestamp quantum (filesystem resolution)
/// - mtime alone misses changes from `cp -p` or sync tools that preserve timestamps
/// - Size changes almost always indicate content changes
///
/// This is a "cheap win" without requiring content hashing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileFingerprint {
    /// Last modification time
    pub mtime: SystemTime,
    /// File size in bytes
    pub size: u64,
}
impl FileFingerprint {
    /// Create a new fingerprint from mtime and size
    pub fn new(mtime: SystemTime, size: u64) -> Self {
        Self { mtime, size }
    }

    /// Create a fingerprint from filesystem metadata
    ///
    /// Returns None if metadata cannot be read (file doesn't exist, permissions, etc.)
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let metadata = std::fs::metadata(path.as_ref()).ok()?;
        let mtime = metadata.modified().ok()?;
        let size = metadata.len();
        Some(Self { mtime, size })
    }
}
/// Lightweight struct tracking scriptlet registration metadata.
/// This is a subset of the full Scriptlet struct, containing only
/// the fields needed for change detection and registration updates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedScriptlet {
    /// Name of the scriptlet (used as identifier)
    pub name: String,
    /// Keyboard shortcut (e.g., "cmd shift k")
    pub shortcut: Option<String>,
    /// Text expansion trigger (e.g., "type,,")
    pub keyword: Option<String>,
    /// Alias trigger (e.g., "gpt")
    pub alias: Option<String>,
    /// Source file path with anchor (e.g., "/path/to/file.md#my-snippet")
    pub file_path: String,
}
impl CachedScriptlet {
    /// Create a new CachedScriptlet
    pub fn new(
        name: impl Into<String>,
        shortcut: Option<String>,
        keyword: Option<String>,
        alias: Option<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            shortcut,
            keyword,
            alias,
            file_path: file_path.into(),
        }
    }
}
/// Per-file cache entry tracking scriptlets and staleness metadata.
#[derive(Clone, Debug)]
pub struct CachedScriptletFile {
    /// Absolute path to the markdown file (NOTE: redundant with map key, kept for convenience)
    pub path: PathBuf,
    /// Last modification time when the file was cached (legacy, prefer fingerprint)
    pub mtime: SystemTime,
    /// Full fingerprint for robust staleness detection (mtime + size)
    pub fingerprint: Option<FileFingerprint>,
    /// Scriptlets extracted from this file
    pub scriptlets: Vec<CachedScriptlet>,
}
impl CachedScriptletFile {
    /// Create a new CachedScriptletFile (legacy mtime-only API)
    pub fn new(
        path: impl Into<PathBuf>,
        mtime: SystemTime,
        scriptlets: Vec<CachedScriptlet>,
    ) -> Self {
        Self {
            path: path.into(),
            mtime,
            fingerprint: None,
            scriptlets,
        }
    }

    /// Create a new CachedScriptletFile with full fingerprint
    pub fn with_fingerprint(
        path: impl Into<PathBuf>,
        fingerprint: FileFingerprint,
        scriptlets: Vec<CachedScriptlet>,
    ) -> Self {
        Self {
            path: path.into(),
            mtime: fingerprint.mtime,
            fingerprint: Some(fingerprint),
            scriptlets,
        }
    }
}
/// Cache for all scriptlet files, providing staleness detection and CRUD operations.
#[derive(Debug, Default)]
pub struct ScriptletCache {
    files: HashMap<PathBuf, CachedScriptletFile>,
}
impl ScriptletCache {
    /// Create a new empty ScriptletCache
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Check if a file is stale (mtime differs from cached mtime)
    pub fn is_stale(&self, path: impl AsRef<Path>, current_mtime: SystemTime) -> bool {
        match self.files.get(path.as_ref()) {
            Some(cached) => cached.mtime != current_mtime,
            None => true, // Not in cache means stale (needs initial load)
        }
    }

    /// Get the cached scriptlets for a file (clones the Vec)
    ///
    /// Note: Prefer `get_scriptlets_ref()` to avoid cloning when possible.
    pub fn get_scriptlets(&self, path: impl AsRef<Path>) -> Option<Vec<CachedScriptlet>> {
        self.files.get(path.as_ref()).map(|f| f.scriptlets.clone())
    }

    /// Get the cached scriptlets as a slice reference (zero-copy).
    ///
    /// This is the preferred API when you don't need ownership, as it avoids
    /// cloning the Vec and all the Strings inside each CachedScriptlet.
    ///
    /// # Panics (debug only)
    /// Panics if path is not absolute (helps catch path identity bugs early).
    pub fn get_scriptlets_ref(&self, path: impl AsRef<Path>) -> Option<&[CachedScriptlet]> {
        let path = path.as_ref();
        debug_assert!(
            path.is_absolute(),
            "ScriptletCache expects absolute paths, got: {}",
            path.display()
        );
        self.files.get(path).map(|f| f.scriptlets.as_slice())
    }

    /// Get the cached file entry
    pub fn get_file(&self, path: impl AsRef<Path>) -> Option<&CachedScriptletFile> {
        self.files.get(path.as_ref())
    }

    /// Update or insert a file's scriptlets in the cache
    pub fn update_file(
        &mut self,
        path: impl Into<PathBuf>,
        mtime: SystemTime,
        scriptlets: Vec<CachedScriptlet>,
    ) {
        let path = path.into();
        self.files.insert(
            path.clone(),
            CachedScriptletFile::new(path, mtime, scriptlets),
        );
    }

    /// Remove a file from the cache
    pub fn remove_file(&mut self, path: impl AsRef<Path>) -> Option<CachedScriptletFile> {
        self.files.remove(path.as_ref())
    }

    /// Get the number of cached files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get all cached file paths
    pub fn file_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.keys()
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.files.clear();
    }

    // =========================================================================
    // New fingerprint-based API (preferred over mtime-only)
    // =========================================================================

    /// Check if a file is stale using full fingerprint (mtime + size).
    ///
    /// This is more robust than mtime-only because it catches:
    /// - Edits within the same timestamp quantum
    /// - Files replaced with `cp -p` or sync tools preserving timestamps
    ///
    /// # Panics (debug only)
    /// Panics if path is not absolute (helps catch path identity bugs early).
    pub fn is_stale_fingerprint(&self, path: impl AsRef<Path>, current: FileFingerprint) -> bool {
        let path = path.as_ref();
        debug_assert!(
            path.is_absolute(),
            "ScriptletCache expects absolute paths, got: {}",
            path.display()
        );
        match self.files.get(path) {
            Some(cached) => match cached.fingerprint {
                Some(fp) => fp != current,
                // Fallback to mtime-only comparison if no fingerprint stored
                None => cached.mtime != current.mtime,
            },
            None => true, // Not in cache means stale
        }
    }

    /// Update or insert a file using fingerprint (preferred API)
    pub fn update_file_with_fingerprint(
        &mut self,
        path: impl Into<PathBuf>,
        fingerprint: FileFingerprint,
        scriptlets: Vec<CachedScriptlet>,
    ) {
        let path = path.into();
        self.files.insert(
            path.clone(),
            CachedScriptletFile::with_fingerprint(path, fingerprint, scriptlets),
        );
    }

    /// Upsert a file and return the diff (atomic operation).
    ///
    /// This is the preferred API because it:
    /// 1. Computes diff before replacing old scriptlets (no need to clone first)
    /// 2. Returns the diff so callers can correctly unregister/register
    /// 3. Ensures callers can't "forget" to handle changes
    ///
    /// # Panics (debug only)
    /// Panics if path is not absolute (helps catch path identity bugs early).
    pub fn upsert_file(
        &mut self,
        path: PathBuf,
        fingerprint: FileFingerprint,
        scriptlets: Vec<CachedScriptlet>,
    ) -> ScriptletDiff {
        debug_assert!(
            path.is_absolute(),
            "ScriptletCache expects absolute paths, got: {}",
            path.display()
        );
        match self.files.entry(path.clone()) {
            Entry::Vacant(v) => {
                // New file - all scriptlets are "added"
                let diff = ScriptletDiff {
                    added: scriptlets.clone(),
                    ..Default::default()
                };
                v.insert(CachedScriptletFile::with_fingerprint(
                    path,
                    fingerprint,
                    scriptlets,
                ));
                diff
            }
            Entry::Occupied(mut o) => {
                // Existing file - compute diff then replace
                let old_scriptlets = &o.get().scriptlets;
                let diff = diff_scriptlets(old_scriptlets, &scriptlets);
                // Replace with new content
                let entry = o.get_mut();
                entry.mtime = fingerprint.mtime;
                entry.fingerprint = Some(fingerprint);
                entry.scriptlets = scriptlets;
                diff
            }
        }
    }

    /// Remove a file and return its scriptlets (for unregistration).
    ///
    /// This is preferred over `remove_file()` when you need to unregister
    /// hotkeys/expands for the removed scriptlets.
    pub fn remove_file_with_scriptlets(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Option<Vec<CachedScriptlet>> {
        self.files.remove(path.as_ref()).map(|f| f.scriptlets)
    }
}
/// Represents a change to a scriptlet's shortcut
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutChange {
    pub name: String,
    pub file_path: String,
    pub old: Option<String>,
    pub new: Option<String>,
}
/// Represents a change to a scriptlet's expand trigger
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeywordChange {
    pub name: String,
    pub file_path: String,
    pub old: Option<String>,
    pub new: Option<String>,
}
/// Represents a change to a scriptlet's alias
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasChange {
    pub name: String,
    pub file_path: String,
    pub old: Option<String>,
    pub new: Option<String>,
}
/// Represents a change to a scriptlet's file_path (anchor changed but name stayed same)
///
/// This is critical for hotkey registrations: if the anchor changes, the registration
/// must be updated to point to the new location, even if the shortcut itself didn't change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePathChange {
    pub name: String,
    pub old: String,
    pub new: String,
}
/// Diff result identifying what changed between old and new scriptlets.
/// Used to update hotkey and expand registrations incrementally.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptletDiff {
    /// Scriptlets that were added (not present in old)
    pub added: Vec<CachedScriptlet>,
    /// Scriptlets that were removed (not present in new)
    pub removed: Vec<CachedScriptlet>,
    /// Scriptlets whose shortcut changed
    pub shortcut_changes: Vec<ShortcutChange>,
    /// Scriptlets whose expand trigger changed
    pub keyword_changes: Vec<KeywordChange>,
    /// Scriptlets whose alias changed
    pub alias_changes: Vec<AliasChange>,
    /// Scriptlets whose file_path/anchor changed (critical for re-registration)
    pub file_path_changes: Vec<FilePathChange>,
}
impl ScriptletDiff {
    /// Check if there are no changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.shortcut_changes.is_empty()
            && self.keyword_changes.is_empty()
            && self.alias_changes.is_empty()
            && self.file_path_changes.is_empty()
    }

    /// Get total number of changes
    pub fn change_count(&self) -> usize {
        self.added.len()
            + self.removed.len()
            + self.shortcut_changes.len()
            + self.keyword_changes.len()
            + self.alias_changes.len()
            + self.file_path_changes.len()
    }
}
/// Compute the diff between old and new scriptlets.
///
/// Scriptlets are matched by name. A scriptlet is considered:
/// - **Added**: Present in new but not in old
/// - **Removed**: Present in old but not in new
/// - **Changed**: Present in both but with different shortcut/expand/alias/file_path
///
/// CRITICAL: file_path changes are now detected. If the anchor changes but the name
/// stays the same, this is reported in `file_path_changes`. Without this, hotkey
/// registrations can silently point to stale paths.
pub fn diff_scriptlets(old: &[CachedScriptlet], new: &[CachedScriptlet]) -> ScriptletDiff {
    let mut diff = ScriptletDiff::default();

    // Build lookup maps by name
    let old_by_name: HashMap<&str, &CachedScriptlet> =
        old.iter().map(|s| (s.name.as_str(), s)).collect();
    let new_by_name: HashMap<&str, &CachedScriptlet> =
        new.iter().map(|s| (s.name.as_str(), s)).collect();

    // Find added and changed
    for new_scriptlet in new {
        match old_by_name.get(new_scriptlet.name.as_str()) {
            Some(old_scriptlet) => {
                // Check for shortcut changes
                if old_scriptlet.shortcut != new_scriptlet.shortcut {
                    diff.shortcut_changes.push(ShortcutChange {
                        name: new_scriptlet.name.clone(),
                        file_path: new_scriptlet.file_path.clone(),
                        old: old_scriptlet.shortcut.clone(),
                        new: new_scriptlet.shortcut.clone(),
                    });
                }
                // Check for expand changes
                if old_scriptlet.keyword != new_scriptlet.keyword {
                    diff.keyword_changes.push(KeywordChange {
                        name: new_scriptlet.name.clone(),
                        file_path: new_scriptlet.file_path.clone(),
                        old: old_scriptlet.keyword.clone(),
                        new: new_scriptlet.keyword.clone(),
                    });
                }
                // Check for alias changes
                if old_scriptlet.alias != new_scriptlet.alias {
                    diff.alias_changes.push(AliasChange {
                        name: new_scriptlet.name.clone(),
                        file_path: new_scriptlet.file_path.clone(),
                        old: old_scriptlet.alias.clone(),
                        new: new_scriptlet.alias.clone(),
                    });
                }
                // CRITICAL: Check for file_path/anchor changes
                // This catches the case where the anchor changed but the name stayed the same,
                // which would otherwise cause hotkey registrations to point to stale paths.
                if old_scriptlet.file_path != new_scriptlet.file_path {
                    diff.file_path_changes.push(FilePathChange {
                        name: new_scriptlet.name.clone(),
                        old: old_scriptlet.file_path.clone(),
                        new: new_scriptlet.file_path.clone(),
                    });
                }
            }
            None => {
                // Added
                diff.added.push(new_scriptlet.clone());
            }
        }
    }

    // Find removed
    for old_scriptlet in old {
        if !new_by_name.contains_key(old_scriptlet.name.as_str()) {
            diff.removed.push(old_scriptlet.clone());
        }
    }

    diff
}
// =============================================================================
// VALIDATION ERROR HANDLING
// =============================================================================

/// Get the path to the Script Kit log file
pub fn get_log_file_path() -> PathBuf {
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".scriptkit/logs/script-kit-gpui.jsonl"))
        .unwrap_or_else(|_| {
            // Use system temp directory instead of hardcoded /tmp for better security
            std::env::temp_dir().join("script-kit-gpui.jsonl")
        })
}

// --- merged from part_001.rs ---
/// Format a HUD message for scriptlet validation errors
///
/// # Returns
/// A user-friendly message suitable for HUD display
///
/// # Examples
/// - Single error: "Failed to parse 'My Script' in snippets.md"
/// - Multiple errors in one file: "Failed to parse 2 scriptlet(s) in snippets.md"
/// - Multiple files: "Parse errors in 3 file(s). Check logs for details."
pub fn format_parse_error_message(errors: &[ScriptletValidationError]) -> String {
    if errors.is_empty() {
        return String::new();
    }

    // Group errors by file path
    let mut by_file: HashMap<&Path, Vec<&ScriptletValidationError>> = HashMap::new();
    for error in errors {
        by_file.entry(&error.file_path).or_default().push(error);
    }

    let file_count = by_file.len();

    if file_count == 1 {
        // Single file - show more detail
        let (path, file_errors) = by_file.iter().next().unwrap();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

        if file_errors.len() == 1 {
            // Single error in single file - show scriptlet name if available
            if let Some(ref name) = file_errors[0].scriptlet_name {
                format!("Failed to parse '{}' in {}", name, filename)
            } else {
                format!("Failed to parse scriptlet in {}", filename)
            }
        } else {
            // Multiple errors in single file
            format!(
                "Failed to parse {} scriptlet(s) in {}",
                file_errors.len(),
                filename
            )
        }
    } else {
        // Multiple files with errors
        let total_errors: usize = by_file.values().map(|v| v.len()).sum();
        format!(
            "Parse errors in {} file(s) ({} total). Check logs.",
            file_count, total_errors
        )
    }
}
/// Log validation errors to JSONL for debugging
///
/// Logs each error with structured fields for easy filtering:
/// - category: "SCRIPTLET_PARSE"
/// - file_path: Source file
/// - scriptlet_name: Name of failing scriptlet (if known)
/// - line_number: Line where error occurred (if known)
/// - error_message: Description of the error
pub fn log_validation_errors(errors: &[ScriptletValidationError]) {
    use crate::logging;

    for error in errors {
        let scriptlet_info = error
            .scriptlet_name
            .as_ref()
            .map(|n| format!(" [{}]", n))
            .unwrap_or_default();

        let line_info = error
            .line_number
            .map(|l| format!(":{}", l))
            .unwrap_or_default();

        let message = format!(
            "{}{}{}:{}",
            error.file_path.display(),
            line_info,
            scriptlet_info,
            error.error_message
        );

        // Use the logging module to log to JSONL
        logging::log("SCRIPTLET_PARSE", &message);

        // Also log with tracing for structured fields
        tracing::warn!(
            category = "SCRIPTLET_PARSE",
            file_path = %error.file_path.display(),
            scriptlet_name = ?error.scriptlet_name,
            line_number = ?error.line_number,
            error_message = %error.error_message,
            "Scriptlet validation error"
        );
    }
}
/// Convert a Scriptlet to a CachedScriptlet for caching
pub fn scriptlet_to_cached(
    scriptlet: &crate::scriptlets::Scriptlet,
    file_path: &Path,
) -> CachedScriptlet {
    // Create file_path with anchor from scriptlet.command (the kebab-case identifier)
    // Note: This uses `command` not `name` - command is the stable identifier for anchors
    let anchor = scriptlet.command.clone();
    let full_path = format!("{}#{}", file_path.display(), anchor);

    CachedScriptlet::new(
        &scriptlet.name,
        scriptlet.metadata.shortcut.clone(),
        scriptlet
            .typed_metadata
            .as_ref()
            .and_then(|t| t.keyword.clone())
            .or(scriptlet.metadata.keyword.clone()),
        scriptlet.metadata.alias.clone(),
        full_path,
    )
}
/// Load and cache scriptlets from a markdown file with validation
///
/// This is the main entry point for loading scriptlets with error handling.
/// It:
/// 1. Parses the file using `parse_scriptlets_with_validation`
/// 2. Logs any validation errors to JSONL
/// 3. Returns the parse result for HUD notification and caching
///
/// # Arguments
/// * `content` - The markdown file content
/// * `file_path` - Path to the source file (for error reporting)
///
/// # Returns
/// `ScriptletParseResult` containing valid scriptlets and any errors
pub fn load_scriptlets_with_validation(content: &str, file_path: &Path) -> ScriptletParseResult {
    use crate::scriptlets::parse_scriptlets_with_validation;

    let source_path = file_path.to_str();
    let result = parse_scriptlets_with_validation(content, source_path);

    // Log any errors to JSONL
    if !result.errors.is_empty() {
        log_validation_errors(&result.errors);

        tracing::info!(
            category = "SCRIPTLET_PARSE",
            file_path = %file_path.display(),
            valid_count = result.scriptlets.len(),
            error_count = result.errors.len(),
            "Loaded scriptlets with {} valid, {} errors",
            result.scriptlets.len(),
            result.errors.len()
        );
    }

    result
}
/// Summary of errors suitable for creating cache + HUD notification
pub struct ParseErrorSummary {
    /// User-friendly message for HUD display
    pub hud_message: String,
    /// Total number of errors
    pub error_count: usize,
    /// Path to log file for "Open Logs" action
    pub log_file_path: PathBuf,
}
/// Create a summary of parse errors for HUD notification
///
/// # Arguments
/// * `errors` - Validation errors from parsing
///
/// # Returns
/// `Some(ParseErrorSummary)` if there are errors, `None` otherwise
pub fn create_error_summary(errors: &[ScriptletValidationError]) -> Option<ParseErrorSummary> {
    if errors.is_empty() {
        return None;
    }

    Some(ParseErrorSummary {
        hud_message: format_parse_error_message(errors),
        error_count: errors.len(),
        log_file_path: get_log_file_path(),
    })
}

// --- merged from part_002.rs ---
// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    // --- merged from part_000.rs ---
    use super::*;
    use std::time::Duration;
    // Helper to create a test mtime
    fn test_mtime(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }
    // -------------------------------------------------------------------------
    // Cache tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cache_add_and_retrieve() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);
        let path = PathBuf::from("/path/to/file.md");

        let scriptlets = vec![
            CachedScriptlet::new(
                "Snippet One",
                Some("cmd+shift+1".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-one",
            ),
            CachedScriptlet::new(
                "Snippet Two",
                None,
                Some("snip,,".to_string()),
                None,
                "/path/to/file.md#snippet-two",
            ),
        ];

        cache.update_file(&path, mtime, scriptlets.clone());

        // Verify file is in cache
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        // Retrieve scriptlets
        let retrieved = cache.get_scriptlets(&path).unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "Snippet One");
        assert_eq!(retrieved[0].shortcut, Some("cmd+shift+1".to_string()));
        assert_eq!(retrieved[1].name, "Snippet Two");
        assert_eq!(retrieved[1].keyword, Some("snip,,".to_string()));

        // Verify get_file
        let file = cache.get_file(&path).unwrap();
        assert_eq!(file.mtime, mtime);
        assert_eq!(file.path, path);
    }
    #[test]
    fn test_cache_staleness_detection() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime_old = test_mtime(1000);
        let mtime_new = test_mtime(2000);

        // Not in cache = stale
        assert!(cache.is_stale(&path, mtime_old));

        // Add to cache
        cache.update_file(&path, mtime_old, vec![]);

        // Same mtime = not stale
        assert!(!cache.is_stale(&path, mtime_old));

        // Different mtime = stale
        assert!(cache.is_stale(&path, mtime_new));
    }
    #[test]
    fn test_cache_remove_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime = test_mtime(1000);

        let scriptlets = vec![CachedScriptlet::new(
            "Test",
            None,
            None,
            None,
            "/path/to/file.md#test",
        )];

        cache.update_file(&path, mtime, scriptlets);
        assert_eq!(cache.len(), 1);

        // Remove file
        let removed = cache.remove_file(&path);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().scriptlets.len(), 1);

        // Verify removed
        assert_eq!(cache.len(), 0);
        assert!(cache.get_scriptlets(&path).is_none());

        // Remove non-existent returns None
        assert!(cache.remove_file(&path).is_none());
    }
    #[test]
    fn test_cache_update_existing() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let mtime1 = test_mtime(1000);
        let mtime2 = test_mtime(2000);

        // Initial add
        let scriptlets1 = vec![CachedScriptlet::new(
            "Original",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#original",
        )];
        cache.update_file(&path, mtime1, scriptlets1);

        // Update with new data
        let scriptlets2 = vec![
            CachedScriptlet::new(
                "Updated",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#updated",
            ),
            CachedScriptlet::new(
                "New One",
                None,
                Some("new,,".to_string()),
                None,
                "/path/to/file.md#new-one",
            ),
        ];
        cache.update_file(&path, mtime2, scriptlets2);

        // Verify update
        assert_eq!(cache.len(), 1); // Still one file
        let file = cache.get_file(&path).unwrap();
        assert_eq!(file.mtime, mtime2);
        assert_eq!(file.scriptlets.len(), 2);
        assert_eq!(file.scriptlets[0].name, "Updated");
    }
    #[test]
    fn test_cache_clear() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);

        cache.update_file("/path/a.md", mtime, vec![]);
        cache.update_file("/path/b.md", mtime, vec![]);
        cache.update_file("/path/c.md", mtime, vec![]);

        assert_eq!(cache.len(), 3);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
    #[test]
    fn test_cache_file_paths() {
        let mut cache = ScriptletCache::new();
        let mtime = test_mtime(1000);

        cache.update_file("/path/a.md", mtime, vec![]);
        cache.update_file("/path/b.md", mtime, vec![]);

        let paths: Vec<_> = cache.file_paths().collect();
        assert_eq!(paths.len(), 2);
    }
    // -------------------------------------------------------------------------
    // Diff tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_diff_no_changes() {
        let scriptlets = vec![
            CachedScriptlet::new(
                "Snippet One",
                Some("cmd+1".to_string()),
                None,
                Some("s1".to_string()),
                "/path/to/file.md#snippet-one",
            ),
            CachedScriptlet::new(
                "Snippet Two",
                None,
                Some("snip,,".to_string()),
                None,
                "/path/to/file.md#snippet-two",
            ),
        ];

        let diff = diff_scriptlets(&scriptlets, &scriptlets);

        assert!(diff.is_empty());
        assert_eq!(diff.change_count(), 0);
    }
    #[test]
    fn test_diff_scriptlet_added() {
        let old = vec![CachedScriptlet::new(
            "Existing",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#existing",
        )];

        let new = vec![
            CachedScriptlet::new(
                "Existing",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#existing",
            ),
            CachedScriptlet::new(
                "New Snippet",
                Some("cmd+2".to_string()),
                Some("new,,".to_string()),
                None,
                "/path/to/file.md#new-snippet",
            ),
        ];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");
        assert_eq!(diff.added[0].shortcut, Some("cmd+2".to_string()));
        assert_eq!(diff.added[0].keyword, Some("new,,".to_string()));
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_scriptlet_removed() {
        let old = vec![
            CachedScriptlet::new(
                "Will Stay",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#will-stay",
            ),
            CachedScriptlet::new(
                "Will Be Removed",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#will-be-removed",
            ),
        ];

        let new = vec![CachedScriptlet::new(
            "Will Stay",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#will-stay",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Will Be Removed");
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_shortcut_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+2".to_string()),
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Snippet");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+2".to_string()));
        assert!(diff.keyword_changes.is_empty());
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_shortcut_added() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None, // No shortcut initially
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()), // Shortcut added
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].old, None);
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+1".to_string()));
    }
    #[test]
    fn test_diff_shortcut_removed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            Some("cmd+1".to_string()), // Has shortcut
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None, // Shortcut removed
            None,
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, None);
    }
    #[test]
    fn test_diff_keyword_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            Some("old,,".to_string()),
            None,
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            Some("new,,".to_string()),
            None,
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert_eq!(diff.keyword_changes.len(), 1);
        assert_eq!(diff.keyword_changes[0].name, "Snippet");
        assert_eq!(diff.keyword_changes[0].old, Some("old,,".to_string()));
        assert_eq!(diff.keyword_changes[0].new, Some("new,,".to_string()));
        assert!(diff.alias_changes.is_empty());
    }
    #[test]
    fn test_diff_alias_changed() {
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            Some("old".to_string()),
            "/path/to/file.md#snippet",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            Some("new".to_string()),
            "/path/to/file.md#snippet",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());
        assert!(diff.keyword_changes.is_empty());
        assert_eq!(diff.alias_changes.len(), 1);
        assert_eq!(diff.alias_changes[0].name, "Snippet");
        assert_eq!(diff.alias_changes[0].old, Some("old".to_string()));
        assert_eq!(diff.alias_changes[0].new, Some("new".to_string()));
    }

    // --- merged from part_001.rs ---
    #[test]
    fn test_diff_multiple_changes() {
        let old = vec![
            CachedScriptlet::new(
                "Unchanged",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#unchanged",
            ),
            CachedScriptlet::new(
                "Will Be Removed",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#will-be-removed",
            ),
            CachedScriptlet::new(
                "Shortcut Changed",
                Some("cmd+3".to_string()),
                None,
                None,
                "/path/to/file.md#shortcut-changed",
            ),
            CachedScriptlet::new(
                "Keyword Changed",
                None,
                Some("old,,".to_string()),
                None,
                "/path/to/file.md#keyword-changed",
            ),
            CachedScriptlet::new(
                "Alias Changed",
                None,
                None,
                Some("oldalias".to_string()),
                "/path/to/file.md#alias-changed",
            ),
        ];

        let new = vec![
            CachedScriptlet::new(
                "Unchanged",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#unchanged",
            ),
            CachedScriptlet::new(
                "Shortcut Changed",
                Some("cmd+9".to_string()), // Changed
                None,
                None,
                "/path/to/file.md#shortcut-changed",
            ),
            CachedScriptlet::new(
                "Keyword Changed",
                None,
                Some("new,,".to_string()), // Changed
                None,
                "/path/to/file.md#keyword-changed",
            ),
            CachedScriptlet::new(
                "Alias Changed",
                None,
                None,
                Some("newalias".to_string()), // Changed
                "/path/to/file.md#alias-changed",
            ),
            CachedScriptlet::new(
                "New Snippet",
                Some("cmd+0".to_string()),
                None,
                None,
                "/path/to/file.md#new-snippet",
            ),
        ];

        let diff = diff_scriptlets(&old, &new);

        // Verify added
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");

        // Verify removed
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Will Be Removed");

        // Verify shortcut change
        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Shortcut Changed");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+3".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+9".to_string()));

        // Verify expand change
        assert_eq!(diff.keyword_changes.len(), 1);
        assert_eq!(diff.keyword_changes[0].name, "Keyword Changed");
        assert_eq!(diff.keyword_changes[0].old, Some("old,,".to_string()));
        assert_eq!(diff.keyword_changes[0].new, Some("new,,".to_string()));

        // Verify alias change
        assert_eq!(diff.alias_changes.len(), 1);
        assert_eq!(diff.alias_changes[0].name, "Alias Changed");
        assert_eq!(diff.alias_changes[0].old, Some("oldalias".to_string()));
        assert_eq!(diff.alias_changes[0].new, Some("newalias".to_string()));

        // Total changes
        assert_eq!(diff.change_count(), 5);
        assert!(!diff.is_empty());
    }
    #[test]
    fn test_diff_empty_to_empty() {
        let diff = diff_scriptlets(&[], &[]);
        assert!(diff.is_empty());
        assert_eq!(diff.change_count(), 0);
    }
    #[test]
    fn test_diff_empty_to_some() {
        let new = vec![CachedScriptlet::new(
            "New",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new",
        )];

        let diff = diff_scriptlets(&[], &new);

        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert_eq!(diff.change_count(), 1);
    }
    #[test]
    fn test_diff_some_to_empty() {
        let old = vec![CachedScriptlet::new(
            "Old",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#old",
        )];

        let diff = diff_scriptlets(&old, &[]);

        assert!(diff.added.is_empty());
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.change_count(), 1);
    }
    // -------------------------------------------------------------------------
    // Error message formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_parse_error_empty() {
        let errors: Vec<ScriptletValidationError> = vec![];
        assert_eq!(super::format_parse_error_message(&errors), "");
    }
    #[test]
    fn test_format_parse_error_single_with_name() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/snippets.md",
            Some("My Script".to_string()),
            Some(10),
            "Invalid syntax",
        )];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse 'My Script' in snippets.md");
    }
    #[test]
    fn test_format_parse_error_single_without_name() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/snippets.md",
            None,
            Some(5),
            "No code block found",
        )];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse scriptlet in snippets.md");
    }
    #[test]
    fn test_format_parse_error_multiple_in_one_file() {
        let errors = vec![
            ScriptletValidationError::new(
                "/path/to/snippets.md",
                Some("Script One".to_string()),
                Some(10),
                "Error 1",
            ),
            ScriptletValidationError::new(
                "/path/to/snippets.md",
                Some("Script Two".to_string()),
                Some(20),
                "Error 2",
            ),
        ];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Failed to parse 2 scriptlet(s) in snippets.md");
    }
    #[test]
    fn test_format_parse_error_multiple_files() {
        let errors = vec![
            ScriptletValidationError::new(
                "/path/to/file1.md",
                Some("Script A".to_string()),
                Some(10),
                "Error A",
            ),
            ScriptletValidationError::new(
                "/path/to/file2.md",
                Some("Script B".to_string()),
                Some(20),
                "Error B",
            ),
            ScriptletValidationError::new(
                "/path/to/file2.md",
                Some("Script C".to_string()),
                Some(30),
                "Error C",
            ),
        ];

        let msg = super::format_parse_error_message(&errors);
        assert_eq!(msg, "Parse errors in 2 file(s) (3 total). Check logs.");
    }
    #[test]
    fn test_get_log_file_path() {
        let path = super::get_log_file_path();
        // Should end with the expected filename
        assert!(path.ends_with("script-kit-gpui.jsonl"));
        // Should contain .scriptkit/logs in the path (or /tmp as fallback)
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(".scriptkit/logs") || path_str.contains("/tmp"),
            "Path should be in .scriptkit/logs or /tmp, got: {}",
            path_str
        );
    }
    #[test]
    fn test_create_error_summary_none_for_empty() {
        let errors: Vec<ScriptletValidationError> = vec![];
        assert!(super::create_error_summary(&errors).is_none());
    }
    #[test]
    fn test_create_error_summary_has_fields() {
        let errors = vec![ScriptletValidationError::new(
            "/path/to/file.md",
            Some("Test".to_string()),
            Some(1),
            "Error",
        )];

        let summary = super::create_error_summary(&errors).unwrap();
        assert_eq!(summary.error_count, 1);
        assert!(!summary.hud_message.is_empty());
        assert!(summary.log_file_path.ends_with("script-kit-gpui.jsonl"));
    }
    // -------------------------------------------------------------------------
    // Bug fix tests: file_path change detection (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_diff_file_path_changed_same_name() {
        // BUG: When a scriptlet's anchor/file_path changes but name stays the same,
        // the diff should detect this. Otherwise hotkey registrations point to stale paths.
        let old = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#old-anchor", // Old anchor
        )];

        let new = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new-anchor", // New anchor - this should be detected!
        )];

        let diff = diff_scriptlets(&old, &new);

        // This is the critical assertion: file_path changes MUST be detected
        assert!(
            !diff.file_path_changes.is_empty(),
            "file_path change should be detected when anchor changes"
        );
        assert_eq!(diff.file_path_changes.len(), 1);
        assert_eq!(diff.file_path_changes[0].name, "My Snippet");
        assert_eq!(diff.file_path_changes[0].old, "/path/to/file.md#old-anchor");
        assert_eq!(diff.file_path_changes[0].new, "/path/to/file.md#new-anchor");
    }
    #[test]
    fn test_diff_file_path_no_change() {
        // When file_path is the same, no file_path_change should be reported
        let old = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#same-anchor",
        )];

        let new = vec![CachedScriptlet::new(
            "My Snippet",
            Some("cmd+2".to_string()), // Shortcut changed, but file_path same
            None,
            None,
            "/path/to/file.md#same-anchor",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(
            diff.file_path_changes.is_empty(),
            "No file_path change when paths are identical"
        );
        assert_eq!(diff.shortcut_changes.len(), 1); // But shortcut did change
    }
    #[test]
    fn test_diff_is_empty_includes_file_path_changes() {
        // is_empty() should return false when there are file_path changes
        let old = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            None,
            "/path/to/file.md#old",
        )];

        let new = vec![CachedScriptlet::new(
            "Snippet",
            None,
            None,
            None,
            "/path/to/file.md#new",
        )];

        let diff = diff_scriptlets(&old, &new);

        assert!(
            !diff.is_empty(),
            "Diff with file_path changes should not be empty"
        );
        assert!(
            diff.change_count() > 0,
            "change_count should include file_path changes"
        );
    }
    // -------------------------------------------------------------------------
    // Bug fix tests: FileFingerprint staleness detection (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_fingerprint_equality() {
        let fp1 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp2 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp3 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 2048, // Different size
        };
        let fp4 = FileFingerprint {
            mtime: test_mtime(2000), // Different mtime
            size: 1024,
        };

        assert_eq!(fp1, fp2, "Same mtime and size should be equal");
        assert_ne!(fp1, fp3, "Different size should not be equal");
        assert_ne!(fp1, fp4, "Different mtime should not be equal");
    }
    #[test]
    fn test_cache_staleness_with_fingerprint() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp_old = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp_same_mtime_diff_size = FileFingerprint {
            mtime: test_mtime(1000),
            size: 2048, // Same mtime but different size
        };

        // Add to cache with fingerprint
        cache.update_file_with_fingerprint(&path, fp_old, vec![]);

        // Same fingerprint = not stale
        assert!(
            !cache.is_stale_fingerprint(&path, fp_old),
            "Same fingerprint should not be stale"
        );

        // Same mtime but different size = stale (this is the bug fix!)
        assert!(
            cache.is_stale_fingerprint(&path, fp_same_mtime_diff_size),
            "Same mtime but different size should be stale"
        );
    }

    // --- merged from part_002.rs ---
    // -------------------------------------------------------------------------
    // API improvement tests: upsert_file returning diff (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_upsert_file_returns_diff_for_new_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![CachedScriptlet::new(
            "New Snippet",
            Some("cmd+1".to_string()),
            None,
            None,
            "/path/to/file.md#new-snippet",
        )];

        let diff = cache.upsert_file(path.clone(), fp, scriptlets);

        // New file means all scriptlets are "added"
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "New Snippet");
        assert!(diff.removed.is_empty());
        assert!(diff.shortcut_changes.is_empty());

        // File should now be in cache
        assert_eq!(cache.len(), 1);
    }
    #[test]
    fn test_upsert_file_returns_diff_for_existing_file() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp1 = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };
        let fp2 = FileFingerprint {
            mtime: test_mtime(2000),
            size: 2048,
        };

        // Initial insert
        let initial = vec![
            CachedScriptlet::new(
                "Snippet A",
                Some("cmd+1".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-a",
            ),
            CachedScriptlet::new(
                "Snippet B",
                Some("cmd+2".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-b",
            ),
        ];
        cache.upsert_file(path.clone(), fp1, initial);

        // Update: change shortcut, remove B, add C
        let updated = vec![
            CachedScriptlet::new(
                "Snippet A",
                Some("cmd+9".to_string()), // Changed shortcut
                None,
                None,
                "/path/to/file.md#snippet-a",
            ),
            CachedScriptlet::new(
                "Snippet C",
                Some("cmd+3".to_string()),
                None,
                None,
                "/path/to/file.md#snippet-c",
            ),
        ];

        let diff = cache.upsert_file(path.clone(), fp2, updated);

        // Verify diff captures all changes
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "Snippet C");

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].name, "Snippet B");

        assert_eq!(diff.shortcut_changes.len(), 1);
        assert_eq!(diff.shortcut_changes[0].name, "Snippet A");
        assert_eq!(diff.shortcut_changes[0].old, Some("cmd+1".to_string()));
        assert_eq!(diff.shortcut_changes[0].new, Some("cmd+9".to_string()));

        // Cache should have updated content
        let cached = cache.get_scriptlets(&path).unwrap();
        assert_eq!(cached.len(), 2);
    }
    #[test]
    fn test_remove_file_returns_removed_scriptlets() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![
            CachedScriptlet::new("A", None, None, None, "/path/to/file.md#a"),
            CachedScriptlet::new("B", None, None, None, "/path/to/file.md#b"),
        ];
        cache.upsert_file(path.clone(), fp, scriptlets);

        // Remove returns the scriptlets that were removed (for unregistration)
        let removed = cache.remove_file_with_scriptlets(&path);
        assert!(removed.is_some());
        let removed = removed.unwrap();
        assert_eq!(removed.len(), 2);
        assert!(removed.iter().any(|s| s.name == "A"));
        assert!(removed.iter().any(|s| s.name == "B"));

        // Cache should be empty
        assert!(cache.is_empty());
    }
    // -------------------------------------------------------------------------
    // Zero-copy API tests (TDD)
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_scriptlets_ref_returns_slice() {
        let mut cache = ScriptletCache::new();
        let path = PathBuf::from("/path/to/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        let scriptlets = vec![
            CachedScriptlet::new("A", None, None, None, "/path/to/file.md#a"),
            CachedScriptlet::new("B", None, None, None, "/path/to/file.md#b"),
        ];
        cache.upsert_file(path.clone(), fp, scriptlets);

        // Zero-copy API should return a reference to the slice
        let slice = cache.get_scriptlets_ref(&path);
        assert!(slice.is_some());
        let slice = slice.unwrap();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].name, "A");
        assert_eq!(slice[1].name, "B");
    }
    #[test]
    fn test_get_scriptlets_ref_returns_none_for_missing() {
        let cache = ScriptletCache::new();
        let path = PathBuf::from("/nonexistent.md");

        let slice = cache.get_scriptlets_ref(&path);
        assert!(slice.is_none());
    }
    // -------------------------------------------------------------------------
    // Path normalization tests (TDD)
    // -------------------------------------------------------------------------

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_update_file_rejects_relative_path_in_debug() {
        let mut cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        // Should panic in debug mode when given a relative path
        cache.upsert_file(relative_path, fp, vec![]);
    }
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_is_stale_rejects_relative_path_in_debug() {
        let cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");
        let fp = FileFingerprint {
            mtime: test_mtime(1000),
            size: 1024,
        };

        // Should panic in debug mode when given a relative path
        let _ = cache.is_stale_fingerprint(&relative_path, fp);
    }
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ScriptletCache expects absolute paths")]
    fn test_get_scriptlets_ref_rejects_relative_path_in_debug() {
        let cache = ScriptletCache::new();
        let relative_path = PathBuf::from("relative/path/file.md");

        // Should panic in debug mode when given a relative path
        let _ = cache.get_scriptlets_ref(&relative_path);
    }
}
