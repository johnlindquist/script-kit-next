//! Source-scanning regression tests that verify structural patterns in production code.
//!
//! These tests read source files and assert the presence of expected patterns,
//! ensuring coding conventions are maintained across the codebase.

use std::fs;

/// Read a source file and panic with a clear message on failure.
pub fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
}

/// Count non-overlapping occurrences of `needle` in `haystack`.
pub fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

/// Read and concatenate all modular handle_action source files.
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

#[path = "source_audits/builtin_confirmation.rs"]
mod builtin_confirmation;

#[path = "source_audits/clipboard_actions.rs"]
mod clipboard_actions;

#[path = "source_audits/execution_helpers.rs"]
mod execution_helpers;

#[path = "source_audits/shortcut_alias_file_actions.rs"]
mod shortcut_alias_file_actions;

#[path = "source_audits/action_coverage_audit.rs"]
mod action_coverage_audit;

#[path = "source_audits/action_file_clipboard_tools.rs"]
mod action_file_clipboard_tools;

#[path = "source_audits/action_script_management.rs"]
mod action_script_management;

#[path = "source_audits/action_scriptlet_ranking.rs"]
mod action_scriptlet_ranking;

#[path = "source_audits/action_shortcut_alias.rs"]
mod action_shortcut_alias;
