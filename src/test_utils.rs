//! Shared test utilities used across source-scanning regression tests.
//!
//! Consolidates the `read()` helper that was previously duplicated in every
//! test module that asserts against source file contents.

#![allow(dead_code)]

use std::fs;
use std::sync::{Mutex, OnceLock};

/// Global lock for tests that mutate the `SK_PATH` environment variable.
///
/// `std::env::set_var` is process-global, so any test that changes `SK_PATH`
/// must hold this lock to avoid racing with other tests that also read or
/// write the same variable.  Use `unwrap_or_else(|e| e.into_inner())` when
/// acquiring to recover from a poisoned mutex (prior test panic).
pub static SK_PATH_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Read a source file and panic with a clear message on failure.
///
/// Intended for tests that scan source files for structural patterns.
/// Use this instead of inline `fs::read_to_string(...).unwrap_or_else(...)`.
pub fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
}

/// Count non-overlapping occurrences of `needle` in `haystack`.
pub fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

/// The canonical set of live handler files under `src/app_actions/handle_action/`.
///
/// Source-audit and coverage tests should iterate over this list rather than
/// scanning the directory with a glob.  If a handler file is added or removed,
/// update this list and the corresponding tests will follow.
pub const LIVE_HANDLE_ACTION_FILES: &[&str] = &[
    "src/app_actions/handle_action/clipboard.rs",
    "src/app_actions/handle_action/files.rs",
    "src/app_actions/handle_action/mod.rs",
    "src/app_actions/handle_action/scripts.rs",
    "src/app_actions/handle_action/scriptlets.rs",
    "src/app_actions/handle_action/shortcuts.rs",
];

/// Read and concatenate all live modular handle_action source files.
///
/// Uses [`LIVE_HANDLE_ACTION_FILES`] instead of a directory glob so that
/// source-scanning tests only validate the actual live implementation.
pub fn read_all_handle_action_sources() -> String {
    let mut combined = String::new();
    for path in LIVE_HANDLE_ACTION_FILES {
        let content = read_source(path);
        combined.push_str(&content);
        combined.push('\n');
    }
    combined
}
