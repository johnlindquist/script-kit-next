// Regression tests verifying the Cmd+I context split:
// - Script list: toggle_info (UI state toggle for focused info panel)
// - File search: file:show_info (native Finder info via macOS)
//
// These tests ensure discoverability, correct dispatch routing,
// and that the two contexts never collide.

use std::fs;

use super::builders::get_script_context_actions;
use super::types::ScriptInfo;

// ---------------------------------------------------------------------------
// 1. Script-list actions dialog includes "Show Info" with ⌘I
// ---------------------------------------------------------------------------

#[test]
fn script_context_actions_include_show_info_with_cmd_i() {
    let script = ScriptInfo::new("MyScript", "/tmp/info-context-split.ts");
    let actions = get_script_context_actions(&script);

    let info_action = actions
        .iter()
        .find(|a| a.id == "toggle_info")
        .expect("script context actions must include toggle_info for discoverability");

    assert_eq!(info_action.title, "Show Info");
    assert_eq!(
        info_action.shortcut.as_deref(),
        Some("⌘I"),
        "toggle_info must advertise ⌘I shortcut in actions dialog"
    );
    assert_eq!(
        info_action.section.as_deref(),
        Some("Actions"),
        "toggle_info must be in the Actions section"
    );
}

// ---------------------------------------------------------------------------
// 2. toggle_info dispatches through script action handler → toggle_info_panel
// ---------------------------------------------------------------------------

#[test]
fn toggle_info_dispatches_to_info_panel_toggle() {
    let source = fs::read_to_string("src/app_actions/handle_action/scripts.rs")
        .expect("Failed to read scripts.rs");

    assert!(
        source.contains("\"toggle_info\""),
        "script action handler must recognize toggle_info action ID"
    );
    assert!(
        source.contains("toggle_info_panel"),
        "toggle_info must call toggle_info_panel to mutate UI state"
    );
}

#[test]
fn toggle_info_panel_emits_structured_log_and_notifies() {
    let source =
        fs::read_to_string("src/app_impl/ui_window.rs").expect("Failed to read ui_window.rs");

    let fn_start = source
        .find("fn toggle_info_panel")
        .expect("toggle_info_panel function not found");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body.contains("event = \"toggle_info_panel\""),
        "toggle_info_panel must emit structured log event for agent observability"
    );
    assert!(
        fn_body.contains("visible = self.show_info_panel"),
        "toggle_info_panel log must include current visibility state"
    );
    assert!(
        fn_body.contains("cx.notify()"),
        "toggle_info_panel must call cx.notify() to trigger re-render"
    );
}

// ---------------------------------------------------------------------------
// 3. File-search Cmd+I maps to file:show_info (Finder), not toggle_info
// ---------------------------------------------------------------------------

#[test]
fn file_search_uses_distinct_show_info_action_id() {
    let source =
        fs::read_to_string("src/actions/builders/file_path.rs").expect("Failed to read file_path.rs");

    assert!(
        source.contains("\"file:show_info\""),
        "file search must use file:show_info, distinct from toggle_info"
    );
}

#[test]
fn file_search_show_info_dispatches_to_native_finder() {
    let source = fs::read_to_string("src/app_actions/handle_action/files.rs")
        .expect("Failed to read files.rs");

    assert!(
        source.contains("\"show_info\""),
        "file action handler must recognize show_info for Finder dispatch"
    );
    assert!(
        source.contains("crate::file_search::show_info"),
        "file show_info must dispatch to Finder-native show_info function"
    );
}

#[test]
fn script_list_cmd_i_routes_to_toggle_info_not_file_show_info() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read render_script_list/mod.rs");

    let cmd_i_pos = source
        .find("\"i\" => {")
        .expect("render_script_list must have Cmd+I handler for 'i' key");
    let snippet = &source[cmd_i_pos..cmd_i_pos + 200];

    assert!(
        snippet.contains("toggle_info"),
        "script list Cmd+I must route to toggle_info"
    );
    assert!(
        !snippet.contains("file:show_info"),
        "script list Cmd+I must NOT route to file:show_info"
    );
}

// ---------------------------------------------------------------------------
// 4. Shortcut normalization resolves ⌘I correctly through action matching
// ---------------------------------------------------------------------------

#[test]
fn normalize_display_shortcut_maps_cmd_i_symbols() {
    let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
        .expect("Failed to read actions_dialog.rs");

    let fn_start = source
        .find("fn normalize_display_shortcut")
        .expect("normalize_display_shortcut function not found");
    let fn_body = &source[fn_start..];

    assert!(
        fn_body.contains("'⌘' => parts.push(\"cmd\")"),
        "normalize_display_shortcut must map ⌘ to cmd"
    );
    assert!(
        fn_body.contains("parts.sort()"),
        "normalize_display_shortcut must sort modifiers for deterministic comparison"
    );
}

#[test]
fn startup_interceptor_scopes_cmd_i_to_script_list_view() {
    let source = fs::read_to_string("src/app_impl/startup_new_actions.rs")
        .expect("Failed to read startup_new_actions.rs");

    assert!(
        source.contains("Cmd+I -> toggle_info (ScriptList)"),
        "startup interceptor must route Cmd+I to toggle_info for ScriptList"
    );
    assert!(
        source.contains("AppView::ScriptList"),
        "startup interceptor must scope Cmd+I to ScriptList view only"
    );
}
