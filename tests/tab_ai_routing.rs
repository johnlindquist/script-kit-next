//! Contract tests verifying Tab AI routes to the harness terminal surface.
//!
//! The primary Tab AI entry path is:
//!   Tab key → `open_tab_ai_chat()` → `open_tab_ai_harness_terminal()` →
//!   `AppView::QuickTerminalView` rendered via `TermPrompt`.
//!
//! The inline `TabAiChat` entity is retained for legacy/internal use but
//! is NOT the implicit Tab entry point. Tests in this file validate that
//! the harness-terminal contract is the authoritative surface.

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");
const SCRIPT_LIST_SOURCE: &str = include_str!("../src/render_script_list/mod.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const HARNESS_SOURCE: &str = include_str!("../src/ai/harness/mod.rs");
const TERM_RENDER_SOURCE: &str = include_str!("../src/render_prompts/term.rs");

// =========================================================================
// Primary contract: Tab → harness terminal (QuickTerminalView)
// =========================================================================

#[test]
fn open_tab_ai_chat_routes_to_harness_terminal() {
    // open_tab_ai_chat must unconditionally call open_tab_ai_harness_terminal.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_chat(")
        .expect("open_tab_ai_chat must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("open_tab_ai_harness_terminal(cx)"),
        "open_tab_ai_chat must call open_tab_ai_harness_terminal as the primary surface"
    );
    assert!(
        !open_fn_body.contains("open_tab_ai_full_view_chat"),
        "open_tab_ai_chat must NOT route to the legacy inline chat"
    );
}

#[test]
fn open_tab_ai_chat_does_not_create_tab_ai_chat_entity() {
    // The primary entry must not instantiate TabAiChat — it uses TermPrompt instead.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_chat(")
        .expect("open_tab_ai_chat must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        !open_fn_body.contains("TabAiChat::new"),
        "open_tab_ai_chat must not create a TabAiChat entity — the harness terminal is primary"
    );
}

#[test]
fn startup_routes_tab_into_harness_terminal() {
    assert!(
        TAB_SOURCE.contains("open_tab_ai_chat(cx)"),
        "startup_new_tab.rs must call open_tab_ai_chat for universal Tab AI"
    );
    assert!(
        !TAB_SOURCE.contains("open_tab_ai_overlay"),
        "startup_new_tab.rs must not reference the removed overlay opener"
    );
    assert!(
        !TAB_SOURCE.contains("open_tab_ai_full_view_chat"),
        "startup_new_tab.rs must not call the legacy inline chat directly"
    );
}

#[test]
fn tab_ai_creates_quick_terminal_view() {
    // The harness terminal function must switch to QuickTerminalView.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal(")
        .expect("open_tab_ai_harness_terminal must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("AppView::QuickTerminalView"),
        "open_tab_ai_harness_terminal must switch to QuickTerminalView"
    );
}

#[test]
fn tab_ai_reads_harness_config() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("read_tab_ai_harness_config"),
        "harness terminal path must load harness configuration before spawning"
    );
}

#[test]
fn tab_ai_uses_persistent_harness_session_state() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness: Option<crate::ai::TabAiHarnessSessionState>"),
        "app_state.rs must persist the Tab AI harness session"
    );
}

// =========================================================================
// Tab interceptor routing order
// =========================================================================

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

// =========================================================================
// Tab re-entry guards
// =========================================================================

#[test]
fn startup_tab_interceptor_does_not_steal_tab_from_quick_terminal() {
    assert!(
        TAB_SOURCE.contains("AppView::QuickTerminalView"),
        "startup_new_tab.rs must special-case QuickTerminalView"
    );
    assert!(
        TAB_SOURCE.contains("cx.propagate();"),
        "QuickTerminalView Tab handling must propagate so the harness TUI receives Tab"
    );
}

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

// =========================================================================
// Harness terminal: context injection and capture profile
// =========================================================================

#[test]
fn harness_terminal_uses_text_safe_capture_profile() {
    // The harness terminal path must use tab_ai_submit() (no screenshots)
    // not tab_ai() (includes base64 screenshots that bloat PTY payloads).
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal(")
        .expect("open_tab_ai_harness_terminal must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("tab_ai_submit()"),
        "harness terminal must use tab_ai_submit() (text-safe, no screenshots) for generic PTY"
    );
    assert!(
        !open_fn_body.contains("CaptureContextOptions::tab_ai()"),
        "harness terminal must NOT use tab_ai() (includes screenshots) for generic PTY backends"
    );
}

#[test]
fn harness_terminal_entry_uses_paste_only_mode() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal(")
        .expect("open_tab_ai_harness_terminal must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("PasteOnly"),
        "Tab entry must use PasteOnly so context is staged without auto-submitting"
    );
    assert!(
        !open_fn_body.contains("TabAiHarnessSubmissionMode::Submit"),
        "Tab entry must not use Submit mode — user types intent before pressing Enter"
    );
}

#[test]
fn harness_injection_supports_paste_only_mode() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("send_text_as_paste"),
        "PasteOnly injection must use send_text_as_paste, not send_line"
    );
}

#[test]
fn harness_submission_uses_script_kit_context_block() {
    assert!(
        HARNESS_SOURCE.contains("<scriptKitContext schemaVersion="),
        "harness submission must wrap context in a structured scriptKitContext block"
    );
}

// =========================================================================
// Harness session lifecycle: reuse and validation
// =========================================================================

#[test]
fn ensure_harness_reuses_existing_live_session() {
    // ensure_tab_ai_harness_terminal must check for an existing live session
    // and return it without spawning a new PTY.
    let ensure_fn_start = TAB_AI_MODE_SOURCE
        .find("fn ensure_tab_ai_harness_terminal(")
        .expect("ensure_tab_ai_harness_terminal must exist");
    let ensure_fn_body = &TAB_AI_MODE_SOURCE[ensure_fn_start..];
    let next_fn = ensure_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(ensure_fn_body.len());
    let ensure_fn_body = &ensure_fn_body[..next_fn];

    assert!(
        ensure_fn_body.contains("tab_ai_harness"),
        "must check the existing harness session"
    );
    assert!(
        ensure_fn_body.contains("is_alive()"),
        "must check whether the existing session is still alive"
    );
    assert!(
        ensure_fn_body.contains("false"),
        "reused session must return was_cold_start=false"
    );
}

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
    assert!(
        TAB_AI_MODE_SOURCE.contains("harness.json"),
        "harness startup failure toast must mention the config file for user guidance"
    );
}

// =========================================================================
// Close semantics: Cmd+W closes wrapper, Escape owned by PTY
// =========================================================================

#[test]
fn quick_terminal_cmd_w_restores_previous_view() {
    assert!(
        TERM_RENDER_SOURCE.contains("close_tab_ai_harness_terminal"),
        "Cmd+W in QuickTerminalView must restore the previous surface"
    );
}

#[test]
fn quick_terminal_close_semantics_documented_in_code() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("Cmd+W` closes the wrapper"),
        "close_tab_ai_harness_terminal doc must state Cmd+W closes the wrapper"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("Escape` is forwarded to the PTY"),
        "close_tab_ai_harness_terminal doc must state Escape goes to the PTY"
    );
}

#[test]
fn quick_terminal_render_documents_close_contract() {
    assert!(
        TERM_RENDER_SOURCE.contains("close semantics contract"),
        "render_prompts/term.rs must document the close semantics contract"
    );
    assert!(
        TERM_RENDER_SOURCE.contains("Cmd+W closes the wrapper"),
        "render_prompts/term.rs must state Cmd+W closes the wrapper"
    );
    assert!(
        TERM_RENDER_SOURCE.contains("Escape is forwarded to the PTY"),
        "render_prompts/term.rs must state Escape goes to the PTY"
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
fn close_harness_terminal_restores_return_view() {
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("tab_ai_harness_return_view"),
        "close must read the saved return view"
    );
    assert!(
        close_fn_body.contains("tab_ai_harness_return_focus_target"),
        "close must read the saved focus target"
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

// =========================================================================
// Overlay state is fully removed
// =========================================================================

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

// =========================================================================
// Save-offer overlay rendering (still active)
// =========================================================================

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

// =========================================================================
// ScriptList fallback routes through open_tab_ai_chat (harness terminal)
// =========================================================================

#[test]
fn script_list_tab_fallback_routes_to_open_tab_ai_chat() {
    assert!(
        SCRIPT_LIST_SOURCE.contains("open_tab_ai_chat(cx)"),
        "ScriptList Tab fallback must route to open_tab_ai_chat"
    );
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_tab_ai_overlay"),
        "ScriptList Tab fallback must not reference the removed overlay"
    );
    assert!(
        !SCRIPT_LIST_SOURCE.contains("open_ai_chat_from_main_window_query"),
        "ScriptList Tab fallback must not reopen the old inline AI chat path"
    );
}

// =========================================================================
// Deterministic context capture (used by tests)
// =========================================================================

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

// =========================================================================
// Legacy inline chat: retained but NOT the primary Tab entry
// =========================================================================

#[test]
fn legacy_inline_chat_retained_but_not_primary() {
    // open_tab_ai_full_view_chat still exists for internal use
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn open_tab_ai_full_view_chat("),
        "legacy open_tab_ai_full_view_chat must be retained for internal use"
    );
    // But it's NOT called from the primary Tab route
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_chat(")
        .expect("open_tab_ai_chat must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        !open_fn_body.contains("open_tab_ai_full_view_chat"),
        "primary Tab route must NOT call legacy inline chat open function"
    );
}

#[test]
fn app_view_state_marks_tab_ai_chat_as_legacy() {
    assert!(
        APP_VIEW_STATE_SOURCE.contains("Legacy full-view Tab AI chat surface"),
        "TabAiChat must be marked legacy in app_view_state.rs"
    );
}

#[test]
fn submit_tab_ai_chat_with_intent_marked_legacy() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("Legacy full-view chat submission path"),
        "submit_tab_ai_chat_with_intent must be marked legacy in tab_ai_mode.rs"
    );
}

#[test]
fn legacy_close_tab_ai_chat_still_exists() {
    // close_tab_ai_chat handles closing the retained TabAiChat entity
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn close_tab_ai_chat("),
        "close_tab_ai_chat must exist for the retained TabAiChat entity"
    );
}

#[test]
fn legacy_tab_ai_chat_stores_preview_desktop_snapshot() {
    // The retained TabAiChat entity still stores a preview snapshot
    assert!(
        APP_VIEW_STATE_SOURCE.contains("preview_desktop_snapshot"),
        "TabAiChat entity must still store the desktop snapshot (retained for internal use)"
    );
}

#[test]
fn implicit_target_detection_uses_public_function() {
    let tab_context_source = include_str!("../src/ai/tab_context.rs");
    assert!(
        tab_context_source.contains("pub fn tab_ai_intent_uses_implicit_target("),
        "tab_ai_intent_uses_implicit_target must be a public function"
    );
}

// =========================================================================
// Harness injection: cold vs warm start delay
// =========================================================================

#[test]
fn harness_injection_uses_cold_start_delay() {
    let inject_fn_start = TAB_AI_MODE_SOURCE
        .find("fn inject_tab_ai_harness_submission(")
        .expect("inject_tab_ai_harness_submission must exist");
    let inject_fn_body = &TAB_AI_MODE_SOURCE[inject_fn_start..];
    let next_fn = inject_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(inject_fn_body.len());
    let inject_fn_body = &inject_fn_body[..next_fn];

    assert!(
        inject_fn_body.contains("was_cold_start"),
        "injection must differentiate cold vs warm start for delay"
    );
    assert!(
        inject_fn_body.contains("delay_ms"),
        "injection must apply a startup delay for cold starts"
    );
}

#[test]
fn harness_injection_submits_or_pastes_based_on_flag() {
    let inject_fn_start = TAB_AI_MODE_SOURCE
        .find("fn inject_tab_ai_harness_submission(")
        .expect("inject_tab_ai_harness_submission must exist");
    let inject_fn_body = &TAB_AI_MODE_SOURCE[inject_fn_start..];
    let next_fn = inject_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(inject_fn_body.len());
    let inject_fn_body = &inject_fn_body[..next_fn];

    assert!(
        inject_fn_body.contains("send_line"),
        "Submit mode must use send_line for a full turn"
    );
    assert!(
        inject_fn_body.contains("send_text_as_paste"),
        "PasteOnly mode must use send_text_as_paste for staging"
    );
}
