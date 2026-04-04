//! Contract tests verifying Tab AI routes to the harness terminal surface.
//!
//! The primary Tab AI entry path is:
//!   Tab key → `open_tab_ai_chat()` → `open_tab_ai_harness_terminal_from_request()` →
//!   `AppView::QuickTerminalView` rendered via `TermPrompt`.
//!
//! Tests in this file validate that the harness-terminal contract is the
//! authoritative Tab AI surface.

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const TAB_NEW_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const RENDER_IMPL_SOURCE: &str = include_str!("../src/main_sections/render_impl.rs");
const SCRIPT_LIST_SOURCE: &str = include_str!("../src/render_script_list/mod.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const HARNESS_SOURCE: &str = include_str!("../src/ai/harness/mod.rs");
const TERM_RENDER_SOURCE: &str = include_str!("../src/render_prompts/term.rs");
const AI_MOD_SOURCE: &str = include_str!("../src/ai/mod.rs");
const BUILTIN_EXECUTION_SOURCE: &str = include_str!("../src/app_execute/builtin_execution.rs");
const PROMPT_AI_SOURCE: &str = include_str!("../src/app_impl/prompt_ai.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const SCREENSHOT_FILES_SOURCE: &str = include_str!("../src/ai/harness/screenshot_files.rs");

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
        open_fn_body.contains("open_tab_ai_chat_with_entry_intent(None, cx)")
            || open_fn_body.contains("begin_tab_ai_harness_entry("),
        "open_tab_ai_chat must delegate to the harness terminal as the primary surface"
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
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("AppView::QuickTerminalView"),
        "open_tab_ai_harness_terminal_from_request must switch to QuickTerminalView"
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
fn tab_ai_routing_shift_tab_routes_through_harness() {
    assert!(
        TAB_SOURCE.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab must route typed queries through the quick-submit planner"
    );
    assert!(
        !TAB_SOURCE.contains("dispatch_ai_script_generation_from_query(query, cx)"),
        "Shift+Tab must not use the legacy script generation bypass"
    );

    let shift_tab_pos = TAB_SOURCE
        .find("submit_to_current_or_new_tab_ai_harness_from_text")
        .expect("Shift+Tab quick-submit route must exist");
    let ai_pos = TAB_SOURCE
        .find("open_tab_ai_chat(cx)")
        .expect("open_tab_ai_chat must exist");

    assert!(
        shift_tab_pos < ai_pos,
        "Shift+Tab harness route must come before universal Tab AI route"
    );
}

// =========================================================================
// Shift+Tab ScriptList routing: both startup interceptors
// =========================================================================

/// Assert that a startup interceptor file gates the Shift+Tab ScriptList
/// path on the expected preconditions and routes through the harness
/// entry-intent path (not the legacy inline script-generation dispatch).
fn assert_shift_tab_script_list_routes_query_to_harness_entry_intent(source: &str, label: &str) {
    assert!(
        source.contains("has_shift")
            && source.contains("matches!(this.current_view, AppView::ScriptList)")
            && source.contains("!this.filter_text.is_empty()")
            && source.contains("!this.show_actions_popup"),
        "{label} must guard the Shift+Tab ScriptList path with the expected preconditions",
    );
    assert!(
        source.contains("let query = this.filter_text.clone();"),
        "{label} must clone the ScriptList filter text before routing",
    );
    assert!(
        source.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "{label} must route the typed query through the quick-submit planner",
    );
    assert!(
        !source.contains("dispatch_ai_script_generation_from_query(query, cx);"),
        "{label} must not call the legacy inline script-generation path anymore",
    );
}

#[test]
fn startup_shift_tab_script_list_routes_query_to_harness_entry_intent() {
    assert_shift_tab_script_list_routes_query_to_harness_entry_intent(TAB_SOURCE, "startup.rs");
}

#[test]
fn startup_new_tab_shift_tab_script_list_routes_query_to_harness_entry_intent() {
    assert_shift_tab_script_list_routes_query_to_harness_entry_intent(
        TAB_NEW_SOURCE,
        "startup_new_tab.rs",
    );
}

// =========================================================================
// Entry-intent normalization and submit mode selection
// =========================================================================

#[test]
fn entry_intent_is_trimmed_before_submit_mode_is_selected() {
    assert!(
        TAB_AI_MODE_SOURCE.contains(".map(|value| value.trim().to_string())"),
        "entry intent must be trimmed before mode selection",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains(".filter(|value| !value.is_empty())"),
        "whitespace-only entry intent must collapse to None",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("let submit_now = request"),
        "submission mode must derive from quick_submit_plan or entry_intent",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("let submission_mode = if submit_now"),
        "submission mode must branch on the resolved submit_now flag",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("crate::ai::TabAiHarnessSubmissionMode::Submit"),
        "non-empty normalized entry intent must use Submit mode",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("crate::ai::TabAiHarnessSubmissionMode::PasteOnly"),
        "empty normalized entry intent must use PasteOnly mode",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("effective_intent"),
        "quick-submit plan's synthesized_intent must be preferred over raw entry_intent",
    );
}

// =========================================================================
// Tab re-entry guards
// =========================================================================

#[test]
fn startup_tab_interceptor_does_not_steal_tab_from_quick_terminal() {
    assert!(
        TAB_SOURCE.contains("AppView::QuickTerminalView"),
        "startup.rs must special-case QuickTerminalView"
    );
    // The QuickTerminalView Tab handler writes raw bytes directly to the PTY
    // and stops propagation so GPUI focus traversal does not consume the Tab.
    assert!(
        TAB_SOURCE.contains("term.terminal.input(bytes)"),
        "QuickTerminalView Tab handling must write raw bytes to the PTY"
    );
    assert!(
        TAB_SOURCE.contains("b\"\\t\""),
        "plain Tab must be written as a tab byte"
    );
    assert!(
        TAB_SOURCE.contains("b\"\\x1b[Z\""),
        "Shift+Tab must be written as a backtab escape sequence"
    );
    assert!(
        TAB_SOURCE.contains("cx.stop_propagation();"),
        "QuickTerminalView Tab handling must stop propagation so GPUI focus traversal does not consume the key"
    );
}

#[test]
fn startup_new_tab_interceptor_writes_tab_bytes_directly_to_quick_terminal_pty() {
    assert!(
        TAB_NEW_SOURCE.contains("AppView::QuickTerminalView"),
        "startup_new_tab.rs must special-case QuickTerminalView"
    );
    assert!(
        TAB_NEW_SOURCE.contains("term.terminal.input(bytes)"),
        "QuickTerminalView Tab handling must write raw bytes to the PTY"
    );
    assert!(
        TAB_NEW_SOURCE.contains("b\"\\t\""),
        "plain Tab must be written as a tab byte"
    );
    assert!(
        TAB_NEW_SOURCE.contains("b\"\\x1b[Z\""),
        "Shift+Tab must be written as a backtab escape sequence"
    );
    assert!(
        TAB_NEW_SOURCE.contains("cx.stop_propagation();"),
        "QuickTerminalView Tab handling must stop propagation so GPUI focus traversal does not consume the key"
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
    // The deferred capture pipeline must use tab_ai_submit() (no screenshots
    // in the blob) not tab_ai() (includes base64 screenshots that bloat PTY).
    let capture_fn_start = TAB_AI_MODE_SOURCE
        .find("fn spawn_tab_ai_pre_switch_capture(")
        .expect("spawn_tab_ai_pre_switch_capture must exist");
    let capture_fn_body = &TAB_AI_MODE_SOURCE[capture_fn_start..];
    let next_fn = capture_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(capture_fn_body.len());
    let capture_fn_body = &capture_fn_body[..next_fn];

    assert!(
        capture_fn_body.contains("tab_ai_submit()"),
        "deferred capture must use tab_ai_submit() (text-safe, no screenshots) for generic PTY"
    );
    assert!(
        !capture_fn_body.contains("CaptureContextOptions::tab_ai()"),
        "deferred capture must NOT use tab_ai() (includes screenshots) for generic PTY backends"
    );
}

#[test]
fn harness_terminal_entry_derives_mode_from_intent() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("PasteOnly"),
        "Zero-intent Tab entry must use PasteOnly so context is staged without auto-submitting"
    );
    assert!(
        open_fn_body.contains("let submit_now = request"),
        "tab_ai_mode.rs must derive submit flag from quick_submit_plan or entry_intent"
    );
    assert!(
        open_fn_body.contains("let submission_mode = if submit_now"),
        "tab_ai_mode.rs must derive submission mode from resolved submit flag"
    );
    assert!(
        open_fn_body.contains("TabAiHarnessSubmissionMode::Submit"),
        "Typed Tab entry must submit immediately through the harness"
    );
    assert!(
        open_fn_body.contains("effective_intent"),
        "Typed Tab entry must prefer quick-submit synthesized intent over raw entry_intent"
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
        HARNESS_SOURCE.contains("Script Kit context"),
        "harness submission must contain the flat labeled context header"
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
        err.contains("config.ts"),
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
        err.contains("config.ts"),
        "error must mention config file for setup guidance: {err}"
    );
}

#[test]
fn harness_startup_failure_toast_mentions_setup_guidance() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("config.ts"),
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
        TERM_RENDER_SOURCE.contains("wrapper semantics contract"),
        "render_prompts/term.rs must document the wrapper semantics contract"
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

#[test]
fn close_tab_ai_harness_terminal_clears_cached_session() {
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("self.tab_ai_harness.take()"),
        "close must clear cached harness session via take()"
    );
    assert!(
        close_fn_body.contains("terminate_session"),
        "close must tear down the PTY session via terminate_session()"
    );
    assert!(
        close_fn_body.contains("tab_ai_harness_capture_generation += 1"),
        "close must invalidate in-flight deferred capture generations"
    );
    assert!(
        close_fn_body.contains("schedule_tab_ai_harness_prewarm"),
        "close must schedule a fresh prewarm after teardown via schedule_tab_ai_harness_prewarm"
    );
}

#[test]
fn close_tab_ai_harness_logs_session_cleared() {
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("session_cleared = true"),
        "close log must include session_cleared = true for observability"
    );
}

#[test]
fn term_prompt_exposes_terminate_session() {
    let term_source = include_str!("../src/term_prompt/mod.rs");
    assert!(
        term_source.contains("pub fn terminate_session(&mut self) -> anyhow::Result<()>"),
        "TermPrompt must expose a public terminate_session method"
    );
    // Must not use unwrap/expect in production code
    let term_fn_start = term_source
        .find("pub fn terminate_session(")
        .expect("terminate_session must exist");
    let term_fn_body = &term_source[term_fn_start..];
    let next_fn = term_fn_body[1..]
        .find("\n    pub fn ")
        .unwrap_or(term_fn_body.len());
    let term_fn_body = &term_fn_body[..next_fn];

    assert!(
        !term_fn_body.contains(".unwrap()") && !term_fn_body.contains(".expect("),
        "terminate_session must not use unwrap/expect in production code"
    );
}

#[test]
fn schedule_tab_ai_harness_prewarm_exists_and_uses_timer() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn schedule_tab_ai_harness_prewarm("),
        "schedule_tab_ai_harness_prewarm helper must exist"
    );
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn schedule_tab_ai_harness_prewarm(")
        .expect("schedule_tab_ai_harness_prewarm must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..]
        .find("\n    fn ")
        .or_else(|| fn_body[1..].find("\n    pub"))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("background_executor().timer("),
        "prewarm scheduler must use background_executor timer for the delay"
    );
    // The scheduler delegates to the post-close prewarm helper, which
    // internally calls warm_tab_ai_harness_silently.
    assert!(
        fn_body.contains("warm_tab_ai_harness_after_close"),
        "prewarm scheduler must delegate to warm_tab_ai_harness_after_close"
    );
    assert!(
        fn_body.contains(".detach()"),
        "prewarm task must be detached"
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
// ScriptList fallback comment matches quick-terminal contract
// =========================================================================

#[test]
fn script_list_tab_fallback_comment_matches_quick_terminal_contract() {
    assert!(
        SCRIPT_LIST_SOURCE.contains("route to Tab AI quick terminal (harness surface)."),
        "render_script_list fallback comment must describe the quick terminal harness surface"
    );
    assert!(
        !SCRIPT_LIST_SOURCE.contains("route to Tab AI full-view chat"),
        "render_script_list fallback comment must not describe the legacy full-view chat"
    );
    assert!(
        SCRIPT_LIST_SOURCE.contains("this.open_tab_ai_chat(cx);"),
        "render_script_list fallback must still route through open_tab_ai_chat"
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
    // The warm_on_startup check lives in warm_tab_ai_harness_silently,
    // which warm_tab_ai_harness_on_startup delegates to.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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
    // Prewarm must call the same session constructor that Tab uses.
    // The logic lives in warm_tab_ai_harness_silently, which both
    // warm_tab_ai_harness_on_startup and warm_tab_ai_harness_after_close delegate to.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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
    // The logic lives in warm_tab_ai_harness_silently.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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
    // The logic lives in warm_tab_ai_harness_silently.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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
    // The logic lives in warm_tab_ai_harness_silently.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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
    // The logic lives in warm_tab_ai_harness_silently.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
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

// =========================================================================
// Post-close prewarm lifecycle: close clears session, prewarm seeds startup,
// explicit Tab still forces a fresh PTY
// =========================================================================

#[test]
fn open_path_reuses_fresh_prewarm_once() {
    // First explicit Tab after a silent prewarm reuses the warm PTY once;
    // subsequent opens force a fresh session via !reuse_fresh_prewarm.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("is_fresh_prewarm"),
        "open path must check for a fresh silently-prewarmed session"
    );
    assert!(
        open_fn_body.contains("mark_consumed"),
        "open path must consume a fresh prewarm exactly once"
    );
    assert!(
        open_fn_body.contains("!reuse_fresh_prewarm"),
        "open path must pass !reuse_fresh_prewarm to ensure_tab_ai_harness_terminal"
    );
    // FreshPrewarm is used by both the prewarm and open paths.
    assert!(
        TAB_AI_MODE_SOURCE.contains("FreshPrewarm"),
        "tab ai lifecycle must distinguish prewarmed sessions"
    );
}

#[test]
fn prewarm_tags_cold_start_as_fresh_prewarm() {
    // After a cold-start prewarm, the session must be tagged FreshPrewarm.
    // The logic lives in warm_tab_ai_harness_silently.
    let warm_fn_start = TAB_AI_MODE_SOURCE
        .find("fn warm_tab_ai_harness_silently(")
        .expect("warm_tab_ai_harness_silently must exist");
    let warm_fn_body = &TAB_AI_MODE_SOURCE[warm_fn_start..];
    let next_fn = warm_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| warm_fn_body[1..].find("\n    pub"))
        .unwrap_or(warm_fn_body.len());
    let warm_fn_body = &warm_fn_body[..next_fn];

    assert!(
        warm_fn_body.contains("mark_fresh_prewarm"),
        "prewarm must tag cold-start sessions via mark_fresh_prewarm()"
    );
    assert!(
        warm_fn_body.contains("was_cold_start"),
        "prewarm must only tag FreshPrewarm when was_cold_start is true"
    );
}

#[test]
fn open_path_reuses_fresh_prewarm_once_then_forces_fresh() {
    // First explicit Tab after a silent prewarm reuses the warm PTY once;
    // subsequent opens force a fresh session.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("is_fresh_prewarm"),
        "open path must check for a fresh silently-prewarmed session"
    );
    assert!(
        open_fn_body.contains("mark_consumed"),
        "open path must consume a fresh prewarm exactly once"
    );
    assert!(
        open_fn_body.contains("!reuse_fresh_prewarm"),
        "open path must pass !reuse_fresh_prewarm to ensure_tab_ai_harness_terminal"
    );
}

#[test]
fn close_then_prewarm_then_reuse_once_lifecycle_contract() {
    // End-to-end contract: close clears session → prewarm seeds a warm PTY
    // → first explicit Tab reuses the warm PTY once → later opens force fresh.

    // 1. close_tab_ai_harness_terminal clears session and schedules prewarm
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .or_else(|| close_fn_body[1..].find("\n    pub"))
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("tab_ai_harness.take()"),
        "close must take (clear) the harness session"
    );
    assert!(
        close_fn_body.contains("schedule_tab_ai_harness_prewarm"),
        "close must schedule a deferred prewarm"
    );

    // 2. warm_tab_ai_harness_silently tags cold starts as FreshPrewarm
    assert!(
        TAB_AI_MODE_SOURCE.contains("mark_fresh_prewarm()"),
        "prewarm path must tag cold-start sessions via mark_fresh_prewarm()"
    );

    // 3. open path reuses a fresh prewarm once, then forces fresh
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("is_fresh_prewarm"),
        "open path must check for a fresh silently-prewarmed session"
    );
    assert!(
        open_fn_body.contains("mark_consumed"),
        "open path must consume the prewarm once"
    );
    assert!(
        open_fn_body.contains("!reuse_fresh_prewarm"),
        "open path must conditionally reuse: !reuse_fresh_prewarm → force_fresh"
    );
}

#[test]
fn warm_state_enum_has_both_variants() {
    // TabAiHarnessWarmState must exist in the harness source with both variants.
    assert!(
        HARNESS_SOURCE.contains("enum TabAiHarnessWarmState"),
        "TabAiHarnessWarmState enum must exist in harness/mod.rs"
    );
    assert!(
        HARNESS_SOURCE.contains("FreshPrewarm"),
        "TabAiHarnessWarmState must have FreshPrewarm variant"
    );
    assert!(
        HARNESS_SOURCE.contains("Consumed"),
        "TabAiHarnessWarmState must have Consumed variant"
    );
}

#[test]
fn session_state_has_warm_state_field() {
    // TabAiHarnessSessionState must include the warm_state field.
    assert!(
        HARNESS_SOURCE.contains("warm_state: TabAiHarnessWarmState"),
        "TabAiHarnessSessionState must have a warm_state field of type TabAiHarnessWarmState"
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
        inject_fn_body.contains("wait_for_readiness"),
        "injection must accept a readiness flag independent of cold/warm start"
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
// Harness readiness gate: prewarmed sessions also wait for output
// =========================================================================

#[test]
fn harness_readiness_gate_applies_to_prewarmed_sessions() {
    // A reused (prewarmed) session with has_received_output == false must
    // still take the readiness-wait path before context paste. The gate
    // must NOT be conditioned on was_cold_start alone.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    // The open function must call the readiness check method
    assert!(
        open_fn_body.contains("tab_ai_harness_needs_readiness_wait"),
        "open_tab_ai_harness_terminal_from_request must call tab_ai_harness_needs_readiness_wait \
         to determine readiness independent of cold-start status"
    );

    // The readiness check must be based on has_received_output, not was_cold_start
    assert!(
        open_fn_body.contains("wait_for_readiness"),
        "open_tab_ai_harness_terminal_from_request must use wait_for_readiness from the readiness check"
    );
}

#[test]
fn harness_readiness_check_method_exists_and_checks_output() {
    // tab_ai_harness_needs_readiness_wait must exist and check has_received_output
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn tab_ai_harness_needs_readiness_wait("),
        "tab_ai_harness_needs_readiness_wait method must exist"
    );

    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_harness_needs_readiness_wait(")
        .expect("method must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("has_received_output"),
        "readiness check must be based on has_received_output"
    );
}

#[test]
fn harness_injection_uses_wait_for_readiness_not_cold_start() {
    // The injection function must use wait_for_readiness parameter,
    // not was_cold_start, ensuring prewarmed sessions also wait.
    let inject_fn_start = TAB_AI_MODE_SOURCE
        .find("fn inject_tab_ai_harness_submission(")
        .expect("inject_tab_ai_harness_submission must exist");
    let inject_fn_body = &TAB_AI_MODE_SOURCE[inject_fn_start..];
    let next_fn = inject_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(inject_fn_body.len());
    let inject_fn_body = &inject_fn_body[..next_fn];

    assert!(
        inject_fn_body.contains("wait_for_readiness: bool"),
        "injection must accept wait_for_readiness parameter"
    );
    assert!(
        !inject_fn_body.contains("was_cold_start: bool"),
        "injection must NOT use was_cold_start — readiness is output-based"
    );
}

#[test]
fn harness_submission_planned_event_logged() {
    // open_tab_ai_harness_terminal_from_request must log a structured submission_planned event
    // with the wait_for_readiness flag for diagnostics.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("tab_ai_harness_submission_planned"),
        "must log tab_ai_harness_submission_planned event"
    );
    assert!(
        open_fn_body.contains("wait_for_readiness"),
        "submission_planned event must include wait_for_readiness field"
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
        "PasteOnly must leave the staged context on a fresh line so the next user keystrokes do not join the context block"
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
        None,
        None,
        &[],
    )
    .expect("Claude Code PasteOnly submission must succeed");

    assert!(submission.contains("Script Kit context"));
    assert!(submission.ends_with('\n'));
    assert!(!submission.contains("Await the user"));
    assert!(!submission.contains("User intent:"));
}

#[test]
fn smoke_matrix_codex_paste_only_stages_context() {
    let _config = config_for_backend(script_kit_gpui::ai::HarnessBackendKind::Codex, "codex");
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("Codex PasteOnly submission must succeed");

    assert!(submission.contains("Script Kit context"));
    assert!(submission.ends_with('\n'));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_gemini_cli_paste_only_stages_context() {
    let _config = config_for_backend(script_kit_gpui::ai::HarnessBackendKind::GeminiCli, "gemini");
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("Gemini CLI PasteOnly submission must succeed");

    assert!(submission.contains("Script Kit context"));
    assert!(submission.ends_with('\n'));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_copilot_cli_paste_only_stages_context() {
    let _config = config_for_backend(script_kit_gpui::ai::HarnessBackendKind::CopilotCli, "gh");
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("Copilot CLI PasteOnly submission must succeed");

    assert!(submission.contains("Script Kit context"));
    assert!(submission.ends_with('\n'));
    assert!(!submission.contains("Await the user"));
}

#[test]
fn smoke_matrix_all_backends_produce_identical_context_block() {
    // The context block must be backend-agnostic — all four produce the same output.
    let context = smoke_matrix_context();
    let mode = script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly;

    let claude =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("Claude Code");
    let codex =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("Codex");
    let gemini =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("Gemini CLI");
    let copilot =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("Copilot CLI");

    assert_eq!(
        claude, codex,
        "Claude Code and Codex must produce identical context"
    );
    assert_eq!(
        codex, gemini,
        "Codex and Gemini CLI must produce identical context"
    );
    assert_eq!(
        gemini, copilot,
        "Gemini CLI and Copilot CLI must produce identical context"
    );
}

#[test]
fn smoke_matrix_submit_mode_appends_sentinel_for_all_backends() {
    let context = smoke_matrix_context();
    let mode = script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit;

    for label in ["Claude Code", "Codex", "Gemini CLI", "Copilot CLI"] {
        let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
            &context,
            None,
            mode,
            None,
            None,
            &[],
        )
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
        (
            "PasteOnly",
            script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
        ),
        (
            "Submit",
            script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit,
        ),
    ] {
        let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
            &context,
            Some("rename this file"),
            mode,
            None,
            None,
            &[],
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
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let harness_fn_body = &TAB_AI_MODE_SOURCE[harness_fn_start..];
    let next_fn = harness_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(harness_fn_body.len());
    let harness_fn_body = &harness_fn_body[..next_fn];
    assert!(
        !harness_fn_body.contains("TabAiChat"),
        "open_tab_ai_harness_terminal_from_request must not reference TabAiChat"
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

// =========================================================================
// Intent-aware harness entry: typed Tab queries route through harness
// =========================================================================

#[test]
fn startup_tab_interceptor_routes_nonempty_script_list_query_into_harness() {
    assert!(
        TAB_SOURCE.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "startup.rs must route non-empty ScriptList Shift+Tab queries through the quick-submit planner"
    );
    assert!(
        !TAB_SOURCE.contains("dispatch_ai_script_generation_from_query(query, cx)"),
        "startup.rs must not keep the legacy Script Kit AI generation Tab bypass"
    );
}

#[test]
fn typed_tab_entry_uses_submit_mode_in_harness() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn open_tab_ai_chat_with_entry_intent("),
        "tab_ai_mode.rs must expose an entry-intent-aware harness entry point"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("let submit_now = request"),
        "tab_ai_mode.rs must derive submit flag from quick_submit_plan or entry_intent"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("crate::ai::TabAiHarnessSubmissionMode::Submit"),
        "typed Tab entry must submit immediately through the harness"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("effective_intent"),
        "typed Tab entry must prefer quick-submit synthesized intent over raw entry_intent"
    );
}

#[test]
fn entry_intent_is_trimmed_and_empty_filtered() {
    // Whitespace-only intents must be treated as None (zero-intent).
    assert!(
        TAB_AI_MODE_SOURCE.contains(".map(|value| value.trim().to_string())"),
        "entry_intent must be trimmed"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains(".filter(|value| !value.is_empty())"),
        "empty trimmed intent must be filtered to None"
    );
}

// =========================================================================
// Legacy AI command redirection: all active AI builtins → harness terminal
// =========================================================================

#[test]
fn generate_script_builtin_routes_to_harness_not_chat_prompt() {
    // The GenerateScript match arm must open the harness terminal, not the legacy ChatPrompt.
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("ai_generate_script_routed_to_harness"),
        "GenerateScript must use harness routing success label"
    );
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("open_tab_ai_chat_with_entry_intent(Some(trimmed), cx)"),
        "GenerateScript must route typed query to harness entry intent"
    );
}

#[test]
fn send_to_ai_commands_route_to_harness_not_legacy_window() {
    // All SendXToAi commands must route to the harness terminal.
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("ai_{cmd_type:?}_routed_to_harness"),
        "Send-to-AI commands must use harness routing success label"
    );
    // The AI command match block must not invoke legacy hide-then-capture calls.
    // We scope to the AiCommand match block (starts at "AiCommand(cmd_type)").
    let ai_block_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::AiCommand(cmd_type) =>")
        .expect("AiCommand match arm must exist");
    // Grab ~4000 chars of the match block (enough to cover all arms).
    let ai_block = &BUILTIN_EXECUTION_SOURCE
        [ai_block_start..(ai_block_start + 4000).min(BUILTIN_EXECUTION_SOURCE.len())];

    assert!(
        !ai_block.contains("spawn_send_screen_to_ai_after_hide"),
        "AI command block must not call the legacy spawn_send_screen_to_ai_after_hide"
    );
    assert!(
        !ai_block.contains("spawn_send_focused_window_to_ai_after_hide"),
        "AI command block must not call the legacy spawn_send_focused_window_to_ai_after_hide"
    );
    assert!(
        !ai_block.contains("spawn_send_selected_text_to_ai_after_hide"),
        "AI command block must not call the legacy spawn_send_selected_text_to_ai_after_hide"
    );
    assert!(
        !ai_block.contains("spawn_send_browser_tab_to_ai_after_hide"),
        "AI command block must not call the legacy spawn_send_browser_tab_to_ai_after_hide"
    );
    assert!(
        !ai_block.contains("spawn_send_screen_area_to_ai_after_hide"),
        "AI command block must not call the legacy spawn_send_screen_area_to_ai_after_hide"
    );
}

#[test]
fn legacy_ai_window_commands_route_to_harness() {
    // OpenAi, MiniAi, NewConversation, ClearConversation must all route to harness.
    // Scope to the AI command match block.
    let ai_block_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::AiCommand(cmd_type) =>")
        .expect("AiCommand match arm must exist");
    let ai_block = &BUILTIN_EXECUTION_SOURCE
        [ai_block_start..(ai_block_start + 4000).min(BUILTIN_EXECUTION_SOURCE.len())];

    assert!(
        !ai_block.contains("open_ai_window_after_already_hidden"),
        "AI command block must not call the legacy open_ai_window_after_already_hidden"
    );
    assert!(
        !ai_block.contains("open_mini_ai_window_from"),
        "AI command block must not call the legacy open_mini_ai_window_from"
    );
    assert!(
        !ai_block.contains("clear_all_chats"),
        "AI command block must not call legacy clear_all_chats"
    );
}

#[test]
fn legacy_ai_window_builtins_removed_from_registration() {
    // OpenAi, MiniAi, NewConversation must not appear as registered builtins.
    // Scope the search to the get_builtin_entries function body so we don't
    // false-positive on the string appearing inside test assertions.
    let fn_start = BUILTINS_SOURCE
        .find("pub fn get_builtin_entries(")
        .expect("get_builtin_entries must exist");
    let fn_body = &BUILTINS_SOURCE[fn_start..];
    let fn_end = fn_body.find("\n#[cfg(test)]").unwrap_or(fn_body.len());
    let registration_section = &fn_body[..fn_end];

    for legacy_id in [
        "builtin/open-ai-chat",
        "builtin/mini-ai-chat",
        "builtin/new-conversation",
        "builtin/clear-conversation",
    ] {
        assert!(
            !registration_section.contains(legacy_id),
            "{legacy_id} should be removed from get_builtin_entries registration"
        );
    }
}

#[test]
fn generate_script_and_send_builtins_still_registered() {
    // These builtins remain registered (but route to harness now).
    assert!(
        BUILTINS_SOURCE.contains("\"builtin/generate-script-with-ai\""),
        "builtin/generate-script-with-ai should still be registered"
    );
    assert!(
        BUILTINS_SOURCE.contains("\"builtin/generate-script-from-current-app\""),
        "builtin/generate-script-from-current-app should still be registered"
    );
    assert!(
        BUILTINS_SOURCE.contains("\"builtin/send-screen-to-ai\""),
        "builtin/send-screen-to-ai should still be registered"
    );
    assert!(
        BUILTINS_SOURCE.contains("\"builtin/send-selected-text-to-ai\""),
        "builtin/send-selected-text-to-ai should still be registered"
    );
}

#[test]
fn dispatch_ai_script_generation_routes_to_harness() {
    // The legacy dispatch function must now route to the harness, not show_script_generation_chat.
    assert!(
        PROMPT_AI_SOURCE.contains("open_tab_ai_chat_with_entry_intent(Some(query), cx)"),
        "dispatch_ai_script_generation_from_query must route to harness entry intent"
    );
    // Must NOT call the legacy ChatPrompt path
    let dispatch_fn_start = PROMPT_AI_SOURCE
        .find("fn dispatch_ai_script_generation_from_query(")
        .expect("dispatch function must exist");
    let dispatch_fn_body = &PROMPT_AI_SOURCE[dispatch_fn_start..];
    let next_fn = dispatch_fn_body[1..]
        .find("\n    pub")
        .or_else(|| dispatch_fn_body[1..].find("\n    fn "))
        .unwrap_or(dispatch_fn_body.len());
    let dispatch_fn_body = &dispatch_fn_body[..next_fn];

    assert!(
        !dispatch_fn_body.contains("show_script_generation_chat"),
        "dispatch must not call the legacy show_script_generation_chat"
    );
}

#[test]
fn show_script_generation_chat_is_only_a_harness_shim() {
    // show_script_generation_chat must delegate to the harness entry points
    // and must NOT construct a ChatPrompt or reference ProviderRegistry.
    let fn_start = PROMPT_AI_SOURCE
        .find("pub fn show_script_generation_chat(")
        .expect("show_script_generation_chat must exist");
    let fn_body = &PROMPT_AI_SOURCE[fn_start..];
    let next_fn = fn_body[1..]
        .find("\n    pub")
        .or_else(|| fn_body[1..].find("\n    fn "))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("open_tab_ai_chat"),
        "show_script_generation_chat must delegate to open_tab_ai_chat"
    );
    assert!(
        !fn_body.contains("ChatPrompt::new"),
        "show_script_generation_chat must not construct a ChatPrompt"
    );
    assert!(
        !fn_body.contains("ProviderRegistry"),
        "show_script_generation_chat must not reference ProviderRegistry"
    );
    assert!(
        !fn_body.contains("AppView::ChatPrompt"),
        "show_script_generation_chat must not set the view to ChatPrompt"
    );
}

#[test]
fn all_ai_commands_keep_main_window_visible_for_harness() {
    // Since all active AI commands route to the harness terminal (a view
    // inside the main window), they must all keep the window visible.
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains(
            "builtins::AiCommandType::GenerateScript\n        | builtins::AiCommandType::GenerateScriptFromCurrentApp"
        ),
        "visibility function must list GenerateScript and GenerateScriptFromCurrentApp as keeping window visible"
    );
    assert!(
        BUILTIN_EXECUTION_SOURCE
            .contains("builtins::AiCommandType::OpenAi\n        | builtins::AiCommandType::MiniAi"),
        "visibility function must list OpenAi and MiniAi as keeping window visible"
    );
}

#[test]
fn manual_creation_paths_unaffected_by_ai_redirect() {
    // NewScript and NewExtension must remain registered as non-AI creation paths.
    assert!(
        BUILTINS_SOURCE.contains("ScriptCommandType::NewScript"),
        "NewScript must still be registered"
    );
    assert!(
        BUILTINS_SOURCE.contains("ScriptCommandType::NewExtension"),
        "NewExtension must still be registered"
    );
}

// =========================================================================
// Deferred capture pipeline: "open first, inject later"
// =========================================================================

#[test]
fn harness_terminal_open_path_defers_capture_until_after_view_switch() {
    // open_tab_ai_harness_terminal_from_request must set AppView::QuickTerminalView
    // and call cx.notify() BEFORE any deferred-capture await.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\nfn ")
        .or_else(|| open_fn_body[1..].find("\n    fn "))
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    let view_switch_pos = open_fn_body
        .find("AppView::QuickTerminalView")
        .expect("must switch to QuickTerminalView");
    let notify_pos = open_fn_body
        .find("cx.notify()")
        .expect("must call cx.notify()");

    // The capture_rx.recv().await must come AFTER the view switch + notify
    let capture_await_pos = open_fn_body
        .find("capture_rx.recv().await")
        .expect("must await capture_rx");

    assert!(
        view_switch_pos < capture_await_pos,
        "view switch must happen before deferred capture await"
    );
    assert!(
        notify_pos < capture_await_pos,
        "cx.notify() must happen before deferred capture await"
    );
}

#[test]
fn harness_terminal_open_path_does_not_call_capture_context_snapshot_inline() {
    // open_tab_ai_harness_terminal_from_request must NOT call
    // capture_context_snapshot directly — that now happens in the
    // deferred capture task (spawn_tab_ai_pre_switch_capture).
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    // Find the end of this function (next fn at the same indentation)
    let next_fn = open_fn_body[1..]
        .find("\nfn ")
        .or_else(|| open_fn_body[1..].find("\n    fn "))
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    // The function body should NOT contain a direct call to capture_context_snapshot
    // because it now receives capture results via capture_rx channel.
    // However, capture_context_snapshot DOES appear inside the spawned async closure
    // that receives results. We check that it doesn't appear before cx.notify().
    let notify_pos = open_fn_body
        .find("cx.notify()")
        .expect("must call cx.notify()");
    let before_notify = &open_fn_body[..notify_pos];

    assert!(
        !before_notify.contains("capture_context_snapshot("),
        "open_tab_ai_harness_terminal_from_request must not call capture_context_snapshot \
         inline before cx.notify() — capture is deferred to spawn_tab_ai_pre_switch_capture"
    );
}

#[test]
fn begin_tab_ai_harness_entry_increments_capture_generation() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("tab_ai_harness_capture_generation += 1"),
        "begin_tab_ai_harness_entry must increment the capture generation counter"
    );
}

#[test]
fn app_state_has_tab_ai_harness_capture_generation() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_capture_generation: u64"),
        "app_state.rs must have a tab_ai_harness_capture_generation field"
    );
}

#[test]
fn deferred_capture_uses_bounded_channel() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("async_channel::bounded"),
        "spawn_tab_ai_pre_switch_capture must use a bounded channel"
    );
}

#[test]
fn deferred_capture_checks_generation_before_injection() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("tab_ai_harness_capture_generation != capture_gen"),
        "deferred capture must check generation counter to drop stale results"
    );
}

// =========================================================================
// BuiltInFeature::AiChat must route to harness, not legacy AI window
// =========================================================================

#[test]
fn builtin_ai_chat_does_not_open_legacy_ai_window() {
    let arm_start = BUILTIN_EXECUTION_SOURCE
        .find("builtins::BuiltInFeature::AiChat =>")
        .expect("AiChat arm must exist in builtin_execution.rs");
    let arm =
        &BUILTIN_EXECUTION_SOURCE[arm_start..BUILTIN_EXECUTION_SOURCE.len().min(arm_start + 500)];
    assert!(
        !arm.contains("open_ai_window_after_main_hide("),
        "AiChat entry must no longer open the legacy AI window"
    );
    assert!(
        arm.contains("open_tab_ai_chat(cx)"),
        "AiChat entry must route to the harness terminal"
    );
}

#[test]
fn builtin_ai_chat_entry_reflects_harness_label() {
    assert!(
        BUILTINS_SOURCE.contains("\"Open AI Harness\""),
        "builtin/ai-chat entry must use the harness label, not the legacy AI Chat label"
    );
}

// =========================================================================
// Legacy command intent preservation
// =========================================================================

/// Helper: find a match arm inside `execute_builtin_inner` (not in helper fns).
fn find_execution_arm(needle: &str) -> &str {
    // Scope to `execute_builtin_inner` body so we skip helper-function references.
    let fn_start = BUILTIN_EXECUTION_SOURCE
        .find("fn execute_builtin_inner(")
        .expect("execute_builtin_inner must exist");
    let inner = &BUILTIN_EXECUTION_SOURCE[fn_start..];
    let arm_offset = inner
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} arm must exist inside execute_builtin_inner"));
    let arm_start = fn_start + arm_offset;
    &BUILTIN_EXECUTION_SOURCE[arm_start..BUILTIN_EXECUTION_SOURCE.len().min(arm_start + 1200)]
}

#[test]
fn generate_script_from_current_app_preserves_explicit_harness_intent() {
    let arm = find_execution_arm("AiCommandType::GenerateScriptFromCurrentApp");

    assert!(
        arm.contains("query_override") && arm.contains("open_tab_ai_chat_with_entry_intent(Some("),
        "GenerateScriptFromCurrentApp must preserve explicit harness intent \
         instead of flattening to open_tab_ai_chat(cx)"
    );
    assert!(
        arm.contains("Generate a Script Kit script"),
        "GenerateScriptFromCurrentApp must include a script-generation intent string"
    );
}

#[test]
fn legacy_screenshot_commands_preserve_requested_capture_kind() {
    // SendScreenToAi must carry a full-screen intent and capture kind
    let screen_arm = find_execution_arm("AiCommandType::SendScreenToAi");
    assert!(
        screen_arm.contains("open_tab_ai_chat_with_capture_kind(")
            && screen_arm.contains("full screen"),
        "SendScreenToAi must request full-screen capture via explicit intent"
    );
    assert!(
        screen_arm.contains("TabAiCaptureKind::FullScreen"),
        "SendScreenToAi must request FullScreen capture kind"
    );

    // SendFocusedWindowToAi must carry a focused-window intent and capture kind
    let focused_arm = find_execution_arm("AiCommandType::SendFocusedWindowToAi");
    assert!(
        focused_arm.contains("open_tab_ai_chat_with_capture_kind(")
            && focused_arm.contains("focused window"),
        "SendFocusedWindowToAi must request focused-window capture via explicit intent"
    );
    assert!(
        focused_arm.contains("TabAiCaptureKind::FocusedWindow"),
        "SendFocusedWindowToAi must request FocusedWindow capture kind"
    );

    // SendScreenAreaToAi must fail with an explicit error — no real region capture yet.
    let area_arm_full = find_execution_arm("AiCommandType::SendScreenAreaToAi");
    // Trim to just this arm (stop before the next AiCommandType match).
    let area_arm = area_arm_full
        .find("AiCommandType::SendSelectedTextToAi")
        .map(|pos| &area_arm_full[..pos])
        .unwrap_or(area_arm_full);
    assert!(
        area_arm.contains("builtin_error(") && area_arm.contains("unavailable"),
        "SendScreenAreaToAi must return an explicit error until region capture is attached"
    );
    assert!(
        area_arm.contains("toast_manager.push("),
        "SendScreenAreaToAi must show an error toast to the user"
    );
    assert!(
        !area_arm.contains("open_tab_ai_chat"),
        "SendScreenAreaToAi must NOT open the harness without real region data"
    );
}

#[test]
fn send_selected_text_to_ai_submits_explicit_intent() {
    let arm = find_execution_arm("AiCommandType::SendSelectedTextToAi");
    assert!(
        arm.contains("open_tab_ai_chat_with_capture_kind("),
        "SendSelectedTextToAi must open the harness with an explicit intent"
    );
    assert!(
        arm.contains("selected text"),
        "SendSelectedTextToAi must include 'selected text' in the intent string"
    );
    assert!(
        arm.contains("TabAiCaptureKind::SelectedText"),
        "SendSelectedTextToAi must request SelectedText capture kind"
    );
}

#[test]
fn send_browser_tab_to_ai_submits_explicit_intent() {
    let arm = find_execution_arm("AiCommandType::SendBrowserTabToAi");
    assert!(
        arm.contains("open_tab_ai_chat_with_capture_kind("),
        "SendBrowserTabToAi must open the harness with an explicit intent"
    );
    assert!(
        arm.contains("browser tab"),
        "SendBrowserTabToAi must include 'browser tab' in the intent string"
    );
    assert!(
        arm.contains("TabAiCaptureKind::BrowserTab"),
        "SendBrowserTabToAi must request BrowserTab capture kind"
    );
}

// =========================================================================
// Capture kind plumbing regression coverage
// =========================================================================

#[test]
fn deferred_capture_stops_ignoring_launch_request() {
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn spawn_tab_ai_pre_switch_capture(")
        .expect("spawn_tab_ai_pre_switch_capture must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        !fn_body.contains("_request: &TabAiLaunchRequest"),
        "deferred capture must stop ignoring the launch request"
    );
    assert!(
        fn_body.contains("request.capture_kind"),
        "deferred capture must branch on request.capture_kind"
    );
}

#[test]
fn explicit_ai_capture_commands_use_capture_kind_plumbing() {
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("TabAiCaptureKind::FullScreen"),
        "SendScreenToAi must request FullScreen capture"
    );
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("TabAiCaptureKind::FocusedWindow"),
        "SendFocusedWindowToAi must request FocusedWindow capture"
    );
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("TabAiCaptureKind::SelectedText"),
        "SendSelectedTextToAi must request SelectedText capture"
    );
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("TabAiCaptureKind::BrowserTab"),
        "SendBrowserTabToAi must request BrowserTab capture"
    );
}

#[test]
fn capture_kind_enum_is_exported_from_ai_module() {
    assert!(
        AI_MOD_SOURCE.contains("TabAiCaptureKind"),
        "TabAiCaptureKind must be re-exported from the ai module"
    );
}

#[test]
fn launch_request_carries_capture_kind_field() {
    let struct_start = TAB_AI_MODE_SOURCE
        .find("struct TabAiLaunchRequest")
        .expect("TabAiLaunchRequest must exist");
    let struct_body = &TAB_AI_MODE_SOURCE[struct_start..];
    let struct_end = struct_body.find("\n}").unwrap_or(struct_body.len());
    let struct_body = &struct_body[..struct_end];

    assert!(
        struct_body.contains("capture_kind"),
        "TabAiLaunchRequest must carry a capture_kind field"
    );
}

// =========================================================================
// Apply-back regression coverage
// =========================================================================

#[test]
fn tab_ai_harness_tracks_apply_back_route_state() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_apply_back_route"),
        "ScriptListApp must persist apply-back routing state for the active harness session"
    );
}

#[test]
fn quick_terminal_cmd_enter_routes_to_apply_back() {
    assert!(
        TERM_RENDER_SOURCE.contains("this.apply_tab_ai_result_from_terminal(entity.clone(), cx);"),
        "QuickTerminalView must route Cmd+Enter into apply-back via the terminal helper"
    );
}

#[test]
fn tab_ai_apply_back_uses_running_command_prompt_reinjection() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.try_set_prompt_input(text.clone(), cx)"),
        "RunningCommand apply-back must reuse try_set_prompt_input instead of frontmost-app paste"
    );
}

#[test]
fn tab_ai_frontmost_apply_back_hides_before_paste() {
    let hide_pos = TAB_AI_MODE_SOURCE
        .find("crate::platform::defer_hide_main_window(cx)")
        .expect("apply-back must defer-hide the main window");

    let replace_pos = TAB_AI_MODE_SOURCE
        .find("selected_text::set_selected_text(&text_for_apply)")
        .expect("apply-back must support selected-text replacement");

    let paste_pos = TAB_AI_MODE_SOURCE
        .find(".paste_text(&text_for_apply)")
        .expect("apply-back must support frontmost-app paste");

    assert!(
        hide_pos < replace_pos,
        "main window must hide before set_selected_text fires"
    );
    assert!(
        hide_pos < paste_pos,
        "main window must hide before TextInjector::paste_text fires"
    );
}

#[test]
fn tab_ai_apply_back_route_cleared_on_harness_close() {
    // close_tab_ai_harness_terminal must clear the apply-back route to prevent
    // stale routing after the harness session ends.
    let close_fn_start = TAB_AI_MODE_SOURCE
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_fn_body = &TAB_AI_MODE_SOURCE[close_fn_start..];
    let next_fn = close_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(close_fn_body.len());
    let close_fn_body = &close_fn_body[..next_fn];

    assert!(
        close_fn_body.contains("tab_ai_harness_apply_back_route"),
        "close_tab_ai_harness_terminal must clear the apply-back route"
    );
}

#[test]
fn tab_ai_apply_back_hint_strip_visible_in_quick_terminal() {
    // The hint strip must advertise the ⌘⏎ Apply action so users know about it.
    assert!(
        TERM_RENDER_SOURCE.contains("Apply") && TERM_RENDER_SOURCE.contains("Close"),
        "QuickTerminalView hint strip must show both Apply and Close actions"
    );
}

// =========================================================================
// ScriptListItem apply-back contract
// =========================================================================

#[test]
fn script_list_item_apply_back_no_longer_shows_stub_toast() {
    // The ScriptListItem branch must NOT contain the old "not wired yet" stub.
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];
    let next_impl = apply_fn_body[1..]
        .find("\nimpl ")
        .unwrap_or(apply_fn_body.len());
    let apply_fn_body = &apply_fn_body[..next_impl];

    assert!(
        !apply_fn_body.contains("not wired yet"),
        "ScriptListItem apply-back must no longer show the placeholder toast"
    );
}

#[test]
fn script_list_item_apply_back_saves_generated_script() {
    // The ScriptListItem branch must save a generated script via the existing
    // save_generated_script_from_response pipeline.
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    // Find the ScriptListItem match arm.
    let arm_start = apply_fn_body
        .find("ScriptListItem =>")
        .expect("ScriptListItem match arm must exist in apply-back");
    let arm_body = &apply_fn_body[arm_start..];

    assert!(
        arm_body.contains("save_generated_script_from_response"),
        "ScriptListItem apply-back must route through save_generated_script_from_response"
    );
}

#[test]
fn script_list_item_apply_back_executes_saved_script() {
    // After saving the script, the ScriptListItem branch must execute it.
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    let arm_start = apply_fn_body
        .find("ScriptListItem =>")
        .expect("ScriptListItem match arm must exist");
    let arm_body = &apply_fn_body[arm_start..];

    assert!(
        arm_body.contains("execute_script_by_path"),
        "ScriptListItem apply-back must execute the saved script"
    );
}

#[test]
fn script_list_item_apply_back_closes_harness_first() {
    // The harness must be closed before saving/executing the script.
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    let arm_start = apply_fn_body
        .find("ScriptListItem =>")
        .expect("ScriptListItem match arm must exist");
    let arm_body = &apply_fn_body[arm_start..];

    let close_pos = arm_body
        .find("close_tab_ai_harness_terminal")
        .expect("ScriptListItem apply-back must close the harness");
    let save_pos = arm_body
        .find("save_generated_script_from_response")
        .expect("ScriptListItem apply-back must save a script");

    assert!(
        close_pos < save_pos,
        "Harness must close before saving the generated script"
    );
}

#[test]
fn script_list_item_apply_back_uses_focused_target_label() {
    // The focused target label should be used as the prompt for slug derivation,
    // not a hardcoded string.
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    let arm_start = apply_fn_body
        .find("ScriptListItem =>")
        .expect("ScriptListItem match arm must exist");
    let arm_body = &apply_fn_body[arm_start..];

    assert!(
        arm_body.contains("focused_target") && arm_body.contains("label"),
        "ScriptListItem apply-back must use the focused target's label for the prompt"
    );
}

// =========================================================================
// ClipboardEntry apply-back contract
// =========================================================================

#[test]
fn clipboard_entry_apply_back_closes_harness_first() {
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    let arm_start = apply_fn_body
        .find("ClipboardEntry =>")
        .expect("ClipboardEntry match arm must exist in apply-back");
    let arm_body = &apply_fn_body[arm_start..];

    let close_pos = arm_body
        .find("close_tab_ai_harness_terminal")
        .expect("ClipboardEntry apply-back must close the harness");
    let clipboard_pos = arm_body
        .find("write_tab_ai_apply_back_clipboard_text")
        .expect("ClipboardEntry apply-back must write to clipboard");

    assert!(
        close_pos < clipboard_pos,
        "Harness must close before writing clipboard result"
    );
}

#[test]
fn clipboard_entry_apply_back_writes_to_clipboard() {
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    let arm_start = apply_fn_body
        .find("ClipboardEntry =>")
        .expect("ClipboardEntry match arm must exist");
    let arm_body = &apply_fn_body[arm_start..];

    assert!(
        arm_body.contains("write_tab_ai_apply_back_clipboard_text"),
        "ClipboardEntry apply-back must use write_tab_ai_apply_back_clipboard_text"
    );
}

// =========================================================================
// Desktop / DesktopSelection apply-back contract
// =========================================================================

#[test]
fn desktop_selection_apply_back_uses_set_selected_text() {
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    // The DesktopSelection arm should call set_selected_text
    assert!(
        apply_fn_body.contains("selected_text::set_selected_text(&text_for_apply)"),
        "DesktopSelection apply-back must replace via set_selected_text"
    );
}

#[test]
fn generic_desktop_apply_back_uses_paste_text() {
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    // The Desktop arm should use TextInjector::paste_text
    assert!(
        apply_fn_body.contains(".paste_text(&text_for_apply)"),
        "Desktop (generic) apply-back must paste via TextInjector"
    );
}

#[test]
fn apply_back_match_covers_all_five_source_types() {
    let apply_fn_start = TAB_AI_MODE_SOURCE
        .find("fn apply_tab_ai_result_from_clipboard(")
        .expect("apply_tab_ai_result_from_clipboard must exist");
    let apply_fn_body = &TAB_AI_MODE_SOURCE[apply_fn_start..];

    for arm in &[
        "RunningCommand =>",
        "ClipboardEntry =>",
        "ScriptListItem =>",
        "DesktopSelection",
        "Desktop =>",
    ] {
        assert!(
            apply_fn_body.contains(arm),
            "apply_tab_ai_result_from_clipboard must handle {arm}"
        );
    }
}

// =========================================================================
// Script List context shaping: explicit branch in target resolution
// =========================================================================

#[test]
fn resolve_targets_has_explicit_script_list_branch() {
    // resolve_tab_ai_surface_targets_for_view must have an explicit
    // AppView::ScriptList arm that uses script-native metadata, not
    // the generic element-based fallback.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("AppView::ScriptList =>"),
        "resolve_tab_ai_surface_targets_for_view must have an explicit ScriptList arm"
    );
}

#[test]
fn script_list_branch_uses_grouped_results_cache() {
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    // The ScriptList arm must read from the same grouped results cache
    // that the main list renderer uses, not from UI snapshot elements.
    assert!(
        fn_body.contains("cached_grouped_items"),
        "ScriptList branch must resolve from cached_grouped_items"
    );
    assert!(
        fn_body.contains("cached_grouped_flat_results"),
        "ScriptList branch must resolve from cached_grouped_flat_results"
    );
}

#[test]
fn script_list_branch_delegates_to_search_result_helper() {
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("tab_ai_target_from_search_result"),
        "ScriptList branch must use tab_ai_target_from_search_result for rich metadata"
    );
}

#[test]
fn tab_ai_target_from_search_result_helper_exists() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn tab_ai_target_from_search_result("),
        "tab_ai_target_from_search_result helper must exist in tab_ai_mode.rs"
    );
}

#[test]
fn search_result_helper_produces_script_list_source() {
    // The helper must tag all targets with source "ScriptList" so that
    // detect_tab_ai_source_type can classify them correctly.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_target_from_search_result(")
        .expect("tab_ai_target_from_search_result must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    assert!(
        fn_body.contains("\"ScriptList\""),
        "tab_ai_target_from_search_result must set source to \"ScriptList\""
    );
}

#[test]
fn search_result_helper_maps_all_result_kinds() {
    // Every SearchResult variant must have a corresponding kind string.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_target_from_search_result(")
        .expect("tab_ai_target_from_search_result must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    for kind in &[
        "\"script\"",
        "\"scriptlet\"",
        "\"builtin\"",
        "\"app\"",
        "\"window\"",
        "\"agent\"",
        "\"fallback\"",
    ] {
        assert!(
            fn_body.contains(kind),
            "tab_ai_target_from_search_result must map kind {kind}"
        );
    }
}

// =========================================================================
// Harness close lifecycle: full contract assertions
// =========================================================================

#[test]
fn close_tab_ai_harness_terminal_clears_session_and_schedules_fresh_prewarm() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.tab_ai_harness_capture_generation += 1;"),
        "close must invalidate in-flight deferred capture results",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("self.tab_ai_harness_apply_back_route = None;"),
        "close must clear apply-back routing state",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("let session = self.tab_ai_harness.take();"),
        "close must clear app state by taking tab_ai_harness",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("term.terminate_session().map_err(|e| e.to_string())"),
        "close must terminate the PTY session",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains(
            "self.schedule_tab_ai_harness_prewarm(std::time::Duration::from_millis(250), cx);"
        ),
        "close must schedule a fresh harness prewarm after teardown",
    );
}

#[test]
fn quick_terminal_cmd_w_routes_to_harness_close() {
    assert!(
        TERM_RENDER_SOURCE.contains("key.eq_ignore_ascii_case(\"w\")"),
        "QuickTerminalView must reserve Cmd+W as the wrapper close gesture",
    );
    assert!(
        TERM_RENDER_SOURCE.contains("this.close_tab_ai_harness_terminal(cx);"),
        "Cmd+W must call close_tab_ai_harness_terminal",
    );
}

#[test]
fn search_result_helper_includes_script_metadata() {
    // Script-type results must include path and description in metadata.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_target_from_search_result(")
        .expect("tab_ai_target_from_search_result must exist");
    let fn_body = &TAB_AI_MODE_SOURCE[fn_start..];
    let next_fn = fn_body[1..].find("\n    fn ").unwrap_or(fn_body.len());
    let fn_body = &fn_body[..next_fn];

    // Script metadata must include path and description for AI context.
    assert!(
        fn_body.contains("\"path\"") && fn_body.contains("\"description\""),
        "Script metadata must include path and description"
    );
    // Script metadata must include shortcut and alias for context.
    assert!(
        fn_body.contains("\"shortcut\"") && fn_body.contains("\"alias\""),
        "Script metadata must include shortcut and alias"
    );
}

// =========================================================================
// SendScreenAreaToAi: removed from registration, unavailable at execution
// =========================================================================

#[test]
fn send_screen_area_not_registered_in_builtin_entries() {
    let fn_start = BUILTINS_SOURCE
        .find("pub fn get_builtin_entries(")
        .expect("get_builtin_entries must exist");
    let fn_body = &BUILTINS_SOURCE[fn_start..];
    let fn_end = fn_body.find("\n#[cfg(test)]").unwrap_or(fn_body.len());
    let registration_section = &fn_body[..fn_end];

    assert!(
        !registration_section.contains("\"builtin/send-screen-area-to-ai\""),
        "builtin/send-screen-area-to-ai must not be registered until harness attachment exists",
    );
}

#[test]
fn send_screen_area_execution_returns_stable_unavailable_result() {
    // The execution arm must still exist for stale invocations...
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("AiCommandType::SendScreenAreaToAi => {"),
        "stale invocations of SendScreenAreaToAi still need a deterministic execution handler",
    );
    // ...and must fail with a stable unavailable result code...
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("ai_send_screen_area_unavailable"),
        "stale invocations must fail with a stable unavailable result code",
    );
    // ...and must explain the exact missing capability.
    assert!(
        BUILTIN_EXECUTION_SOURCE.contains("selected-area capture is attached to the harness"),
        "the unavailable path must explain the exact missing harness capability",
    );
}

// =========================================================================
// Script-generation compat shims: harness-only, no legacy ChatPrompt
// =========================================================================

#[test]
fn script_generation_compat_shims_documented_as_harness_only() {
    assert!(
        PROMPT_AI_SOURCE.contains("Compatibility shim \u{2014} routes script generation requests"),
        "prompt_ai.rs must document the harness-only compatibility shim",
    );
}

#[test]
fn script_generation_compat_shims_do_not_reconstruct_legacy_surface() {
    // The old script-generation ChatPrompt surface used "script-generation"
    // as its ID. The compat shims must not recreate that surface.
    assert!(
        !PROMPT_AI_SOURCE.contains("\"script-generation\""),
        "prompt_ai.rs must not recreate the old script-generation ChatPrompt surface",
    );
}

// =========================================================================
// "Send to AI" fallback: priority 0, harness-native, inline execution
// =========================================================================

const FALLBACK_BUILTINS_SOURCE: &str = include_str!("../src/fallbacks/builtins.rs");
const SELECTION_FALLBACK_SOURCE: &str = include_str!("../src/app_impl/selection_fallback.rs");

#[test]
fn send_to_ai_fallback_label_is_constant() {
    assert!(
        FALLBACK_BUILTINS_SOURCE
            .contains("pub const SEND_TO_AI_FALLBACK_LABEL: &str = \"Auto Submit\""),
        "the top fallback label must be the constant 'Auto Submit'",
    );
}

#[test]
fn send_to_ai_fallback_is_harness_native() {
    assert!(
        FALLBACK_BUILTINS_SOURCE.contains("action: FallbackAction::SendToAiHarness"),
        "the top fallback must use FallbackAction::SendToAiHarness",
    );
}

#[test]
fn send_to_ai_fallback_is_priority_zero() {
    assert!(
        FALLBACK_BUILTINS_SOURCE.contains("priority: 0"),
        "the harness fallback must be priority 0 (first in no-results list)",
    );
}

#[test]
fn selection_fallback_recognizes_harness_result() {
    assert!(
        SELECTION_FALLBACK_SOURCE.contains("FallbackResult::SendToAiHarness { query }"),
        "selection fallback execution must recognize the SendToAiHarness result variant",
    );
}

#[test]
fn selection_fallback_routes_through_quick_submit_planner() {
    assert!(
        SELECTION_FALLBACK_SOURCE
            .contains("self.submit_to_current_or_new_tab_ai_harness_from_text("),
        "non-empty Send to AI fallback must route through the quick-submit planner",
    );
    assert!(
        SELECTION_FALLBACK_SOURCE.contains("TabAiQuickSubmitSource::Fallback"),
        "Send to AI fallback must identify its source as Fallback",
    );
}

// =========================================================================
// Full-screen capture helper: contract and metadata
// =========================================================================

#[test]
fn full_screen_capture_helper_exists_and_uses_screen_api() {
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("pub fn capture_tab_ai_screen_screenshot_file()"),
        "full-screen screenshot helper must exist as a public function",
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("capture_screen_screenshot()"),
        "full-screen helper must call the platform full-screen screenshot API",
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("title: \"Full Screen\".to_string()"),
        "full-screen helper must label the artifact as 'Full Screen'",
    );
}

#[test]
fn full_screen_capture_helper_preserves_screenshot_file_contract() {
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("cleanup_old_tab_ai_screenshot_files"),
        "full-screen helper must clean up old screenshot temp files",
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("TAB_AI_SCREENSHOT_MAX_KEEP"),
        "full-screen helper must use the shared screenshot retention limit",
    );
    assert!(
        SCREENSHOT_FILES_SOURCE.contains("used_fallback: false"),
        "full-screen helper must set used_fallback to false in screenshot metadata",
    );
}

// =========================================================================
// Builtin registry: legacy AI window entries removed, supported paths kept
// =========================================================================

#[test]
fn legacy_ai_window_entries_stay_removed_while_manual_paths_stay_present() {
    let fn_start = BUILTINS_SOURCE
        .find("pub fn get_builtin_entries(")
        .expect("get_builtin_entries must exist");
    let fn_body = &BUILTINS_SOURCE[fn_start..];
    let fn_end = fn_body.find("\n#[cfg(test)]").unwrap_or(fn_body.len());
    let registration_section = &fn_body[..fn_end];

    // Legacy window-style AI builtins must NOT be registered.
    // Match quoted string literals to avoid false positives from comments.
    for legacy_id in [
        "builtin/open-ai-chat",
        "builtin/mini-ai-chat",
        "builtin/new-conversation",
        "builtin/clear-conversation",
        "builtin/send-screen-area-to-ai",
    ] {
        let quoted = format!("\"{}\"", legacy_id);
        assert!(
            !registration_section.contains(&quoted),
            "{legacy_id} must not be registered in the main builtin list",
        );
    }

    // Harness-first AI entries and manual creation paths must remain registered.
    for kept_id in [
        "builtin/generate-script-with-ai",
        "builtin/generate-script-from-current-app",
        "builtin/send-screen-to-ai",
        "builtin/send-selected-text-to-ai",
        "builtin/send-browser-tab-to-ai",
        "builtin/new-script",
        "builtin/new-extension",
    ] {
        let quoted = format!("\"{}\"", kept_id);
        assert!(
            registration_section.contains(&quoted),
            "{kept_id} should remain registered",
        );
    }
}

#[test]
fn post_close_prewarm_feeds_the_next_explicit_tab_open() {
    let source = include_str!("../src/app_impl/tab_ai_mode.rs");

    // Close path must schedule a silent prewarm.
    let close_start = source
        .find("fn close_tab_ai_harness_terminal(")
        .expect("close_tab_ai_harness_terminal must exist");
    let close_body = &source[close_start..];
    assert!(
        close_body.contains("schedule_tab_ai_harness_prewarm"),
        "close path must schedule a silent prewarm"
    );

    // Next explicit Tab must check for a fresh prewarm and consume it.
    let open_start = source
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_body = &source[open_start..];
    let next_fn = open_body[1..].find("\n    fn ").unwrap_or(open_body.len());
    let open_body = &open_body[..next_fn];

    assert!(
        open_body.contains("is_fresh_prewarm"),
        "next explicit Tab must check for a fresh prewarm"
    );
    assert!(
        open_body.contains("mark_consumed"),
        "next explicit Tab must consume the prewarm once"
    );
}

// ---------------------------------------------------------------------------
// Live-session quick submit: structured submission regression tests
// ---------------------------------------------------------------------------

#[test]
fn live_quick_submit_uses_structured_submission_helper() {
    let src = std::fs::read_to_string("src/app_impl/tab_ai_mode.rs").expect("read tab_ai_mode.rs");

    let fn_start = src
        .find("pub(crate) fn submit_to_current_or_new_tab_ai_harness_from_text(")
        .expect("submit_to_current_or_new_tab_ai_harness_from_text must exist");
    let fn_src = &src[fn_start..fn_start + 2500.min(src.len() - fn_start)];

    assert!(
        fn_src.contains("self.submit_live_tab_ai_harness_from_plan("),
        "live harness quick submit must route through the structured submission helper"
    );
    assert!(
        !fn_src.contains("format!(\"{}\\n\", plan.synthesized_intent)"),
        "live harness quick submit must not inject raw intent-only text"
    );
}

#[test]
fn live_quick_submit_helper_builds_full_harness_submission() {
    let src = std::fs::read_to_string("src/app_impl/tab_ai_mode.rs").expect("read tab_ai_mode.rs");

    let helper_start = src
        .find("fn submit_live_tab_ai_harness_from_plan(")
        .expect("submit_live_tab_ai_harness_from_plan must exist");
    let helper_src = &src[helper_start..helper_start + 6000.min(src.len() - helper_start)];

    assert!(
        helper_src.contains("build_tab_ai_harness_submission"),
        "live quick submit must build a full structured harness submission"
    );
    assert!(
        helper_src.contains("TabAiHarnessSubmissionMode::Submit"),
        "live quick submit must submit immediately"
    );
    assert!(
        helper_src.contains("request.quick_submit_plan.as_ref()"),
        "live quick submit must pass quick submit plan to submission builder"
    );
}

// =========================================================================
// ~ mini entry: file search presentation and query normalization
// =========================================================================

const FILTER_INPUT_CHANGE_SOURCE: &str = include_str!("../src/app_impl/filter_input_change.rs");
const FILTER_INPUT_CORE_SOURCE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const FILE_SEARCH_SOURCE: &str = include_str!("../src/render_builtins/file_search.rs");
const UTILITY_VIEWS_SOURCE: &str = include_str!("../src/app_execute/utility_views.rs");

#[test]
fn tilde_trigger_enters_file_search_as_mini() {
    // When the user types `~` in ScriptList, filter_input_change must hand off
    // to mini file search with FileSearchPresentation::Mini.
    assert!(
        FILTER_INPUT_CHANGE_SOURCE.contains("should_enter_file_search_from_script_list"),
        "filter_input_change must call should_enter_file_search_from_script_list for ~ trigger"
    );
    assert!(
        FILTER_INPUT_CHANGE_SOURCE.contains("FileSearchPresentation::Mini"),
        "filter_input_change must open file search as Mini presentation"
    );
}

#[test]
fn tilde_trigger_normalizes_bare_tilde_to_tilde_slash() {
    // Bare `~` must normalize to `~/` so directory listing starts immediately.
    assert!(
        FILTER_INPUT_CHANGE_SOURCE.contains("normalize_mini_file_search_query"),
        "filter_input_change must normalize the query before opening file search"
    );
    assert!(
        FILTER_INPUT_CORE_SOURCE.contains("fn normalize_mini_file_search_query"),
        "normalize_mini_file_search_query must be defined in filter_input_core"
    );
    // The function must convert bare `~` to `~/`
    assert!(
        FILTER_INPUT_CORE_SOURCE.contains(r#"if new_text == "~""#),
        "normalize_mini_file_search_query must detect bare ~"
    );
    assert!(
        FILTER_INPUT_CORE_SOURCE.contains(r#""~/".to_string()"#),
        "normalize_mini_file_search_query must produce ~/ from bare ~"
    );
}

#[test]
fn tilde_trigger_predicate_matches_tilde_and_tilde_slash_prefix() {
    // should_enter_file_search_from_script_list must match `~` and `~/...`
    assert!(
        FILTER_INPUT_CORE_SOURCE.contains(r#"new_text == "~" || new_text.starts_with("~/")"#),
        "should_enter_file_search_from_script_list must match ~ and ~/... patterns"
    );
}

#[test]
fn mini_presentation_exits_when_trigger_no_longer_matches() {
    // If the user edits the query so it no longer starts with ~, mini mode
    // must return to ScriptList rather than staying stuck in file search.
    let exit_check = FILTER_INPUT_CHANGE_SOURCE
        .find("FileSearchPresentation::Mini")
        .expect("Mini presentation check must exist");
    let nearby = &FILTER_INPUT_CHANGE_SOURCE
        [exit_check..exit_check + 500.min(FILTER_INPUT_CHANGE_SOURCE.len() - exit_check)];

    assert!(
        nearby.contains("should_enter_file_search_from_script_list"),
        "Mini presentation must recheck the ~ trigger and exit when it no longer matches"
    );
}

// =========================================================================
// File search AI routing: selection vs query fallback
// =========================================================================

#[test]
fn file_search_cmd_enter_routes_through_selection_or_query() {
    // ⌘↵ in file search must call open_file_search_selection_or_query_in_tab_ai,
    // which tries the selected row first, then falls back to query-level intent.
    assert!(
        FILE_SEARCH_SOURCE.contains("open_file_search_selection_or_query_in_tab_ai"),
        "file_search.rs Enter handler must call the fallback-capable AI opener"
    );
}

#[test]
fn selection_or_query_tries_selection_first() {
    // open_file_search_selection_or_query_in_tab_ai must attempt the selection
    // path before falling back to query-level intent.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_file_search_selection_or_query_in_tab_ai(")
        .expect("open_file_search_selection_or_query_in_tab_ai must exist");
    let fn_body =
        &TAB_AI_MODE_SOURCE[fn_start..fn_start + 800.min(TAB_AI_MODE_SOURCE.len() - fn_start)];

    assert!(
        fn_body.contains("open_file_search_selection_in_tab_ai"),
        "must try selection-based AI first"
    );
    // The selection attempt must come before the query fallback
    let sel_pos = fn_body
        .find("open_file_search_selection_in_tab_ai")
        .unwrap();
    let query_pos = fn_body
        .find("build_file_search_ai_query_intent")
        .expect("must have query fallback");
    assert!(
        sel_pos < query_pos,
        "selection AI must be attempted before query fallback"
    );
}

#[test]
fn query_fallback_returns_none_for_empty_state() {
    // build_file_search_ai_query_intent must return None when both query is
    // empty and there are no visible results, so ⌘↵ is a no-op only in the
    // truly empty state.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn build_file_search_ai_query_intent(")
        .expect("build_file_search_ai_query_intent must exist");
    let fn_body =
        &TAB_AI_MODE_SOURCE[fn_start..fn_start + 400.min(TAB_AI_MODE_SOURCE.len() - fn_start)];

    assert!(
        fn_body.contains("query.is_empty() && self.file_search_display_indices.is_empty()"),
        "query intent must return None only when both query and results are empty"
    );
    assert!(
        fn_body.contains("return None"),
        "must explicitly return None for the empty state"
    );
}

#[test]
fn selection_ai_uses_quick_submit_with_file_search_source() {
    // open_file_search_selection_in_tab_ai must build a TabAiQuickSubmitPlan
    // with source = FileSearch so harness logging has provenance.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_file_search_selection_in_tab_ai(")
        .expect("open_file_search_selection_in_tab_ai must exist");
    let fn_body =
        &TAB_AI_MODE_SOURCE[fn_start..fn_start + 1500.min(TAB_AI_MODE_SOURCE.len() - fn_start)];

    assert!(
        fn_body.contains("TabAiQuickSubmitSource::FileSearch"),
        "selection AI must use FileSearch as the quick submit source"
    );
    assert!(
        fn_body.contains("TabAiQuickSubmitKind::FileDrop"),
        "selection AI must use FileDrop as the quick submit kind"
    );
    assert!(
        fn_body.contains("open_tab_ai_chat_with_quick_submit_plan"),
        "selection AI must route through the quick submit plan path"
    );
}

#[test]
fn query_fallback_uses_entry_intent_not_quick_submit() {
    // When no row is selected, query-level AI uses open_tab_ai_chat_with_entry_intent
    // (not quick submit), because there is no specific file to submit.
    let fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_file_search_selection_or_query_in_tab_ai(")
        .expect("open_file_search_selection_or_query_in_tab_ai must exist");
    let fn_body =
        &TAB_AI_MODE_SOURCE[fn_start..fn_start + 800.min(TAB_AI_MODE_SOURCE.len() - fn_start)];

    assert!(
        fn_body.contains("open_tab_ai_chat_with_entry_intent"),
        "query fallback must use entry intent (not quick submit)"
    );
}

#[test]
fn file_search_ai_intent_includes_presentation_label() {
    // Both build_file_search_ai_entry_intent and build_file_search_ai_query_intent
    // must embed the current presentation (mini vs full) so the AI model knows
    // which surface the user is on.
    let entry_fn_start = TAB_AI_MODE_SOURCE
        .find("fn build_file_search_ai_entry_intent(")
        .expect("build_file_search_ai_entry_intent must exist");
    let entry_fn_body = &TAB_AI_MODE_SOURCE
        [entry_fn_start..entry_fn_start + 2000.min(TAB_AI_MODE_SOURCE.len() - entry_fn_start)];
    assert!(
        entry_fn_body.contains("File-search presentation:"),
        "entry intent must include presentation label"
    );

    let query_fn_start = TAB_AI_MODE_SOURCE
        .find("fn build_file_search_ai_query_intent(")
        .expect("build_file_search_ai_query_intent must exist");
    let query_fn_body = &TAB_AI_MODE_SOURCE
        [query_fn_start..query_fn_start + 2000.min(TAB_AI_MODE_SOURCE.len() - query_fn_start)];
    assert!(
        query_fn_body.contains("File-search presentation:"),
        "query intent must include presentation label"
    );
}

#[test]
fn file_search_cmd_enter_passes_shift_for_plan_mode() {
    // The ⌘↵ handler must check shift state and pass it as plan_mode,
    // distinguishing ⌘↵ (explain) from ⌘⇧↵ (plan).
    let handler_area = FILE_SEARCH_SOURCE
        .find("open_file_search_selection_or_query_in_tab_ai")
        .expect("AI opener call must exist in file_search.rs");
    let nearby = &FILE_SEARCH_SOURCE[handler_area.saturating_sub(300)
        ..handler_area + 200.min(FILE_SEARCH_SOURCE.len() - handler_area)];

    assert!(
        nearby.contains("has_shift") || nearby.contains("modifiers.shift"),
        "⌘↵ handler must distinguish shift for plan_mode"
    );
}

// =========================================================================
// ACP routing: Tab now opens AcpChatView instead of QuickTerminalView
// =========================================================================

const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const ACP_MOD_SOURCE: &str = include_str!("../src/ai/acp/mod.rs");

#[test]
fn tab_ai_mode_opens_acp_chat_view_for_tab() {
    // Tab AI entry path must create an AcpChatView, not just QuickTerminalView.
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpChatView::new"),
        "tab_ai_mode must create an AcpChatView for Tab AI"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AppView::AcpChatView"),
        "tab_ai_mode must set current_view to AppView::AcpChatView"
    );
}

#[test]
fn tab_ai_mode_creates_acp_thread_with_permission_broker() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpThread::new"),
        "tab_ai_mode must create an AcpThread"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AcpPermissionBroker::new"),
        "tab_ai_mode must create a permission broker for tool approval"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("spawn_with_approval"),
        "AcpConnection must be spawned with approval wiring"
    );
}

#[test]
fn tab_ai_mode_stages_context_on_acp_thread() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("stage_context"),
        "tab_ai_mode must stage context on the AcpThread for first-turn injection"
    );
}

#[test]
fn startup_guards_against_double_acp_open() {
    // Both startup.rs variants must check for AcpChatView to prevent
    // stacking a second ACP session on Tab press.
    assert!(
        TAB_SOURCE.contains("AppView::AcpChatView"),
        "startup.rs must check for existing AcpChatView"
    );
    assert!(
        TAB_SOURCE.contains("handle_tab_key"),
        "startup.rs must delegate Tab to AcpChatView.handle_tab_key"
    );

    assert!(
        TAB_NEW_SOURCE.contains("AppView::AcpChatView"),
        "startup_new_tab.rs must check for existing AcpChatView"
    );
    assert!(
        TAB_NEW_SOURCE.contains("handle_tab_key"),
        "startup_new_tab.rs must delegate Tab to AcpChatView.handle_tab_key"
    );
}

#[test]
fn acp_chat_view_consumes_tab_to_prevent_reentry() {
    // AcpChatView.handle_tab_key must return true to consume the key.
    assert!(
        ACP_VIEW_SOURCE.contains("fn handle_tab_key"),
        "AcpChatView must implement handle_tab_key"
    );
    // The function must return true to consume Tab.
    let fn_start = ACP_VIEW_SOURCE
        .find("fn handle_tab_key")
        .expect("handle_tab_key must exist");
    let fn_body = &ACP_VIEW_SOURCE[fn_start..fn_start + 300.min(ACP_VIEW_SOURCE.len() - fn_start)];
    assert!(
        fn_body.contains("true"),
        "handle_tab_key must return true to consume Tab"
    );
}

#[test]
fn pty_path_still_exists_for_script_terminals() {
    // The PTY/harness path must still exist for script-triggered terminals.
    assert!(
        TAB_AI_MODE_SOURCE.contains("fn open_tab_ai_harness_terminal_from_request"),
        "PTY harness terminal function must still exist"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AppView::QuickTerminalView"),
        "PTY path must still set QuickTerminalView"
    );
}

#[test]
fn acp_thread_supports_staged_context_and_initial_input() {
    // AcpThread must support staged context blocks and initial input for auto-submit.
    assert!(
        ACP_THREAD_SOURCE.contains("pending_context_blocks"),
        "AcpThread must have staged context fields"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("stage_context"),
        "AcpThread must expose a stage_context method"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("initial_input"),
        "AcpThreadInit must accept initial_input for auto-submit"
    );
}

#[test]
fn render_impl_handles_acp_chat_view() {
    assert!(
        RENDER_IMPL_SOURCE.contains("AppView::AcpChatView"),
        "render_impl must dispatch AcpChatView for rendering"
    );
}

#[test]
fn acp_view_renders_markdown_messages() {
    assert!(
        ACP_VIEW_SOURCE.contains("render_markdown_with_scope"),
        "AcpChatView must use markdown rendering for messages"
    );
}

#[test]
fn acp_view_has_permission_overlay() {
    assert!(
        ACP_VIEW_SOURCE.contains("render_permission_overlay"),
        "AcpChatView must render a permission approval overlay"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("approve_pending_permission"),
        "AcpChatView overlay must call approve_pending_permission"
    );
}

#[test]
fn acp_view_has_empty_and_streaming_states() {
    assert!(
        ACP_VIEW_SOURCE.contains("render_empty_state"),
        "AcpChatView must have an empty state"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("render_streaming_hint"),
        "AcpChatView must have a streaming indicator"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("render_status_badge"),
        "AcpChatView must have a status badge"
    );
}

#[test]
fn acp_mod_exports_required_types() {
    // The ACP module must re-export the key types used by tab_ai_mode.
    assert!(ACP_MOD_SOURCE.contains("pub(crate) use view::AcpChatView"));
    assert!(ACP_MOD_SOURCE.contains("pub(crate) use thread::"));
    assert!(ACP_MOD_SOURCE.contains("pub(crate) use permission_broker::"));
    assert!(ACP_MOD_SOURCE.contains("pub(crate) use client::"));
}

// =========================================================================
// Focused-target chip routing and Ask Anything fallback
// =========================================================================

#[test]
fn tab_ai_default_path_stages_focused_target_chip() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("build_tab_ai_focused_part_for_view"),
        "default Tab path must build a focused-target context part",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("AiContextPart::FocusedTarget"),
        "default Tab path must use a FocusedTarget context part",
    );
}

#[test]
fn tab_ai_has_explicit_ask_anything_fallback() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("tab_ai_ask_anything_fallback"),
        "Tab AI routing must have an explicit Ask Anything fallback log/event",
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("should_use_tab_ai_ask_anything_fallback"),
        "Tab AI routing must have a should_use_tab_ai_ask_anything_fallback helper",
    );
}

#[test]
fn tab_ai_focused_path_skips_ambient_capture() {
    // When a focused target is resolved, begin_tab_ai_harness_entry must NOT
    // call spawn_tab_ai_pre_switch_capture for the focused-target branch.
    let begin_fn_start = TAB_AI_MODE_SOURCE
        .find("fn begin_tab_ai_harness_entry(")
        .expect("begin_tab_ai_harness_entry must exist");
    let begin_fn_body = &TAB_AI_MODE_SOURCE[begin_fn_start..];
    let next_fn = begin_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(begin_fn_body.len());
    let begin_fn_body = &begin_fn_body[..next_fn];

    assert!(
        begin_fn_body.contains("if use_ask_anything_fallback"),
        "begin_tab_ai_harness_entry must gate ambient capture on ask_anything_fallback",
    );
    assert!(
        begin_fn_body.contains("tab_ai_focus_chip_staged"),
        "focused-target path must log tab_ai_focus_chip_staged",
    );
    assert!(
        begin_fn_body.contains("tab_ai_ask_anything_fallback"),
        "fallback path must log tab_ai_ask_anything_fallback",
    );
}

#[test]
fn tab_ai_acp_open_stages_chip_on_thread() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];

    assert!(
        open_fn_body.contains("thread.add_context_part(part, cx)"),
        "ACP open path must stage the focused chip on the thread",
    );
    assert!(
        open_fn_body.contains("acp_focused_chip_staged_on_thread"),
        "ACP open path must log focused-chip staging on the thread",
    );
}

#[test]
fn tab_ai_focused_path_marks_bootstrap_ready_without_ambient() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];

    assert!(
        open_fn_body.contains("mark_context_bootstrap_ready"),
        "focused-target path must mark bootstrap ready without waiting for deferred capture",
    );
}

#[test]
fn tab_ai_ask_anything_fallback_stages_resource_uri_chip() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];

    assert!(
        open_fn_body.contains("Ask Anything"),
        "Ask Anything fallback must stage a resource URI chip labeled 'Ask Anything'",
    );
    assert!(
        open_fn_body.contains("kit://context?profile=minimal"),
        "Ask Anything fallback must use the minimal desktop context profile",
    );
}

#[test]
fn acp_focused_chip_session_retains_focused_target_in_apply_back_route() {
    let focused_target_helper_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_focused_target_from_part(")
        .expect("tab_ai_focused_target_from_part must exist");
    let focused_target_helper_body = &TAB_AI_MODE_SOURCE[focused_target_helper_start..];
    let focused_target_helper_end = focused_target_helper_body[1..]
        .find("\n    fn ")
        .unwrap_or(focused_target_helper_body.len());
    let focused_target_helper_body = &focused_target_helper_body[..focused_target_helper_end];

    assert!(
        focused_target_helper_body.contains("AiContextPart::FocusedTarget { target, .. }"),
        "tab_ai_focused_target_from_part must extract AiContextPart::FocusedTarget",
    );
    assert!(
        focused_target_helper_body.contains("Some(target.clone())"),
        "tab_ai_focused_target_from_part must preserve the focused target payload",
    );
    assert!(
        focused_target_helper_body.contains("_ => None"),
        "tab_ai_focused_target_from_part must return None for non-focused parts",
    );

    let seed_fn_start = TAB_AI_MODE_SOURCE
        .find("fn seed_tab_ai_apply_back_route(")
        .expect("seed_tab_ai_apply_back_route must exist");
    let seed_fn_body = &TAB_AI_MODE_SOURCE[seed_fn_start..];
    let seed_fn_end = seed_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(seed_fn_body.len());
    let seed_fn_body = &seed_fn_body[..seed_fn_end];

    assert!(
        seed_fn_body.contains("Self::tab_ai_focused_target_from_part(focused_part)"),
        "seed_tab_ai_apply_back_route must derive focused_target from tab_ai_focused_target_from_part",
    );
    assert!(
        seed_fn_body.contains("focused_target: focused_target.clone()"),
        "seed_tab_ai_apply_back_route must persist the focused target into TabAiApplyBackRoute",
    );
    assert!(
        !seed_fn_body.contains("focused_target: None"),
        "seed_tab_ai_apply_back_route must not hardcode focused_target: None",
    );

    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let open_fn_end = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..open_fn_end];

    assert!(
        open_fn_body.contains("self.seed_tab_ai_apply_back_route("),
        "ACP open path must seed the apply-back route through seed_tab_ai_apply_back_route",
    );
    assert!(
        open_fn_body.contains("focused_part.as_ref()"),
        "ACP focused-chip open path must pass focused_part.as_ref() into seed_tab_ai_apply_back_route",
    );
    assert!(
        !open_fn_body.contains("focused_target: None"),
        "ACP open path must not hardcode focused_target: None during early route seeding",
    );
}

#[test]
fn ask_anything_fallback_seeds_route_without_focused_target() {
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let open_fn_end = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..open_fn_end];

    assert!(
        open_fn_body.contains("self.current_view = AppView::AcpChatView"),
        "Ask Anything fallback must still open the ACP chat view",
    );
    assert!(
        open_fn_body.contains("else if use_ask_anything_fallback"),
        "ACP open path must keep the Ask Anything fallback branch",
    );
    assert!(
        open_fn_body.contains("self.seed_tab_ai_apply_back_route("),
        "Ask Anything fallback must seed the apply-back route before deferred capture",
    );
    assert!(
        open_fn_body.contains("focused_part.as_ref()"),
        "Ask Anything fallback must pass the optional focused part into the route seed helper",
    );

    let seed_fn_start = TAB_AI_MODE_SOURCE
        .find("fn seed_tab_ai_apply_back_route(")
        .expect("seed_tab_ai_apply_back_route must exist");
    let seed_fn_body = &TAB_AI_MODE_SOURCE[seed_fn_start..];
    let seed_fn_end = seed_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(seed_fn_body.len());
    let seed_fn_body = &seed_fn_body[..seed_fn_end];

    assert!(
        seed_fn_body.contains("let focused_target = Self::tab_ai_focused_target_from_part(focused_part);"),
        "seed_tab_ai_apply_back_route must only derive a focused target from the optional focused part",
    );
    assert!(
        seed_fn_body.contains("has_focused_target = focused_target.is_some()"),
        "seed_tab_ai_apply_back_route must explicitly track whether a focused target exists",
    );
    assert!(
        !seed_fn_body.contains("focused_target: Some("),
        "seed_tab_ai_apply_back_route must not inject a spurious focused target when none exists",
    );
}

#[test]
fn acp_view_renders_pending_context_chips() {
    assert!(
        ACP_VIEW_SOURCE.contains("render_pending_context_chips"),
        "AcpChatView must render pending context chips",
    );
    assert!(
        ACP_VIEW_SOURCE.contains("acp-pending-context-chips"),
        "AcpChatView must have an element ID for the chips container",
    );
}

#[test]
fn acp_thread_supports_context_parts() {
    assert!(
        ACP_THREAD_SOURCE.contains("pending_context_parts"),
        "AcpThread must store typed context parts",
    );
    assert!(
        ACP_THREAD_SOURCE.contains("fn add_context_part("),
        "AcpThread must have an add_context_part method",
    );
    assert!(
        ACP_THREAD_SOURCE.contains("fn remove_context_part("),
        "AcpThread must have a remove_context_part method",
    );
    assert!(
        ACP_THREAD_SOURCE.contains("fn mark_context_bootstrap_ready("),
        "AcpThread must have a mark_context_bootstrap_ready method",
    );
}

#[test]
fn acp_thread_resolves_context_parts_on_submit() {
    assert!(
        ACP_THREAD_SOURCE.contains("acp_submit_resolved_context_parts"),
        "AcpThread must resolve context parts at submit time and log it",
    );
    assert!(
        ACP_THREAD_SOURCE.contains("resolve_context_parts_with_receipt"),
        "AcpThread must use resolve_context_parts_with_receipt for typed parts",
    );
}

// =========================================================================
// Query text and input text as first-class focused targets
// =========================================================================

#[test]
fn file_search_query_resolves_search_query_target_when_no_row_selected() {
    // When FileSearchView has a non-empty query but no selected row,
    // resolve_tab_ai_surface_targets_for_view must produce a search_query target.
    let resolve_fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let resolve_fn_body = &TAB_AI_MODE_SOURCE[resolve_fn_start..];

    // Find the FileSearchView arm
    let fs_arm_pos = resolve_fn_body
        .find("AppView::FileSearchView")
        .expect("FileSearchView arm must exist in target resolver");
    let fs_arm_body = &resolve_fn_body[fs_arm_pos..];

    assert!(
        fs_arm_body.contains("tab_ai_target_from_search_query"),
        "FileSearchView must fall back to tab_ai_target_from_search_query when no row is selected",
    );
    assert!(
        fs_arm_body.contains(r#""FileSearch""#),
        "FileSearchView search_query target must use source 'FileSearch'",
    );
}

#[test]
fn script_list_filter_resolves_search_query_target_when_no_row_selected() {
    let resolve_fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let resolve_fn_body = &TAB_AI_MODE_SOURCE[resolve_fn_start..];

    let sl_arm_pos = resolve_fn_body
        .find("AppView::ScriptList =>")
        .expect("ScriptList arm must exist in target resolver");
    let sl_arm_body = &resolve_fn_body[sl_arm_pos..];

    assert!(
        sl_arm_body.contains("tab_ai_target_from_search_query"),
        "ScriptList must fall back to tab_ai_target_from_search_query when no row is selected",
    );
    assert!(
        sl_arm_body.contains(r#""ScriptList""#),
        "ScriptList search_query target must use source 'ScriptList'",
    );
    assert!(
        sl_arm_body.contains("visibleResultCount"),
        "ScriptList search_query metadata must include visibleResultCount",
    );
}

#[test]
fn generic_prompt_input_resolves_input_target_when_no_semantic_selection() {
    let resolve_fn_start = TAB_AI_MODE_SOURCE
        .find("fn resolve_tab_ai_surface_targets_for_view(")
        .expect("resolve_tab_ai_surface_targets_for_view must exist");
    let resolve_fn_body = &TAB_AI_MODE_SOURCE[resolve_fn_start..];

    // The wildcard arm uses `_ => {` inside resolve_tab_ai_surface_targets_for_view
    let resolve_fn_end = resolve_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(resolve_fn_body.len());
    let resolve_fn_body = &resolve_fn_body[..resolve_fn_end];

    let wildcard_pos = resolve_fn_body
        .rfind("_ => {")
        .expect("wildcard arm must exist in target resolver");
    let wildcard_body = &resolve_fn_body[wildcard_pos..];

    assert!(
        wildcard_body.contains("tab_ai_target_from_input_text"),
        "wildcard arm must fall back to tab_ai_target_from_input_text when no semantic target",
    );
    assert!(
        wildcard_body.contains("ui.input_text"),
        "wildcard arm must check ui.input_text for the input fallback",
    );
}

#[test]
fn chip_prefix_maps_search_query_to_search() {
    assert!(
        TAB_AI_MODE_SOURCE.contains(r#""search_query" => "Search""#),
        "tab_ai_chip_prefix_for_kind must map search_query to 'Search'",
    );
}

#[test]
fn chip_prefix_maps_input_to_input() {
    assert!(
        TAB_AI_MODE_SOURCE.contains(r#""input" => "Input""#),
        "tab_ai_chip_prefix_for_kind must map input to 'Input'",
    );
}

#[test]
fn search_query_target_builder_logs_resolution_event() {
    let builder_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_target_from_search_query(")
        .expect("tab_ai_target_from_search_query must exist");
    let builder_body = &TAB_AI_MODE_SOURCE[builder_start..];
    let next_fn = builder_body[1..]
        .find("\n    fn ")
        .unwrap_or(builder_body.len());
    let builder_body = &builder_body[..next_fn];

    assert!(
        builder_body.contains(r#"event = "tab_ai_search_query_target_resolved""#),
        "search query target builder must log tab_ai_search_query_target_resolved",
    );
    assert!(
        builder_body.contains(r#"kind: "search_query""#),
        "search query target must have kind 'search_query'",
    );
}

#[test]
fn input_target_builder_logs_resolution_event() {
    let builder_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_target_from_input_text(")
        .expect("tab_ai_target_from_input_text must exist");
    let builder_body = &TAB_AI_MODE_SOURCE[builder_start..];
    let next_fn = builder_body[1..]
        .find("\n    fn ")
        .unwrap_or(builder_body.len());
    let builder_body = &builder_body[..next_fn];

    assert!(
        builder_body.contains(r#"event = "tab_ai_input_target_resolved""#),
        "input target builder must log tab_ai_input_target_resolved",
    );
    assert!(
        builder_body.contains(r#"kind: "input""#),
        "input target must have kind 'input'",
    );
    assert!(
        builder_body.contains("inputPreview"),
        "input target metadata must include inputPreview (not the full body)",
    );
    assert!(
        builder_body.contains("inputLength"),
        "input target metadata must include inputLength",
    );
}

#[test]
fn ask_anything_fallback_only_when_no_focused_target() {
    // The ask_anything_fallback must only trigger when no focused target
    // was resolved — query text and input text should prevent it.
    let begin_fn_start = TAB_AI_MODE_SOURCE
        .find("fn begin_tab_ai_harness_entry(")
        .expect("begin_tab_ai_harness_entry must exist");
    let begin_fn_body = &TAB_AI_MODE_SOURCE[begin_fn_start..];
    let next_fn = begin_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(begin_fn_body.len());
    let begin_fn_body = &begin_fn_body[..next_fn];

    assert!(
        begin_fn_body.contains("should_use_tab_ai_ask_anything_fallback"),
        "begin_tab_ai_harness_entry must call should_use_tab_ai_ask_anything_fallback",
    );
    // The fallback helper must check for absence of focused target
    let fallback_fn_start = TAB_AI_MODE_SOURCE
        .find("fn should_use_tab_ai_ask_anything_fallback(")
        .expect("should_use_tab_ai_ask_anything_fallback must exist");
    let fallback_fn_body = &TAB_AI_MODE_SOURCE[fallback_fn_start..];
    let next_fn = fallback_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(fallback_fn_body.len());
    let fallback_fn_body = &fallback_fn_body[..next_fn];

    assert!(
        fallback_fn_body.contains("focused_target") || fallback_fn_body.contains("focused_part"),
        "ask_anything_fallback must check for absence of a focused target or focused part",
    );
}

#[test]
fn search_query_and_input_targets_prevent_ambient_capture() {
    // When a search_query or input target is resolved, the code takes
    // the focused-target path (not the ask_anything fallback), which
    // means ambient capture is NOT triggered.
    let begin_fn_start = TAB_AI_MODE_SOURCE
        .find("fn begin_tab_ai_harness_entry(")
        .expect("begin_tab_ai_harness_entry must exist");
    let begin_fn_body = &TAB_AI_MODE_SOURCE[begin_fn_start..];
    let next_fn = begin_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(begin_fn_body.len());
    let begin_fn_body = &begin_fn_body[..next_fn];

    // The focused-target path and the ask_anything path must be mutually
    // exclusive branches — ambient capture only runs under use_ask_anything_fallback.
    assert!(
        begin_fn_body.contains("if use_ask_anything_fallback"),
        "ambient capture must be gated behind use_ask_anything_fallback",
    );
    assert!(
        begin_fn_body.contains("tab_ai_focus_chip_staged"),
        "focused-target path must log tab_ai_focus_chip_staged",
    );
    assert!(
        begin_fn_body.contains("tab_ai_ask_anything_fallback"),
        "ask_anything fallback path must log tab_ai_ask_anything_fallback",
    );
    // The focused-target branch must NOT call spawn_tab_ai_pre_switch_capture
    // — only the ask_anything branch does.
    assert!(
        begin_fn_body.contains("if !use_ask_anything_fallback"),
        "focused-target part must be built only when NOT using ask_anything_fallback",
    );
}

// =========================================================================
// Mandatory script verification guidance: ACP path parity
// =========================================================================

#[test]
fn acp_path_emits_initial_input_builder_telemetry() {
    let acp_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let acp_fn_body = &TAB_AI_MODE_SOURCE[acp_fn_start..];
    let next_fn = acp_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(acp_fn_body.len());
    let acp_fn_body = &acp_fn_body[..next_fn];

    assert!(
        acp_fn_body.contains("event = \"tab_ai_acp_initial_input_built\""),
        "ACP path must emit the shared initial-input telemetry event"
    );
    assert!(
        acp_fn_body.contains("guidance_appended"),
        "ACP telemetry must record whether artifact guidance was appended"
    );
    assert!(
        acp_fn_body.contains("forced_by_script_list_submit"),
        "ACP telemetry must record whether ScriptList submit forced guidance"
    );
    assert!(
        acp_fn_body.contains("includes_script_authoring_skill"),
        "ACP telemetry must check whether guidance references the script-authoring skill"
    );
    assert!(
        acp_fn_body.contains("includes_bun_build_verification"),
        "ACP telemetry must check whether guidance includes the bun build command"
    );
    assert!(
        acp_fn_body.contains("includes_bun_execute_verification"),
        "ACP telemetry must check whether guidance includes the SK_VERIFY bun execute command"
    );
}

#[test]
fn acp_path_uses_shared_initial_input_builder() {
    let acp_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_acp_view_from_request_impl(")
        .expect("open_tab_ai_acp_view_from_request_impl must exist");
    let acp_fn_body = &TAB_AI_MODE_SOURCE[acp_fn_start..];
    let next_fn = acp_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(acp_fn_body.len());
    let acp_fn_body = &acp_fn_body[..next_fn];

    assert!(
        acp_fn_body.contains("build_tab_ai_acp_initial_input_for_prompt"),
        "ACP path must delegate initial-input formatting to the shared helper"
    );
}

#[test]
fn harness_source_builds_acp_initial_input_with_guidance_before_user_intent() {
    assert!(
        HARNESS_SOURCE.contains("pub(crate) fn build_tab_ai_acp_initial_input_for_prompt("),
        "harness module must define the shared ACP initial-input builder"
    );
    assert!(
        HARNESS_SOURCE.contains(r#"format!("{guidance}\n\nUser intent:\n{intent}\n")"#),
        "shared ACP initial-input builder must place guidance before the User intent line"
    );
    assert!(
        HARNESS_SOURCE.contains("TabAiHarnessSubmissionMode::Submit"),
        "shared ACP initial-input builder must always use Submit mode"
    );
}

#[test]
fn harness_source_contains_launchpad_include() {
    // The harness module must embed START_HERE.md as the single source of
    // truth for artifact authoring guidance. If the include path changes,
    // this test catches it.
    assert!(
        HARNESS_SOURCE.contains("kit-init/examples/START_HERE.md"),
        "harness module must include START_HERE.md as the canonical launchpad source"
    );
}
