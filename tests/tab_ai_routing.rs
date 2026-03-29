//! Source-text contract tests verifying Tab AI routing and ownership.
//!
//! These tests read source files and verify:
//! 1. Tab routes through `open_tab_ai_chat` (not the removed overlay)
//! 2. FileSearch/ChatPrompt Tab handling is preserved (before universal route)
//! 3. Shift+Tab still routes to script generation, not Tab AI
//! 4. Legacy overlay state is fully removed
//! 5. Save-offer overlay is still rendered

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");
const SCRIPT_LIST_SOURCE: &str = include_str!("../src/render_script_list/mod.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");

// --- Ownership cutover: overlay state is gone ---

#[test]
fn app_state_no_longer_contains_tab_ai_overlay_state() {
    assert!(
        !APP_STATE_SOURCE.contains("tab_ai_state: Option<TabAiOverlayState>"),
        "legacy overlay state field must be removed from app_state.rs"
    );
}

#[test]
fn app_view_state_no_longer_declares_tab_ai_overlay_state() {
    assert!(
        !APP_VIEW_STATE_SOURCE.contains("struct TabAiOverlayState"),
        "legacy TabAiOverlayState struct must be deleted from app_view_state.rs"
    );
}

#[test]
fn tab_ai_mode_no_longer_renders_overlay() {
    assert!(
        !TAB_AI_MODE_SOURCE.contains("render_tab_ai_overlay"),
        "Tab AI should render only through AppView::TabAiChat, not the overlay"
    );
}

#[test]
fn tab_ai_mode_no_longer_has_overlay_open_close() {
    assert!(
        !TAB_AI_MODE_SOURCE.contains("fn open_tab_ai_overlay"),
        "open_tab_ai_overlay must be removed"
    );
    assert!(
        !TAB_AI_MODE_SOURCE.contains("fn close_tab_ai_overlay"),
        "close_tab_ai_overlay must be removed"
    );
    assert!(
        !TAB_AI_MODE_SOURCE.contains("fn is_tab_ai_overlay_open"),
        "is_tab_ai_overlay_open must be removed"
    );
}

// --- Full-view chat routing ---

#[test]
fn startup_routes_tab_into_full_view_chat() {
    assert!(
        TAB_SOURCE.contains("open_tab_ai_chat(cx)"),
        "startup_new_tab.rs must call open_tab_ai_chat for universal Tab AI"
    );
    assert!(
        !TAB_SOURCE.contains("open_tab_ai_overlay"),
        "startup_new_tab.rs must not reference the removed overlay opener"
    );
}

#[test]
fn tab_ai_routing_preserves_file_search_tab() {
    let file_search_pos = TAB_SOURCE
        .find("AppView::FileSearchView")
        .expect("FileSearch Tab handling must exist");
    let chat_pos = TAB_SOURCE
        .find("open_tab_ai_chat")
        .expect("open_tab_ai_chat must exist");

    assert!(
        file_search_pos < chat_pos,
        "FileSearch Tab handler must come before universal Tab AI route"
    );
}

#[test]
fn tab_ai_routing_preserves_chat_prompt_tab() {
    let chat_pos = TAB_SOURCE
        .find("AppView::ChatPrompt")
        .expect("ChatPrompt Tab handling must exist");
    let ai_pos = TAB_SOURCE
        .find("open_tab_ai_chat")
        .expect("open_tab_ai_chat must exist");

    assert!(
        chat_pos < ai_pos,
        "ChatPrompt Tab handler must come before universal Tab AI route"
    );
}

#[test]
fn tab_ai_routing_shift_tab_uses_script_generation() {
    assert!(
        TAB_SOURCE.contains("dispatch_ai_script_generation_from_query"),
        "Shift+Tab must still route to script generation"
    );

    let shift_tab_pos = TAB_SOURCE
        .find("dispatch_ai_script_generation_from_query")
        .expect("Shift+Tab route must exist");
    let ai_pos = TAB_SOURCE
        .find("open_tab_ai_chat")
        .expect("open_tab_ai_chat must exist");

    assert!(
        shift_tab_pos < ai_pos,
        "Shift+Tab script generation must come before universal Tab AI route"
    );
}

// --- Save-offer overlay rendering (still active) ---

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

// --- Tab re-entry guards ---

#[test]
fn startup_tab_interceptor_blocks_reentry_when_save_offer_is_open() {
    assert!(
        TAB_SOURCE.contains("tab_ai_save_offer_state.is_some()"),
        "startup_new_tab.rs must block Tab reentry while the save-offer overlay is visible"
    );
}

#[test]
fn open_tab_ai_chat_guards_save_offer_state() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("tab_ai_save_offer_state.is_some()"),
        "open_tab_ai_chat must guard against opening while save-offer is visible"
    );
}

// --- ScriptList fallback ---

#[test]
fn script_list_tab_fallback_routes_to_full_view_chat() {
    assert!(
        SCRIPT_LIST_SOURCE.contains("open_tab_ai_chat(cx)"),
        "ScriptList Tab fallback must route to open_tab_ai_chat"
    );
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_tab_ai_overlay"),
        "ScriptList Tab fallback must not reference the removed overlay"
    );
}

#[test]
fn script_list_tab_fallback_no_longer_opens_inline_ai_chat() {
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_ai_chat_from_main_window_query"),
        "ScriptList Tab fallback must not reopen the old inline AI chat path"
    );
}

// --- ChatPrompt-style entity ownership ---

#[test]
fn tab_ai_chat_owns_text_input_state() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("input: TextInputState"),
        "TabAiChat must own input via TextInputState, not raw String"
    );
}

#[test]
fn tab_ai_chat_owns_list_state() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("turns_list_state: ListState"),
        "TabAiChat must own a ListState for scrollable turns"
    );
}

#[test]
fn tab_ai_chat_owns_focus_handle() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("focus_handle: FocusHandle"),
        "TabAiChat must own a FocusHandle for focus tracking"
    );
}

#[test]
fn tab_ai_chat_implements_focusable() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("impl Focusable for TabAiChat")
            || APP_VIEW_STATE_SOURCE.contains("impl Focusable for TabAiChat"),
        "TabAiChat must implement Focusable"
    );
}

#[test]
fn tab_ai_chat_key_handler_uses_text_input_state() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("chat.input.handle_key("),
        "Key handling must route through TextInputState::handle_key()"
    );
}

#[test]
fn tab_ai_chat_no_longer_uses_raw_intent_string() {
    assert!(
        !APP_VIEW_STATE_SOURCE.contains("intent: String"),
        "TabAiChat should use TextInputState, not a raw intent String"
    );
}
