//! Source-contract tests for launcher AI hint visibility.
//!
//! Locks the invariant that the launcher header badge advertises both
//! Tab and Cmd+Enter for ACP Chat entry, in both mini and full modes.

use std::fs;

#[test]
fn launcher_header_advertises_tab_and_cmd_enter() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains("Ask"),
        "Launcher header must show 'Ask' label"
    );
    assert!(
        source.contains(".child(\"⇥\")"),
        "Launcher header must show Tab badge"
    );
    assert!(
        source.contains("⌘↩"),
        "Launcher header must show Cmd+Enter badge"
    );
}

#[test]
fn mini_script_list_shows_tab_and_cmd_enter_ai_hints() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains(".child(\"Ask\")"),
        "Mini ScriptList must keep the Ask label visible"
    );
    assert!(
        source.contains(".child(\"⇥\")"),
        "Mini ScriptList must keep the Tab Ask-AI hint visible"
    );
    assert!(
        source.contains(".child(\"⌘↩\")"),
        "Mini ScriptList must keep the Cmd+Enter Ask-AI hint visible"
    );
    assert!(
        source.contains("script_list_mini_ai_hint_rendered"),
        "Mini ScriptList must emit a structured hint-render log"
    );
}

#[test]
fn full_script_list_uses_unified_footer_with_ai_slot() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains("render_universal_prompt_hint_strip_clickable_with_primary_label"),
        "Full ScriptList must keep using the unified footer hint strip"
    );
    assert!(
        source.contains("script_list_footer_unified"),
        "Full ScriptList must emit the unified footer audit log"
    );
}

#[test]
fn startup_global_cmd_enter_keeps_shift_out_of_the_shared_route() {
    let startup = fs::read_to_string("src/app_impl/startup.rs")
        .expect("Failed to read src/app_impl/startup.rs");
    let startup_new_tab = fs::read_to_string("src/app_impl/startup_new_tab.rs")
        .expect("Failed to read src/app_impl/startup_new_tab.rs");

    for source in [&startup, &startup_new_tab] {
        assert!(
            source.contains("let has_shift = event.keystroke.modifiers.shift;"),
            "Startup interceptor must compute has_shift explicitly"
        );
        assert!(
            source.contains("&& !has_shift"),
            "Shared global Cmd+Enter interceptor must exclude Shift"
        );
    }
}
