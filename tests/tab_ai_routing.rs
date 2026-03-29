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
        "Tab AI should route through the harness terminal, not the overlay"
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

// --- Harness terminal routing ---

const HARNESS_SOURCE: &str = include_str!("../src/ai/harness/mod.rs");
const TERM_RENDER_SOURCE: &str = include_str!("../src/render_prompts/term.rs");

#[test]
fn tab_ai_uses_persistent_harness_session_state() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness: Option<crate::ai::TabAiHarnessSessionState>"),
        "app_state.rs must persist the Tab AI harness session"
    );
}

#[test]
fn tab_ai_creates_quick_terminal_view() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("AppView::QuickTerminalView"),
        "open_tab_ai_chat must switch to QuickTerminalView"
    );
}

#[test]
fn tab_ai_reads_harness_config() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("read_tab_ai_harness_config"),
        "open_tab_ai_chat must load harness configuration before spawning"
    );
}

#[test]
fn harness_submission_uses_script_kit_context_block() {
    assert!(
        HARNESS_SOURCE.contains("<scriptKitContext schemaVersion="),
        "harness submission must wrap context in a structured scriptKitContext block"
    );
}

#[test]
fn quick_terminal_has_escape_to_previous_view() {
    assert!(
        TERM_RENDER_SOURCE.contains("close_tab_ai_harness_terminal"),
        "Escape in QuickTerminalView must restore the previous surface"
    );
}

#[test]
fn app_state_tracks_tab_ai_harness_return_target() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_return_view"),
        "app_state.rs must store the view to restore when leaving the Tab AI terminal"
    );
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_return_focus_target"),
        "app_state.rs must store the focus target to restore when leaving the Tab AI terminal"
    );
}

#[test]
fn tab_ai_reentry_uses_saved_originating_view() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("tab_ai_harness_return_view"),
        "warm-session reentry must use the saved originating view, not QuickTerminalView"
    );
}

#[test]
fn harness_injection_supports_paste_only_mode() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("send_text_as_paste"),
        "PasteOnly injection must use send_text_as_paste, not send_line"
    );
}

// --- Harness config validation ---

#[test]
fn harness_startup_validates_config_before_spawn() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("validate_tab_ai_harness_config"),
        "ensure_tab_ai_harness_terminal must validate config before spawning the PTY"
    );
}

#[test]
fn harness_validation_checks_empty_command() {
    let config = script_kit_gpui::ai::HarnessConfig {
        command: "".to_string(),
        ..Default::default()
    };
    let err = script_kit_gpui::ai::validate_tab_ai_harness_config(&config)
        .expect_err("empty command must fail");
    assert!(
        err.contains("harness.json"),
        "error must mention config file path for setup guidance: {err}"
    );
}

#[test]
fn harness_validation_checks_missing_cli() {
    let config = script_kit_gpui::ai::HarnessConfig {
        command: "nonexistent-harness-cli-abc123".to_string(),
        ..Default::default()
    };
    let err = script_kit_gpui::ai::validate_tab_ai_harness_config(&config)
        .expect_err("missing CLI must fail");
    assert!(
        err.contains("not found on PATH"),
        "error must mention PATH: {err}"
    );
    assert!(
        err.contains("harness.json"),
        "error must mention config file for setup guidance: {err}"
    );
}

#[test]
fn harness_startup_failure_toast_mentions_setup_guidance() {
    // The toast on harness start failure must mention the config file or CLI installation
    assert!(
        TAB_AI_MODE_SOURCE.contains("harness.json"),
        "harness startup failure toast must mention the config file for user guidance"
    );
}
