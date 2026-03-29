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
        "Tab AI should route through the full-view chat, not the overlay"
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

// --- Full-view chat is the primary Tab AI surface ---

#[test]
fn open_tab_ai_chat_always_routes_to_full_view_chat() {
    // open_tab_ai_chat must unconditionally call open_tab_ai_full_view_chat
    // without checking harness config first.
    assert!(
        TAB_AI_MODE_SOURCE.contains("open_tab_ai_full_view_chat(cx)"),
        "open_tab_ai_chat must call open_tab_ai_full_view_chat unconditionally"
    );

    // The harness-first branch must not appear in open_tab_ai_chat
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_chat(")
        .expect("open_tab_ai_chat must exist");
    // Find the next function definition after open_tab_ai_chat
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        !open_fn_body.contains("read_tab_ai_harness_config"),
        "open_tab_ai_chat must not check harness config — full-view chat is the primary surface"
    );
    assert!(
        !open_fn_body.contains("open_tab_ai_harness_terminal"),
        "open_tab_ai_chat must not call harness terminal — full-view chat is the primary surface"
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

// --- Snapshot truthfulness: preview = submit ---

#[test]
fn tab_ai_chat_stores_preview_desktop_snapshot() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("preview_desktop_snapshot"),
        "TabAiChat must store the desktop snapshot captured at open time"
    );
}

#[test]
fn submit_uses_stored_preview_snapshot_not_recapture() {
    // The submit path must read the preview snapshot from the entity,
    // not call capture_context_snapshot again.
    let submit_fn_start = TAB_AI_MODE_SOURCE
        .find("fn submit_tab_ai_chat_with_intent(")
        .expect("submit function must exist");
    let submit_fn_body = &TAB_AI_MODE_SOURCE[submit_fn_start..];
    let next_fn = submit_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(submit_fn_body.len());
    let submit_fn_body = &submit_fn_body[..next_fn];

    assert!(
        submit_fn_body.contains("preview_desktop_snapshot"),
        "submit must read the stored preview snapshot from the chat entity"
    );
    assert!(
        !submit_fn_body.contains("capture_context_snapshot"),
        "submit must not re-capture desktop context — uses the preview snapshot the user saw"
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

// --- Desktop context card completeness ---

#[test]
fn desktop_context_card_includes_browser_row() {
    // build_tab_ai_context_cards must render a Browser row from desktop.browser
    let card_fn_start = TAB_AI_MODE_SOURCE
        .find("fn build_tab_ai_context_cards(")
        .expect("build_tab_ai_context_cards must exist");
    let card_fn_body = &TAB_AI_MODE_SOURCE[card_fn_start..];
    let next_fn = card_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(card_fn_body.len());
    let card_fn_body = &card_fn_body[..next_fn];

    assert!(
        card_fn_body.contains("\"Browser\""),
        "desktop context card must include a Browser row"
    );
}

#[test]
fn visible_items_card_uses_total_count_not_truncated() {
    // The visible items card title must use the full target count,
    // not the truncated take(5) count.
    let card_fn_start = TAB_AI_MODE_SOURCE
        .find("fn build_tab_ai_context_cards(")
        .expect("build_tab_ai_context_cards must exist");
    let card_fn_body = &TAB_AI_MODE_SOURCE[card_fn_start..];
    let next_fn = card_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(card_fn_body.len());
    let card_fn_body = &card_fn_body[..next_fn];

    assert!(
        card_fn_body.contains("visible_target_count"),
        "visible items card must use total visible_target_count in its title"
    );
}

// --- Esc restores the previous view ---

#[test]
fn escape_in_tab_ai_chat_calls_close() {
    // handle_tab_ai_chat_key_down must close the chat on Escape
    let handler_start = TAB_AI_MODE_SOURCE
        .find("fn handle_tab_ai_chat_key_down(")
        .expect("handle_tab_ai_chat_key_down must exist");
    let handler_body = &TAB_AI_MODE_SOURCE[handler_start..];
    let next_fn = handler_body[1..]
        .find("\n    fn ")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_fn];

    assert!(
        handler_body.contains("is_key_escape"),
        "key handler must check for Escape"
    );
    assert!(
        handler_body.contains("close_tab_ai_chat(cx)"),
        "Escape must call close_tab_ai_chat to restore the previous view"
    );
}

#[test]
fn close_tab_ai_chat_restores_return_view_and_focus() {
    // close_tab_ai_chat must read restore_target and set current_view + pending_focus
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_chat(")
        .expect("close_tab_ai_chat must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("restore_target()"),
        "close must read restore_target to get the originating view and focus"
    );
    assert!(
        close_fn_body.contains("self.current_view = return_view"),
        "close must restore the previous current_view"
    );
    assert!(
        close_fn_body.contains("self.pending_focus = Some(return_focus_target)"),
        "close must restore the pending focus target"
    );
}

// --- Suggestion submit on empty input ---

#[test]
fn empty_input_enter_submits_selected_suggestion() {
    // When input is empty and a suggestion is selected, Enter submits the suggestion's intent
    let handler_start = TAB_AI_MODE_SOURCE
        .find("fn handle_tab_ai_chat_key_down(")
        .expect("handle_tab_ai_chat_key_down must exist");
    let handler_body = &TAB_AI_MODE_SOURCE[handler_start..];
    let next_fn = handler_body[1..]
        .find("\n    fn ")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_fn];

    // Must check for empty input
    assert!(
        handler_body.contains("current_intent().trim().is_empty()"),
        "key handler must check whether input is empty for suggestion cycling"
    );

    // Must call selected_suggestion and submit its intent
    assert!(
        handler_body.contains("selected_suggestion()"),
        "Enter on empty input must read the selected suggestion"
    );
    assert!(
        handler_body.contains("submit_tab_ai_chat_with_intent"),
        "Enter on empty input must submit the suggestion's intent via submit_tab_ai_chat_with_intent"
    );
}

#[test]
fn up_down_arrows_cycle_suggestions_on_empty_input() {
    let handler_start = TAB_AI_MODE_SOURCE
        .find("fn handle_tab_ai_chat_key_down(")
        .expect("handle_tab_ai_chat_key_down must exist");
    let handler_body = &TAB_AI_MODE_SOURCE[handler_start..];
    let next_fn = handler_body[1..]
        .find("\n    fn ")
        .unwrap_or(handler_body.len());
    let handler_body = &handler_body[..next_fn];

    assert!(
        handler_body.contains("move_selected_suggestion(-1)"),
        "Up arrow must decrement the selected suggestion index"
    );
    assert!(
        handler_body.contains("move_selected_suggestion(1)"),
        "Down arrow must increment the selected suggestion index"
    );
}

// --- Explicit error on missing stable target ---

#[test]
fn submit_rejects_implicit_target_intent_without_focused_target() {
    // submit_tab_ai_chat_with_intent must check for implicit target and reject
    let submit_fn_start = TAB_AI_MODE_SOURCE
        .find("fn submit_tab_ai_chat_with_intent(")
        .expect("submit function must exist");
    let submit_fn_body = &TAB_AI_MODE_SOURCE[submit_fn_start..];
    let next_fn = submit_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(submit_fn_body.len());
    let submit_fn_body = &submit_fn_body[..next_fn];

    assert!(
        submit_fn_body.contains("focused_target.is_none()"),
        "submit must check whether a focused target exists"
    );
    assert!(
        submit_fn_body.contains("tab_ai_intent_uses_implicit_target"),
        "submit must detect implicit-target intents"
    );
    assert!(
        submit_fn_body.contains("missing_implicit_target"),
        "submit must set an explicit error kind when no stable target exists"
    );
    assert!(
        submit_fn_body.contains("Select an item or describe the target explicitly"),
        "error message must guide the user to select a target or use explicit intent"
    );
}

#[test]
fn implicit_target_detection_uses_public_function() {
    // tab_ai_intent_uses_implicit_target must be a public function for testability
    let tab_context_source = include_str!("../src/ai/tab_context.rs");
    assert!(
        tab_context_source.contains("pub fn tab_ai_intent_uses_implicit_target("),
        "tab_ai_intent_uses_implicit_target must be a public function"
    );
}

// --- Fallback Tab in ScriptList still routes to full-view chat ---

#[test]
fn script_list_tab_fallback_is_full_view_not_overlay_or_harness() {
    // The ScriptList Tab fallback must not reference harness terminal
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_tab_ai_harness_terminal"),
        "ScriptList Tab fallback must not open the harness terminal"
    );
    // It must call open_tab_ai_chat which unconditionally opens full-view chat
    assert!(
        SCRIPT_LIST_SOURCE.contains("open_tab_ai_chat(cx)"),
        "ScriptList Tab fallback must call open_tab_ai_chat"
    );
}

// --- Deterministic context capture enablement ---

#[test]
fn deterministic_context_capture_function_is_public() {
    let capture_source = include_str!("../src/context_snapshot/capture.rs");
    assert!(
        capture_source.contains("pub fn enable_deterministic_context_capture()"),
        "enable_deterministic_context_capture must be a public function for integration tests"
    );
}

#[test]
fn deterministic_capture_uses_atomic_flag() {
    let capture_source = include_str!("../src/context_snapshot/capture.rs");
    assert!(
        capture_source.contains("DETERMINISTIC_CONTEXT"),
        "deterministic capture must use a static atomic flag"
    );
    assert!(
        capture_source.contains("AtomicBool"),
        "the flag must be an AtomicBool for thread safety"
    );
}

#[test]
fn live_capture_respects_deterministic_flag() {
    let capture_source = include_str!("../src/context_snapshot/capture.rs");
    // The main capture function must check the deterministic flag
    let capture_fn_start = capture_source
        .find("pub fn capture_context_snapshot(")
        .expect("capture_context_snapshot must exist");
    let capture_fn_body = &capture_source[capture_fn_start..];
    let next_fn = capture_fn_body[1..]
        .find("\npub ")
        .unwrap_or(capture_fn_body.len());
    let capture_fn_body = &capture_fn_body[..next_fn];

    assert!(
        capture_fn_body.contains("DETERMINISTIC_CONTEXT"),
        "capture_context_snapshot must check the deterministic flag to return empty seed in tests"
    );
}

// --- Snapshot truthfulness: open and submit use the same snapshot ---

#[test]
fn open_captures_preview_once_and_passes_to_entity() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_full_view_chat(")
        .expect("open function must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    // Must capture exactly once
    let capture_count = open_fn_body.matches("capture_context_snapshot").count();
    assert_eq!(
        capture_count, 1,
        "open_tab_ai_full_view_chat must capture desktop context exactly once (got {capture_count})"
    );

    // Must pass to both card construction and the entity
    assert!(
        open_fn_body.contains("build_tab_ai_context_cards("),
        "captured snapshot must be passed to card construction"
    );
    assert!(
        open_fn_body.contains("desktop_preview.clone()"),
        "captured snapshot must be cloned into the TabAiChat entity"
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
fn quick_terminal_cmd_w_restores_previous_view() {
    assert!(
        TERM_RENDER_SOURCE.contains("close_tab_ai_harness_terminal"),
        "Cmd+W in QuickTerminalView must restore the previous surface"
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
