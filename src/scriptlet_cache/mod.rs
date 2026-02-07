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

include!("part_000.rs");
include!("part_001.rs");
include!("part_002.rs");
