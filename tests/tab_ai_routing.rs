//! Source-text pattern tests verifying Tab AI routing in startup_new_tab.rs.
//!
//! These tests read the source of the Tab interceptor and verify:
//! 1. Universal Tab AI routing calls `open_tab_ai_overlay`
//! 2. FileSearch Tab handling is preserved (before universal route)
//! 3. ChatPrompt setup-mode Tab handling is preserved (before universal route)
//! 4. Shift+Tab still routes to script generation, not Tab AI

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");

#[test]
fn tab_ai_routing_universal_tab_calls_open_overlay() {
    // Universal Tab AI routing must exist and call open_tab_ai_overlay
    assert!(
        TAB_SOURCE.contains("open_tab_ai_overlay"),
        "startup_new_tab.rs must call open_tab_ai_overlay for universal Tab AI"
    );

    // The universal route must guard on !has_shift and !show_actions_popup
    assert!(
        TAB_SOURCE.contains("!has_shift && !this.show_actions_popup"),
        "universal Tab AI route must check !has_shift && !show_actions_popup"
    );
}

#[test]
fn tab_ai_routing_preserves_file_search_tab() {
    // FileSearch must be handled BEFORE the universal Tab AI route
    let file_search_pos = TAB_SOURCE
        .find("AppView::FileSearchView")
        .expect("FileSearch Tab handling must exist");
    let overlay_pos = TAB_SOURCE
        .find("open_tab_ai_overlay")
        .expect("open_tab_ai_overlay must exist");

    assert!(
        file_search_pos < overlay_pos,
        "FileSearch Tab handler must come before universal Tab AI route"
    );
}

#[test]
fn tab_ai_routing_preserves_chat_prompt_tab() {
    // ChatPrompt must be handled BEFORE the universal Tab AI route
    let chat_pos = TAB_SOURCE
        .find("AppView::ChatPrompt")
        .expect("ChatPrompt Tab handling must exist");
    let overlay_pos = TAB_SOURCE
        .find("open_tab_ai_overlay")
        .expect("open_tab_ai_overlay must exist");

    assert!(
        chat_pos < overlay_pos,
        "ChatPrompt Tab handler must come before universal Tab AI route"
    );
}

#[test]
fn tab_ai_routing_shift_tab_uses_script_generation() {
    // Shift+Tab script generation route must exist
    assert!(
        TAB_SOURCE.contains("dispatch_ai_script_generation_from_query"),
        "Shift+Tab must still route to script generation"
    );

    // Shift+Tab route must come BEFORE universal Tab AI
    let shift_tab_pos = TAB_SOURCE
        .find("dispatch_ai_script_generation_from_query")
        .expect("Shift+Tab route must exist");
    let overlay_pos = TAB_SOURCE
        .find("open_tab_ai_overlay")
        .expect("open_tab_ai_overlay must exist");

    assert!(
        shift_tab_pos < overlay_pos,
        "Shift+Tab script generation must come before universal Tab AI route"
    );
}
