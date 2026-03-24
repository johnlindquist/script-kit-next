//! Source-scanning regression tests that verify structural patterns in production code.
//!
//! These tests read source files and assert the presence of expected patterns,
//! ensuring coding conventions are maintained across the codebase.

// Re-export shared test utilities so submodules can use `super::*`.
pub use script_kit_gpui::test_utils::{
    count_occurrences, read_all_handle_action_sources, read_source, LIVE_HANDLE_ACTION_FILES,
};

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

#[path = "source_audits/file_action_path_helpers.rs"]
mod file_action_path_helpers;

#[path = "source_audits/structured_logging.rs"]
mod structured_logging;

#[path = "source_audits/trace_propagation.rs"]
mod trace_propagation;

#[path = "source_audits/consistent_structured_fields.rs"]
mod consistent_structured_fields;

#[path = "source_audits/builtin_dispatch_consistency.rs"]
mod builtin_dispatch_consistency;

#[path = "source_audits/emoji_picker.rs"]
mod emoji_picker;

#[path = "source_audits/dialog_tab_navigation.rs"]
mod dialog_tab_navigation;

#[path = "source_audits/no_popup_confirm_callers.rs"]
mod no_popup_confirm_callers;

#[path = "source_audits/arrow_interceptor_filtered_bounds.rs"]
mod arrow_interceptor_filtered_bounds;

#[path = "source_audits/mini_main_window.rs"]
mod mini_main_window;

#[path = "source_audits/mini_ai_window.rs"]
mod mini_ai_window;

/// Regression guard: fails if the deleted monolithic `handle_action.rs` file
/// reappears or if any `.rs` file under `src/` or `tests/` references the old
/// monolith path. This prevents accidental resurrection of the pre-split handler.
#[cfg(test)]
mod no_old_monolith {
    use std::path::Path;

    /// The old monolithic handler file must not exist on disk.
    #[test]
    fn test_no_old_monolith_file_exists() {
        let old_monolith = Path::new("src/app_actions/handle_action.rs");
        assert!(
            !old_monolith.exists(),
            "Old monolithic handler file still exists at {old_monolith:?}. \
             It was replaced by the modular split under src/app_actions/handle_action/."
        );
    }

    /// No `.rs` source file should contain a string-literal reference to the
    /// old monolith path (`handle_action.rs` as a bare file, not the directory).
    #[test]
    fn test_no_old_monolith_references() {
        let needle = "handle_action.rs";
        let mut violations = Vec::new();

        for dir in &["src", "tests"] {
            collect_rs_files_with_needle(Path::new(dir), needle, &mut violations);
        }

        // Exclude this very test file — it legitimately mentions the old path.
        let self_path = Path::new("tests/source_audits.rs")
            .canonicalize()
            .unwrap_or_default();
        violations.retain(|v: &String| {
            let canon = Path::new(v.split(':').next().unwrap_or(""))
                .canonicalize()
                .unwrap_or_default();
            canon != self_path
        });

        assert!(
            violations.is_empty(),
            "Found references to the deleted monolith path `{needle}` in:\n{}",
            violations.join("\n")
        );
    }

    fn collect_rs_files_with_needle(dir: &Path, needle: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files_with_needle(&path, needle, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(needle) {
                            out.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
                        }
                    }
                }
            }
        }
    }
}
