//! Source-text pattern tests verifying Tab AI routing in startup_new_tab.rs.
//!
//! These tests read the source of the Tab interceptor and verify:
//! 1. Universal Tab AI routing calls `open_tab_ai_overlay`
//! 2. FileSearch Tab handling is preserved (before universal route)
//! 3. ChatPrompt setup-mode Tab handling is preserved (before universal route)
//! 4. Shift+Tab still routes to script generation, not Tab AI

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");
const SCRIPT_LIST_SOURCE: &str = include_str!("../src/render_script_list/mod.rs");

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

// --- Save-offer overlay rendering ---

#[test]
fn render_impl_renders_tab_ai_save_offer_overlay() {
    assert!(
        RENDER_IMPL_SOURCE.contains("render_tab_ai_save_offer_overlay"),
        "render_impl.rs must build the Tab AI save-offer overlay"
    );
    assert!(
        RENDER_IMPL_SOURCE.contains("tab_ai_save_offer_overlay"),
        "render_impl.rs must compose the Tab AI save-offer overlay into the overlay stack"
    );
}

// --- Tab re-entry guard ---

#[test]
fn startup_tab_interceptor_blocks_reentry_when_save_offer_is_open() {
    assert!(
        TAB_SOURCE.contains("tab_ai_save_offer_state.is_some()"),
        "startup_new_tab.rs must block Tab reentry while the save-offer overlay is visible"
    );
}

#[test]
fn open_tab_ai_overlay_guards_save_offer_state() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.tab_ai_save_offer_state.is_some()"),
        "open_tab_ai_overlay must guard against opening while save-offer is visible"
    );
}

// --- ScriptList fallback ---

#[test]
fn script_list_tab_fallback_no_longer_opens_inline_ai_chat() {
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_ai_chat_from_main_window_query"),
        "ScriptList Tab fallback must not reopen the old inline AI chat path"
    );
}
