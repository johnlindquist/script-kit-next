//! Shared test utilities used across source-scanning regression tests.
//!
//! Consolidates the `read()` helper that was previously duplicated in every
//! test module that asserts against source file contents.

use std::fs;

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
