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
