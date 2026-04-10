//! Source-contract tests for launcher AI hint visibility.
//!
//! Locks the invariant that the launcher header badge advertises Tab
//! for ACP Chat entry, while Cmd+Enter remains footer-only.

use std::fs;

fn launcher_hint_source() -> String {
    fs::read_to_string("src/components/launcher_ask_ai_hint.rs")
        .expect("Failed to read src/components/launcher_ask_ai_hint.rs")
}

#[test]
fn launcher_header_advertises_tab_without_cmd_enter() {
    let source = launcher_hint_source();

    assert!(
        source.contains("Ask"),
        "Launcher header must show 'Ask' label"
    );
    assert!(
        source.contains(".child(\"⇥\")"),
        "Launcher header must show Tab badge"
    );
    assert!(
        !source.contains(".child(\"⌘↩\")"),
        "Launcher header must not duplicate the Cmd+Enter badge"
    );
}

#[test]
fn mini_script_list_shows_tab_ai_hint_without_cmd_enter_badge() {
    let source = launcher_hint_source();
    let script_list = fs::read_to_string("src/render_script_list/mod.rs")
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
        !source.contains(".child(\"⌘↩\")"),
        "Mini ScriptList header must not keep the Cmd+Enter Ask-AI hint visible"
    );
    assert!(
        script_list.contains("script_list_mini_ai_hint_rendered"),
        "Mini ScriptList must emit a structured hint-render log"
    );
}

#[test]
fn mini_script_list_hint_log_marks_only_tab_affordance_in_header() {
    let source = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");

    assert!(
        source.contains("script_list_mini_ai_hint_rendered"),
        "mini ScriptList must emit a structured Ask AI hint render log"
    );
    assert!(
        source.contains("tab_hint = true") && source.contains("cmd_enter_hint = false"),
        "mini ScriptList hint log must record that only the Tab affordance remains in the header"
    );
}

#[test]
fn runtime_and_storybook_share_launcher_ask_hint_renderer() {
    let script_list = fs::read_to_string("src/render_script_list/mod.rs")
        .expect("Failed to read src/render_script_list/mod.rs");
    let storybook = fs::read_to_string("src/storybook/main_menu_variations/mod.rs")
        .expect("Failed to read src/storybook/main_menu_variations/mod.rs");

    assert!(
        script_list.contains("render_launcher_ask_ai_hint(chrome)"),
        "ScriptList should use the shared launcher ask-hint renderer"
    );
    assert!(
        storybook.contains("render_launcher_ask_ai_hint(chrome)"),
        "Main menu storybook should use the shared launcher ask-hint renderer"
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
