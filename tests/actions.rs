// Integration-level behavior tests for action handler feedback contracts.
// These verify the structural invariants of the action system across files.

use std::fs;

/// Verify the feedback contract constants exist in helpers.rs:
/// success feedback uses HUD, errors use Toast.
#[test]
fn feedback_contract_constants_defined() {
    let content =
        fs::read_to_string("src/app_actions/helpers.rs").expect("Failed to read helpers.rs");

    // HUD constants for success paths
    assert!(
        content.contains("HUD_SHORT_MS"),
        "HUD_SHORT_MS must be defined for success feedback"
    );
    assert!(
        content.contains("HUD_MEDIUM_MS"),
        "HUD_MEDIUM_MS must be defined for success feedback"
    );

    // Toast constants for error paths
    assert!(
        content.contains("TOAST_ERROR_MS"),
        "TOAST_ERROR_MS must be defined for error feedback"
    );
    assert!(
        content.contains("TOAST_CRITICAL_MS"),
        "TOAST_CRITICAL_MS must be defined for critical errors"
    );
}

/// Verify that the action handler files have the expected dispatch structure.
#[test]
fn action_handler_dispatch_structure_exists() {
    // Collect all handler source from modular handler directory
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let mut content = String::new();
    for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let chunk = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
            content.push_str(&chunk);
            content.push('\n');
        }
    }

    // Must handle copy actions
    assert!(
        content.contains("\"copy_path\""),
        "Action handler must dispatch copy_path"
    );
    assert!(
        content.contains("\"copy_deeplink\""),
        "Action handler must dispatch copy_deeplink"
    );
    assert!(
        content.contains("\"copy_content\""),
        "Action handler must dispatch copy_content"
    );

    // Must handle file search actions
    assert!(
        content.contains("\"open_file\""),
        "Action handler must dispatch open_file"
    );
    assert!(
        content.contains("\"quick_look\""),
        "Action handler must dispatch quick_look"
    );

    // Must have toast_manager usage for errors (may be in chunk files)
    assert!(
        content.contains("toast_manager.push("),
        "Action handler must use toast_manager for error feedback"
    );
}

/// Verify SDK actions use tracing, not legacy logging.
#[test]
fn sdk_actions_uses_modern_logging() {
    let content = fs::read_to_string("src/app_actions/sdk_actions.rs")
        .expect("Failed to read sdk_actions.rs");

    assert!(
        !content.contains("logging::log("),
        "sdk_actions.rs must not use legacy logging::log — use tracing:: instead"
    );
    assert!(
        content.contains("tracing::info!(") || content.contains("tracing::warn!("),
        "sdk_actions.rs must use tracing for observability"
    );
}

// ---------------------------------------------------------------------------
// Structural coverage tests for action handler consistency
// ---------------------------------------------------------------------------

/// Every known action ID that appears in the modular handle_action handlers
/// must have a match arm. This test collects all quoted action
/// ID strings from the dispatch match and verifies they form a known set.
#[test]
fn all_action_ids_have_handler_arms() {
    // Collect all handler source from modular handler directory
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let mut all_content = String::new();
    for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let chunk = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
            all_content.push_str(&chunk);
            all_content.push('\n');
        }
    }

    // These are the action IDs that must have handler arms in the dispatch.
    // Each entry represents a user-facing action the system can trigger.
    let required_action_ids = [
        // Clipboard actions
        "clipboard_pin",
        "clipboard_unpin",
        "clipboard_share",
        "clipboard_paste",
        "clipboard_attach_to_ai",
        "clipboard_copy",
        "clipboard_paste_keep_open",
        "clipboard_quick_look",
        "clipboard_open_with",
        "clipboard_annotate_cleanshot",
        "clipboard_upload_cleanshot",
        "clipboard_ocr",
        "clipboard_delete",
        "clipboard_delete_multiple",
        "clipboard_delete_all",
        "clipboard_save_file",
        "clipboard_save_snippet",
        // Script management actions
        "create_script",
        "run_script",
        "view_logs",
        "reveal_in_finder",
        "copy_path",
        "copy_deeplink",
        "copy_content",
        "copy_filename",
        "edit_script",
        "remove_script",
        "delete_script",
        "reload_scripts",
        "settings",
        "quit",
        // Shortcut / alias actions
        "configure_shortcut",
        "add_shortcut",
        "update_shortcut",
        "remove_shortcut",
        "add_alias",
        "update_alias",
        "remove_alias",
        // File search actions
        "open_file",
        "quick_look",
        "open_with",
        "show_info",
        "attach_to_ai",
        // Scriptlet actions
        "edit_scriptlet",
        "reveal_scriptlet_in_finder",
        "copy_scriptlet_path",
        "reset_ranking",
        // Control signals
        "__cancel__",
    ];

    for action_id in &required_action_ids {
        let pattern = format!("\"{}\"", action_id);
        assert!(
            all_content.contains(&pattern),
            "Action ID '{}' must have a handler arm in handle_action dispatch",
            action_id
        );
    }
}

/// No handler files should use the legacy `logging::log()` pattern.
/// All logging must use `tracing::` macros.
#[test]
fn no_legacy_logging_in_handler_files() {
    let handler_dir = std::path::Path::new("src/app_actions/handle_action");
    let files_to_check: Vec<std::path::PathBuf> = {
        let mut files = Vec::new();
        for entry in fs::read_dir(handler_dir).expect("Failed to read handle_action directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        files
    };

    let mut violations = Vec::new();
    for file_path in &files_to_check {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));
        let count = content.matches("logging::log(").count();
        if count > 0 {
            violations.push(format!(
                "{}: {} logging::log() calls",
                file_path.display(),
                count
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Legacy logging::log() calls found in handler files (must use tracing::):\n  {}",
        violations.join("\n  ")
    );
}

/// Every variant of `BuiltInFeature` must have a corresponding match arm
/// in `execute_builtin_with_query` in `builtin_execution.rs`.
#[test]
fn all_builtin_feature_variants_have_execution_arms() {
    let enum_source =
        fs::read_to_string("src/builtins/mod.rs").expect("Failed to read builtins/mod.rs");
    let exec_source = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("Failed to read builtin_execution.rs");

    // Extract variant names from the enum definition.
    // We look for lines between `pub enum BuiltInFeature {` and the closing `}`.
    let enum_start = enum_source
        .find("pub enum BuiltInFeature {")
        .expect("BuiltInFeature enum not found");
    let enum_body_start = enum_source[enum_start..]
        .find('{')
        .expect("Opening brace not found")
        + enum_start
        + 1;

    // Find matching closing brace (handle nested braces)
    let mut depth = 1;
    let mut enum_body_end = enum_body_start;
    for (i, ch) in enum_source[enum_body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    enum_body_end = enum_body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }

    let enum_body = &enum_source[enum_body_start..enum_body_end];

    // Extract variant names (identifier before `(` or `,` or end-of-line)
    let mut variants = Vec::new();
    for line in enum_body.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }
        // Extract the variant name (first identifier on the line)
        let variant_name: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !variant_name.is_empty() {
            variants.push(variant_name);
        }
    }

    assert!(
        !variants.is_empty(),
        "Failed to extract any variants from BuiltInFeature enum"
    );

    // Verify each variant appears in execute_builtin_with_query as a match arm
    let mut missing = Vec::new();
    for variant in &variants {
        // Look for `BuiltInFeature::VariantName` in the execution source
        let pattern = format!("BuiltInFeature::{}", variant);
        if !exec_source.contains(&pattern) {
            missing.push(variant.as_str());
        }
    }

    assert!(
        missing.is_empty(),
        "BuiltInFeature variants missing from execute_builtin_with_query:\n  {}",
        missing.join(", ")
    );
}
