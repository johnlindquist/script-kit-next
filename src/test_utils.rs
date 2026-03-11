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

/// Read and concatenate all modular handle_action source files.
///
/// Returns the combined contents of all `.rs` files under
/// `src/app_actions/handle_action/` so source-scanning tests can search
/// across the full action dispatch implementation.
pub fn read_all_handle_action_sources() -> String {
    let dir = "src/app_actions/handle_action";
    let mut combined = String::new();
    if let Ok(entries) = fs::read_dir(dir) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(content) = fs::read_to_string(&path) {
                combined.push_str(&content);
                combined.push('\n');
            }
        }
    }
    assert!(
        !combined.is_empty(),
        "Failed to read any handle_action module files from {dir}/"
    );
    combined
}
