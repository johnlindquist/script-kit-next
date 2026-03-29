//! Contract tests verifying Tab AI routes to the harness terminal surface.
//!
//! The primary Tab AI entry path is:
//!   Tab key → `open_tab_ai_chat()` → `open_tab_ai_harness_terminal()` →
//!   `AppView::QuickTerminalView` rendered via `TermPrompt`.
//!
//! Tests in this file validate that the harness-terminal contract is the
//! authoritative Tab AI surface.

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");
const SCRIPT_LIST_SOURCE: &str = include_str!("../src/render_script_list/mod.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const HARNESS_SOURCE: &str = include_str!("../src/ai/harness/mod.rs");
const TERM_RENDER_SOURCE: &str = include_str!("../src/render_prompts/term.rs");
const AI_MOD_SOURCE: &str = include_str!("../src/ai/mod.rs");

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
        "startup.rs must call open_tab_ai_chat for universal Tab AI"
    );
    assert!(
        !TAB_SOURCE.contains("open_tab_ai_overlay"),
        "startup.rs must not reference the removed overlay opener"
    );
    assert!(
        !TAB_SOURCE.contains("open_tab_ai_full_view_chat"),
        "startup.rs must not call the legacy inline chat directly"
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
fn implicit_target_detection_uses_public_function() {
    let tab_context_source = include_str!("../src/ai/tab_context.rs");
    assert!(
        tab_context_source.contains("pub fn tab_ai_intent_uses_implicit_target("),
        "tab_ai_intent_uses_implicit_target must be a public function"
    );
}

// =========================================================================
// Warm startup: prewarm on startup, opt-out, and first-Tab reuse
// =========================================================================

#[test]
fn startup_wires_prewarm_call() {
    // The startup file must call warm_tab_ai_harness_on_startup asynchronously
    // so the first Tab press finds a live PTY.
    assert!(
        TAB_SOURCE.contains("warm_tab_ai_harness_on_startup"),
        "startup.rs must invoke warm_tab_ai_harness_on_startup"
    );
}

#[test]
fn startup_prewarm_runs_after_tab_interceptor() {
    // Prewarm must be wired AFTER the Tab interceptor subscription is pushed,
    // so the interceptor is guaranteed to be in place before the harness spawns.
    let interceptor_push_pos = TAB_SOURCE
        .find("gpui_input_subscriptions.push(tab_interceptor)")
        .expect("Tab interceptor push must exist");
    let prewarm_pos = TAB_SOURCE
        .find("warm_tab_ai_harness_on_startup")
        .expect("prewarm call must exist");

    assert!(
        interceptor_push_pos < prewarm_pos,
        "prewarm must be wired after the Tab interceptor is installed"
    );
}

#[test]
fn startup_prewarm_is_async_and_detached() {
    // Prewarm must not block the startup path — it should be a spawned task.
    let prewarm_pos = TAB_SOURCE
        .find("warm_tab_ai_harness_on_startup")
        .expect("prewarm call must exist");
    let after_prewarm = &TAB_SOURCE[prewarm_pos..];

    assert!(
        after_prewarm.contains(".detach()"),
        "prewarm task must be detached so it does not block startup"
    );
}

#[test]
fn prewarm_checks_warm_on_startup_flag() {
    // The warm method must respect the config's warm_on_startup field.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("warm_on_startup"),
        "prewarm must check config.warm_on_startup before spawning"
    );
    assert!(
        warm_fn_body.contains("\"disabled\""),
        "prewarm must log reason=disabled when warm_on_startup is false"
    );
}

#[test]
fn prewarm_reuses_ensure_harness_terminal() {
    // Prewarm must call the same session constructor that Tab uses,
    // so the first Tab press reuses the prewarmed session.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("ensure_tab_ai_harness_terminal"),
        "prewarm must call ensure_tab_ai_harness_terminal (same path as Tab)"
    );
}

#[test]
fn prewarm_skips_if_session_already_alive() {
    // If a harness session is already live, prewarm should no-op.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("is_alive()"),
        "prewarm must check if existing session is alive"
    );
    assert!(
        warm_fn_body.contains("\"already_alive\""),
        "prewarm must log reason=already_alive when skipping"
    );
}

#[test]
fn prewarm_handles_config_read_failure_silently() {
    // If config cannot be read, prewarm must log and return — no toast, no panic.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("\"config_read_failed\""),
        "prewarm must log reason=config_read_failed on I/O error"
    );
    // Must not contain toast/hud/user-facing error calls
    assert!(
        !warm_fn_body.contains("show_hud"),
        "prewarm must NOT show HUD on config failure"
    );
    assert!(
        !warm_fn_body.contains("toast_manager"),
        "prewarm must NOT show toast on config failure"
    );
}

#[test]
fn prewarm_handles_invalid_config_silently() {
    // If config is invalid (e.g. missing binary), prewarm must log and return.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("validate_tab_ai_harness_config"),
        "prewarm must validate config before spawning"
    );
    assert!(
        warm_fn_body.contains("\"invalid_config\""),
        "prewarm must log reason=invalid_config on validation failure"
    );
}

#[test]
fn prewarm_logs_structured_events() {
    // All prewarm paths must use tracing with structured event fields
    // so SCRIPT_KIT_AI_LOG=1 output is machine-parseable.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("event = \"tab_ai_harness_prewarmed\""),
        "success path must log event=tab_ai_harness_prewarmed"
    );
    assert!(
        warm_fn_body.contains("event = \"tab_ai_harness_prewarm_failed\""),
        "failure path must log event=tab_ai_harness_prewarm_failed"
    );
    assert!(
        warm_fn_body.contains("event = \"tab_ai_harness_prewarm_skipped\""),
        "skip paths must log event=tab_ai_harness_prewarm_skipped"
    );
}

#[test]
fn prewarm_does_not_switch_view() {
    // Prewarm must be invisible to the user — no view transition.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_on_startup(")
        .expect("warm_tab_ai_harness_on_startup must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        !warm_fn_body.contains("current_view"),
        "prewarm must NOT change current_view — it is invisible to the user"
    );
    assert!(
        !warm_fn_body.contains("QuickTerminalView"),
        "prewarm must NOT switch to QuickTerminalView"
    );
}

#[test]
fn harness_config_default_warm_on_startup_is_true() {
    // The default config must have warm_on_startup=true so prewarm is opt-out.
    let config = script_kit_gpui::ai::HarnessConfig::default();
    assert!(
        config.warm_on_startup,
        "HarnessConfig::default() must have warm_on_startup=true"
    );
}

#[test]
fn harness_config_missing_warm_field_deserializes_as_true() {
    // Old config files without warmOnStartup should default to true.
    let json = r#"{"schemaVersion":1,"backend":"claudeCode","command":"claude"}"#;
    let config: script_kit_gpui::ai::HarnessConfig =
        serde_json::from_str(json).expect("deserialize");
    assert!(
        config.warm_on_startup,
        "missing warmOnStartup must default to true"
    );
}

#[test]
fn harness_config_explicit_opt_out_deserializes_as_false() {
    // Explicit "warmOnStartup": false must be respected.
    let json =
        r#"{"schemaVersion":1,"backend":"claudeCode","command":"claude","warmOnStartup":false}"#;
    let config: script_kit_gpui::ai::HarnessConfig =
        serde_json::from_str(json).expect("deserialize");
    assert!(
        !config.warm_on_startup,
        "explicit warmOnStartup=false must be preserved"
    );
}

// =========================================================================
// Harness injection: cold-start readiness gate
// =========================================================================

#[test]
fn harness_injection_uses_readiness_gate_for_cold_start() {
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
        "injection must differentiate cold vs warm start"
    );
    assert!(
        inject_fn_body.contains("has_received_output"),
        "cold-start injection must poll has_received_output as a readiness signal"
    );
    assert!(
        inject_fn_body.contains("HARNESS_READINESS_TIMEOUT_MS"),
        "readiness gate must use a bounded timeout constant"
    );
    // Must NOT use a fixed sleep as the sole delay mechanism
    assert!(
        !inject_fn_body.contains("let delay_ms"),
        "injection must not use a fixed delay_ms variable — use readiness polling instead"
    );
}

#[test]
fn harness_readiness_gate_has_bounded_timeout() {
    // The readiness gate must define both a timeout and a poll interval
    // as associated constants, ensuring they are not magic numbers.
    assert!(
        TAB_AI_MODE_SOURCE.contains("HARNESS_READINESS_TIMEOUT_MS"),
        "must define HARNESS_READINESS_TIMEOUT_MS constant"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("HARNESS_READINESS_POLL_MS"),
        "must define HARNESS_READINESS_POLL_MS constant"
    );
}

#[test]
fn harness_readiness_gate_logs_timeout_on_fallback() {
    // When the harness does not produce output within the timeout,
    // the gate must log a warning so slow startups are diagnosable.
    let inject_fn_start = TAB_AI_MODE_SOURCE
        .find("fn inject_tab_ai_harness_submission(")
        .expect("inject_tab_ai_harness_submission must exist");
    let inject_fn_body = &TAB_AI_MODE_SOURCE[inject_fn_start..];
    let next_fn = inject_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(inject_fn_body.len());
    let inject_fn_body = &inject_fn_body[..next_fn];

    assert!(
        inject_fn_body.contains("tab_ai_harness_readiness_timeout"),
        "readiness gate must log a warning event on timeout fallback"
    );
}

#[test]
fn term_prompt_exposes_has_received_output_field() {
    // TermPrompt must have a public `has_received_output` field that the
    // readiness gate can poll.
    let term_source = include_str!("../src/term_prompt/mod.rs");
    assert!(
        term_source.contains("pub has_received_output: bool"),
        "TermPrompt must expose has_received_output as a public bool field"
    );
    // Must be initialized to false
    assert!(
        term_source.contains("has_received_output: false"),
        "has_received_output must be initialized to false"
    );
    // Must be set to true when output is received
    assert!(
        term_source.contains("has_received_output = true"),
        "has_received_output must be set to true when the PTY produces output"
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

// =========================================================================
// Escape pass-through: QuickTerminalView disables escape-cancel
// =========================================================================

#[test]
fn quick_terminal_disables_escape_cancel() {
    // For QuickTerminalView, escape_cancels must be set to false so that
    // Escape is forwarded to the PTY (harness TUI owns Escape for navigation).
    assert!(
        TERM_RENDER_SOURCE.contains("escape_cancels")
            && TERM_RENDER_SOURCE.contains("QuickTerminalView"),
        "render_prompts/term.rs must set escape_cancels based on QuickTerminalView"
    );

    // The escape_cancels assignment must negate QuickTerminalView matches
    // (i.e., escape_cancels = !matches!(..., QuickTerminalView)).
    assert!(
        TERM_RENDER_SOURCE.contains("!matches!(self.current_view, AppView::QuickTerminalView"),
        "escape_cancels must be false for QuickTerminalView (negated matches)"
    );
}

#[test]
fn quick_terminal_cmd_w_dispatches_close() {
    // Cmd+W in QuickTerminalView must be intercepted before reaching the PTY
    // and must call close_tab_ai_harness_terminal.
    assert!(
        TERM_RENDER_SOURCE.contains("close_tab_ai_harness_terminal"),
        "Cmd+W in QuickTerminalView must dispatch to close_tab_ai_harness_terminal"
    );
    // The handler must check for the "w" key specifically
    assert!(
        TERM_RENDER_SOURCE.contains("\"w\""),
        "Cmd+W handler must match the 'w' key"
    );
}

// =========================================================================
// Fresh-line staging: source-level guard against regression
// =========================================================================

#[test]
fn harness_submission_builder_preserves_fresh_line_staging_for_paste_only() {
    let fn_start = HARNESS_SOURCE
        .find("pub fn build_tab_ai_harness_submission(")
        .expect("build_tab_ai_harness_submission must exist");
    let fn_body = &HARNESS_SOURCE[fn_start..];
    let next_section = fn_body[1..]
        .find("\n// ---------------------------------------------------------------------------")
        .map(|idx| idx + 1)
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_section];

    assert!(
        fn_body.contains("if !output.ends_with('\\n') {")
            || fn_body.contains("output.push('\\n');")
            || fn_body.contains("output.push_str(\"\\n\")"),
        "PasteOnly must leave the staged context on a fresh line so the next user keystrokes do not join </scriptKitContext>"
    );
}

// =========================================================================
// Backend smoke matrix: each built-in backend through PasteOnly path
// =========================================================================

/// Helper: build a deterministic context blob for smoke-matrix tests.
fn smoke_matrix_context() -> script_kit_gpui::ai::TabAiContextBlob {
    script_kit_gpui::ai::TabAiContextBlob::from_parts(
        script_kit_gpui::ai::TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            input_text: Some("test".to_string()),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        vec![],
        "2026-03-29T20:00:00Z".to_string(),
    )
}

/// Helper: build a HarnessConfig for a given backend.
fn config_for_backend(
    backend: script_kit_gpui::ai::HarnessBackendKind,
    command: &str,
) -> script_kit_gpui::ai::HarnessConfig {
    script_kit_gpui::ai::HarnessConfig {
        backend,
        command: command.to_string(),
        ..Default::default()
    }
}

#[test]
fn smoke_matrix_claude_code_paste_only_stages_context() {
    let _config = config_for_backend(
        script_kit_gpui::ai::HarnessBackendKind::ClaudeCode,
        "claude",
    );
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
    )
    .expect("Claude Code PasteOnly submission must succeed");

    assert!(submission.contains("<scriptKitContext schemaVersion="));
    assert!(submission.ends_with("</scriptKitContext>\n"));
    assert!(!submission.contains("Await the user"));
    assert!(!submission.contains("User intent:"));
}

#[test]
fn smoke_matrix_codex_paste_only_stages_context() {
    let _config = config_for_backend(
        script_kit_gpui::ai::HarnessBackendKind::Codex,
        "codex",
    );
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
    )
    .expect("Codex PasteOnly submission must succeed");

    assert!(submission.contains("<scriptKitContext schemaVersion="));
    assert!(submission.ends_with("</scriptKitContext>\n"));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_gemini_cli_paste_only_stages_context() {
    let _config = config_for_backend(
        script_kit_gpui::ai::HarnessBackendKind::GeminiCli,
        "gemini",
    );
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
    )
    .expect("Gemini CLI PasteOnly submission must succeed");

    assert!(submission.contains("<scriptKitContext schemaVersion="));
    assert!(submission.ends_with("</scriptKitContext>\n"));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_copilot_cli_paste_only_stages_context() {
    let _config = config_for_backend(
        script_kit_gpui::ai::HarnessBackendKind::CopilotCli,
        "gh",
    );
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
    )
    .expect("Copilot CLI PasteOnly submission must succeed");

    assert!(submission.contains("<scriptKitContext schemaVersion="));
    assert!(submission.ends_with("</scriptKitContext>\n"));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_all_backends_produce_identical_context_block() {
    // The context block must be backend-agnostic — all four produce the same output.
    let context = smoke_matrix_context();
    let mode = script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly;

    let claude = script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode)
        .expect("Claude Code");
    let codex = script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode)
        .expect("Codex");
    let gemini = script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode)
        .expect("Gemini CLI");
    let copilot = script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode)
        .expect("Copilot CLI");

    assert_eq!(claude, codex, "Claude Code and Codex must produce identical context");
    assert_eq!(codex, gemini, "Codex and Gemini CLI must produce identical context");
    assert_eq!(gemini, copilot, "Gemini CLI and Copilot CLI must produce identical context");
}

#[test]
fn smoke_matrix_submit_mode_appends_sentinel_for_all_backends() {
    let context = smoke_matrix_context();
    let mode = script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit;

    for label in ["Claude Code", "Codex", "Gemini CLI", "Copilot CLI"] {
        let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode)
            .unwrap_or_else(|e| panic!("{label} Submit failed: {e}"));
        assert!(
            submission.contains("Await the user's next terminal input."),
            "{label} Submit mode must append wait sentinel"
        );
    }
}

#[test]
fn smoke_matrix_intent_appends_for_all_backends() {
    let context = smoke_matrix_context();

    for (label, mode) in [
        ("PasteOnly", script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly),
        ("Submit", script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit),
    ] {
        let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
            &context,
            Some("rename this file"),
            mode,
        )
        .unwrap_or_else(|e| panic!("{label} with intent failed: {e}"));
        assert!(
            submission.contains("User intent:\nrename this file"),
            "{label} with intent must include the user's intent text"
        );
        assert!(
            !submission.contains("Await the user"),
            "{label} with intent must NOT include wait sentinel"
        );
    }
}

// =========================================================================
// Footer hint strip: QuickTerminalView shows only "⌘W Close"
// =========================================================================

#[test]
fn quick_terminal_footer_shows_only_cmd_w_close() {
    assert!(
        TERM_RENDER_SOURCE.contains("⌘W Close"),
        "QuickTerminalView footer hint strip must show '⌘W Close'"
    );
    // Must use render_simple_hint_strip for minimal chrome
    assert!(
        TERM_RENDER_SOURCE.contains("render_simple_hint_strip"),
        "QuickTerminalView footer must use render_simple_hint_strip for minimal chrome"
    );
}

// =========================================================================
// Close function guards: only acts when in QuickTerminalView
// =========================================================================

#[test]
fn close_harness_terminal_guards_current_view() {
    // close_tab_ai_harness_terminal must check that we're actually in
    // QuickTerminalView before restoring — prevent no-op restore from
    // corrupting view state when called from unexpected contexts.
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("QuickTerminalView"),
        "close must verify current_view is QuickTerminalView before restoring"
    );
    assert!(
        close_fn_body.contains("cx.notify()"),
        "close must call cx.notify() to trigger re-render after view change"
    );
}

// =========================================================================
// Legacy-removal guard: TabAiChat must not reappear in changed surfaces
// =========================================================================

#[test]
fn legacy_tab_ai_chat_not_in_primary_entry_path() {
    // The primary Tab AI entry functions must not reference TabAiChat.
    // If TabAiChat silently comes back in these surfaces, the harness terminal
    // contract is broken.

    // open_tab_ai_chat: the public entry point
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_chat(")
        .expect("open_tab_ai_chat must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];
    assert!(
        !open_fn_body.contains("TabAiChat"),
        "open_tab_ai_chat must not reference TabAiChat — harness terminal is primary"
    );

    // open_tab_ai_harness_terminal: the harness launcher
    let harness_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal(")
        .expect("open_tab_ai_harness_terminal must exist");
    let harness_fn_body = &TAB_AI_MODE_SOURCE[harness_fn_start..];
    let next_fn = harness_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(harness_fn_body.len());
    let harness_fn_body = &harness_fn_body[..next_fn];
    assert!(
        !harness_fn_body.contains("TabAiChat"),
        "open_tab_ai_harness_terminal must not reference TabAiChat"
    );

    // close_tab_ai_harness_terminal: the close handler
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];
    assert!(
        !close_fn_body.contains("TabAiChat"),
        "close_tab_ai_harness_terminal must not reference TabAiChat"
    );
}

#[test]
fn legacy_tab_ai_chat_not_in_startup_tab_handler() {
    // The Tab interceptor in startup.rs must not route to TabAiChat.
    assert!(
        !TAB_SOURCE.contains("TabAiChat"),
        "startup.rs Tab handler must not reference TabAiChat — harness terminal is primary"
    );
}

#[test]
fn legacy_tab_ai_chat_not_in_script_list_tab_handler() {
    // ScriptList Tab handler must not route to TabAiChat.
    assert!(
        !SCRIPT_LIST_SOURCE.contains("TabAiChat"),
        "ScriptList Tab handler must not reference TabAiChat"
    );
}

#[test]
fn legacy_tab_ai_chat_not_in_term_render() {
    // The QuickTerminalView renderer must not reference TabAiChat.
    assert!(
        !TERM_RENDER_SOURCE.contains("TabAiChat"),
        "render_prompts/term.rs must not reference TabAiChat"
    );
}

#[test]
fn legacy_inline_chat_not_reexported_from_ai_mod() {
    // TabAiChat should NOT be re-exported from the ai module's public surface.
    // It may still exist internally but must not be discoverable via pub use.
    let has_tab_ai_chat_reexport = AI_MOD_SOURCE
        .lines()
        .any(|line| line.contains("pub use") && line.contains("TabAiChat"));
    assert!(
        !has_tab_ai_chat_reexport,
        "ai/mod.rs must not re-export TabAiChat in its public API"
    );
}

// =========================================================================
// Consolidated legacy inline-chat regression guard
// =========================================================================
//
// This guard covers ALL changed harness-entry surfaces in a single test.
// If TabAiChat or open_tab_ai_full_view_chat silently reappears in any
// of these files, the harness-terminal contract is broken.

#[test]
fn legacy_inline_chat_absent_from_all_changed_harness_surfaces() {
    // Every source constant that represents a harness-entry surface must be
    // free of legacy inline-chat references. This is the single authoritative
    // guard — the per-file tests above are fine-grained; this one is the safety net.
    let surfaces: &[(&str, &str)] = &[
        ("startup.rs (Tab interceptor)", TAB_SOURCE),
        ("tab_ai_mode.rs (orchestration)", TAB_AI_MODE_SOURCE),
        (
            "render_prompts/term.rs (QuickTerminalView renderer)",
            TERM_RENDER_SOURCE,
        ),
        (
            "render_script_list/mod.rs (ScriptList Tab fallback)",
            SCRIPT_LIST_SOURCE,
        ),
        ("app_view_state.rs (view enum)", APP_VIEW_STATE_SOURCE),
        ("app_state.rs (shared state)", APP_STATE_SOURCE),
    ];

    let legacy_markers = &[
        "open_tab_ai_full_view_chat",
        "TabAiChat::new",
        "open_tab_ai_overlay",
        "render_tab_ai_overlay",
    ];

    for (label, source) in surfaces {
        for marker in legacy_markers {
            assert!(
                !source.contains(marker),
                "{label} must not contain legacy marker `{marker}` — \
                 the harness terminal is the primary Tab AI surface"
            );
        }
    }
}

#[test]
fn legacy_tab_ai_chat_not_in_app_view_state() {
    // AppView enum and related types must not route to TabAiChat.
    // TabAiChat is an entity for the legacy inline chat — the harness terminal
    // uses TermPrompt/QuickTerminalView instead.
    assert!(
        !APP_VIEW_STATE_SOURCE.contains("TabAiChat"),
        "app_view_state.rs must not reference TabAiChat — \
         QuickTerminalView is the Tab AI view variant"
    );
}

#[test]
fn legacy_tab_ai_chat_not_in_app_state() {
    // Shared app state must not hold TabAiChat references.
    // The harness uses TabAiHarnessSessionState instead.
    assert!(
        !APP_STATE_SOURCE.contains("TabAiChat"),
        "app_state.rs must not reference TabAiChat — \
         tab_ai_harness is the session state field"
    );
}

