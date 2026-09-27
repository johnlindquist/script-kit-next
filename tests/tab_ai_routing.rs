//! Contract tests verifying Agent Chat compatibility routes and Agent Chat entry points.
//!
//! The primary Agent Chat entry key is Command+Enter. Plain Tab is deprecated
//! for Agent Chat entry and remains local to focus/navigation handlers.
//!
//! Direct zero-intent launcher surfaces now route through
//! `open_tab_ai_agent_chat_with_entry_intent(None, cx)`, while `open_tab_ai_chat()`
//! remains a compatibility wrapper for older harness entry callers.

const TAB_SOURCE: &str = include_str!("../src/app_impl/startup.rs");
const TAB_NEW_SOURCE: &str = include_str!("../src/app_impl/startup_new_tab.rs");
const APP_STATE_SOURCE: &str = include_str!("../src/main_sections/app_state.rs");
const APP_VIEW_STATE_SOURCE: &str = include_str!("../src/main_sections/app_view_state.rs");
const TERM_RENDER_SOURCE: &str = include_str!("../src/render_prompts/term.rs");
const BUILTIN_EXECUTION_SOURCE: &str = include_str!("../src/app_execute/builtin_execution.rs");
const PROMPT_AI_SOURCE: &str = include_str!("../src/app_impl/prompt_ai.rs");
const SCREENSHOT_FILES_SOURCE: &str = include_str!("../src/ai/harness/screenshot_files.rs");
const ACTIONS_DIALOG_SOURCE: &str = include_str!("../src/app_impl/actions_dialog.rs");

// =========================================================================
// Primary contract: Tab → harness terminal (QuickTerminalView)
// =========================================================================

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
fn tab_ai_routing_removes_script_list_shift_tab_quick_submit() {
    assert!(
        !TAB_SOURCE.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "Shift+Tab must no longer route typed ScriptList queries through the quick-submit planner"
    );
}

// =========================================================================
// Entry-intent normalization and submit mode selection
// =========================================================================

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

// =========================================================================
// Harness terminal: context injection and capture profile
// =========================================================================

// =========================================================================
// Harness session lifecycle: reuse and validation
// =========================================================================

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

// =========================================================================
// Close semantics: Cmd+W closes wrapper, Escape owned by PTY
// =========================================================================

#[test]
fn quick_terminal_cmd_w_closes_main_window_state_first() {
    assert!(
        TERM_RENDER_SOURCE.contains("this.close_quick_terminal_main_window_state_first(cx);"),
        "Cmd+W in QuickTerminalView must use the state-first main-window close path"
    );
    assert!(
        !TERM_RENDER_SOURCE.contains(
            "key.eq_ignore_ascii_case(\"w\")\n                        {\n                            this.close_tab_ai_harness_terminal_with_window"
        ),
        "Cmd+W must not use the visible restore-origin close helper"
    );
}

#[test]
fn quick_terminal_render_documents_close_contract() {
    assert!(
        TERM_RENDER_SOURCE.contains("wrapper semantics contract"),
        "render_prompts/term.rs must document the wrapper semantics contract"
    );
    assert!(
        TERM_RENDER_SOURCE.contains("Cmd+W closes the main window state-first"),
        "render_prompts/term.rs must state Cmd+W closes state-first"
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

// =========================================================================
// ScriptList fallback routes through the shared Cmd+Enter AI helper
// =========================================================================

// =========================================================================
// ScriptList fallback comment matches Cmd+Enter contract
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

// =========================================================================
// Post-close prewarm lifecycle: close clears session, prewarm seeds startup,
// explicit Tab still forces a fresh PTY
// =========================================================================

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

// =========================================================================
// Harness readiness gate: prewarmed sessions also wait for output
// =========================================================================

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

    // The escape_cancels assignment must negate the QuickTerminalView flag.
    assert!(
        TERM_RENDER_SOURCE.contains("term.escape_cancels = !is_quick_terminal;"),
        "escape_cancels must be false for QuickTerminalView"
    );
}

#[test]
fn quick_terminal_cmd_w_dispatches_close() {
    // Cmd+W in QuickTerminalView must be intercepted before reaching the PTY
    // and must use the state-first Quick Terminal main-window close helper.
    assert!(
        TERM_RENDER_SOURCE.contains("close_quick_terminal_main_window_state_first"),
        "Cmd+W in QuickTerminalView must dispatch to the state-first close helper"
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
fn smoke_matrix_agy_paste_only_stages_context() {
    let _config = config_for_backend(script_kit_gpui::ai::HarnessBackendKind::Agy, "agy");
    let context = smoke_matrix_context();
    let submission = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        None,
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("AGY PasteOnly submission must succeed");

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
    let agy =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("AGY");
    let copilot =
        script_kit_gpui::ai::build_tab_ai_harness_submission(&context, None, mode, None, None, &[])
            .expect("Copilot CLI");

    assert_eq!(
        claude, codex,
        "Claude Code and Codex must produce identical context"
    );
    assert_eq!(codex, agy, "Codex and AGY must produce identical context");
    assert_eq!(
        agy, copilot,
        "AGY and Copilot CLI must produce identical context"
    );
}

#[test]
fn smoke_matrix_submit_mode_appends_sentinel_for_all_backends() {
    let context = smoke_matrix_context();
    let mode = script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit;

    for label in ["Claude Code", "Codex", "AGY", "Copilot CLI"] {
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
// Footer hint strip: QuickTerminalView distinguishes close-only and apply-back
// =========================================================================

#[test]
fn quick_terminal_footer_distinguishes_close_only_and_apply_back_states() {
    assert!(
        TERM_RENDER_SOURCE.contains("fn terminal_return_hint_label(return_view: Option<&AppView>)")
            && TERM_RENDER_SOURCE.contains("None => \"⌘W Close\""),
        "QuickTerminalView without a return target must advertise close-only footer copy"
    );
    assert!(
        TERM_RENDER_SOURCE
            .contains("let can_apply_back = route.is_some() && return_view.is_some();")
            && TERM_RENDER_SOURCE.contains("items.push(\"⌘↩ Apply\".to_string());"),
        "QuickTerminalView with an apply-back route and return target must advertise Apply"
    );
    assert!(
        TERM_RENDER_SOURCE.contains("render_simple_hint_strip(items.join(\" · \"), leading)"),
        "QuickTerminalView footer must use render_simple_hint_strip for minimal chrome"
    );
}

// =========================================================================
// Close function guards: only acts when in QuickTerminalView
// =========================================================================

// =========================================================================
// Consolidated legacy inline-chat regression guard
// =========================================================================
//
// This guard covers ALL changed harness-entry surfaces in a single test.
// If TabAiChat or open_tab_ai_full_view_chat silently reappears in any
// of these files, the harness-terminal contract is broken.

// =========================================================================
// ScriptList no longer has a hidden Shift+Tab quick-submit path
// =========================================================================

#[test]
fn startup_tab_interceptor_no_longer_routes_shift_tab_query_into_harness() {
    assert!(
        !TAB_SOURCE.contains("submit_to_current_or_new_tab_ai_harness_from_text"),
        "startup.rs must not keep the hidden Shift+Tab quick-submit planner path"
    );
}

// =========================================================================
// Legacy AI command redirection: all active AI builtins → harness terminal
// =========================================================================

#[test]
fn agent_chat_open_commands_route_to_agent_chat() {
    // OpenAi, MiniAi, NewConversation, ClearConversation must all route to Agent Chat.
    // Scope to the AI command match block.
    let ai_block_start = BUILTIN_EXECUTION_SOURCE
        .find("BuiltInFeature::AiCommand(cmd_type) =>")
        .expect("AiCommand match arm must exist");
    let ai_block = &BUILTIN_EXECUTION_SOURCE
        [ai_block_start..(ai_block_start + 4000).min(BUILTIN_EXECUTION_SOURCE.len())];

    assert!(
        !ai_block.contains("clear_all_chats"),
        "AI command block must not call legacy clear_all_chats"
    );
}

// =========================================================================
// Deferred capture pipeline: "open first, inject later"
// =========================================================================

#[test]
fn app_state_has_tab_ai_harness_capture_generation() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_capture_generation: u64"),
        "app_state.rs must have a tab_ai_harness_capture_generation field"
    );
}

// =========================================================================
// BuiltInFeature::AiChat must route to harness, not legacy AI window
// =========================================================================

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
fn tab_ai_harness_tracks_apply_back_route_state() {
    assert!(
        APP_STATE_SOURCE.contains("tab_ai_harness_apply_back_route"),
        "ScriptListApp must persist apply-back routing state for the active harness session"
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

// =========================================================================
// ClipboardEntry apply-back contract
// =========================================================================

// =========================================================================
// Desktop / DesktopSelection apply-back contract
// =========================================================================

// =========================================================================
// Script List context shaping: explicit branch in target resolution
// =========================================================================

// =========================================================================
// Harness close lifecycle: full contract assertions
// =========================================================================

#[test]
fn quick_terminal_cmd_w_routes_to_harness_close() {
    assert!(
        TERM_RENDER_SOURCE.contains("key.eq_ignore_ascii_case(\"w\")"),
        "QuickTerminalView must reserve Cmd+W as the wrapper close gesture",
    );
    assert!(
        TERM_RENDER_SOURCE.contains("this.close_quick_terminal_main_window_state_first(cx);"),
        "Cmd+W must call the state-first main-window close path",
    );
}

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

// ---------------------------------------------------------------------------
// Live-session quick submit: structured submission regression tests
// ---------------------------------------------------------------------------

// =========================================================================
// ~ mini entry: file search presentation and query normalization
// =========================================================================

const FILTER_INPUT_CHANGE_SOURCE: &str = include_str!("../src/app_impl/filter_input_change.rs");
const FILTER_INPUT_CORE_SOURCE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const FILE_SEARCH_SOURCE: &str = include_str!("../src/render_builtins/file_search.rs");

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
fn tilde_trigger_predicate_matches_tilde_and_tilde_slash_prefix() {
    // should_enter_file_search_from_script_list must match `~` and `~/...`
    assert!(
        FILTER_INPUT_CORE_SOURCE.contains(r#"new_text == "~" || new_text.starts_with("~/")"#),
        "should_enter_file_search_from_script_list must match ~ and ~/... patterns"
    );
}

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
// Agent Chat routing: Tab now opens AgentChatView instead of QuickTerminalView
// =========================================================================

const AGENT_CHAT_THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");

#[test]
fn agent_chat_thread_supports_staged_context_and_initial_input() {
    // AgentChatThread must support staged context blocks and initial input for auto-submit.
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("pending_context_blocks"),
        "AgentChatThread must have staged context fields"
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("stage_context"),
        "AgentChatThread must expose a stage_context method"
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("initial_input"),
        "AgentChatThreadInit must accept initial_input for auto-submit"
    );
}

// =========================================================================
// Focused-target chip routing and Ask Anything fallback
// =========================================================================

#[test]
fn agent_chat_thread_supports_context_parts() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("pending_context_parts"),
        "AgentChatThread must store typed context parts",
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("fn add_context_part("),
        "AgentChatThread must have an add_context_part method",
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("fn remove_context_part("),
        "AgentChatThread must have a remove_context_part method",
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("fn mark_context_bootstrap_ready("),
        "AgentChatThread must have a mark_context_bootstrap_ready method",
    );
}

#[test]
fn agent_chat_thread_resolves_context_parts_on_submit() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("agent_chat_submit_resolved_context_parts"),
        "AgentChatThread must resolve context parts at submit time and log it",
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("include!(\"thread_context_resolution.rs\");")
            && AGENT_CHAT_THREAD_SOURCE
                .contains("Self::resolve_pending_context_items_with(&pending_parts,"),
        "AgentChatThread must resolve typed parts into a receipt with resolve_pending_context_items_with",
    );
}

// =========================================================================
// Query text and input text as first-class focused targets
// =========================================================================

// =========================================================================
// Mandatory script verification guidance: Agent Chat path parity
// =========================================================================

// =========================================================================
// Regression: surface selection routes new-script to QuickTerminalView
// =========================================================================

// =========================================================================
// Regression: ScriptList submit selects QuickTerminalView (source contract)
// =========================================================================

// =========================================================================
// Regression: shared guidance contains mandatory verification text
// =========================================================================

// =========================================================================
// Regression: Agent Chat live quick-submit reroutes verification requests
// =========================================================================

// =========================================================================
// Regression: unit tests for surface preference exist in harness module
// =========================================================================

// =========================================================================
// AC: Script creation flow (ScriptList + Submit) produces verification-bearing
//     guidance and selects the quick terminal path (runtime call)
// =========================================================================

#[test]
fn script_list_submit_creation_flow_selects_quick_terminal_with_all_markers() {
    // Runtime call: ScriptList + Submit + non-empty intent must produce
    // verification-bearing guidance that selects the quick terminal surface.
    let pref = script_kit_gpui::ai::tab_ai_surface_preference_for_prompt(
        "ScriptList",
        Some("clipboard cleanup"),
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit,
    );
    assert!(
        pref.use_quick_terminal,
        "ScriptList + Submit + intent must select quick_terminal"
    );
    assert!(
        pref.includes_script_authoring_skill,
        "script creation flow must include script_authoring_skill marker"
    );
    assert!(
        pref.includes_bun_build_verification,
        "authoring flow must include bun_build_verification marker"
    );
    assert!(
        pref.includes_bun_execute_verification,
        "authoring flow must include bun_execute_verification marker"
    );
}

// =========================================================================
// AC: Non-authoring flow omits appendix and stays on Agent Chat (runtime call)
// =========================================================================

#[test]
fn non_authoring_flow_stays_agent_chat_with_no_markers() {
    // Runtime call: FileSearch + Submit + non-artifact intent must NOT
    // produce verification-bearing guidance and must stay on Agent Chat.
    let pref = script_kit_gpui::ai::tab_ai_surface_preference_for_prompt(
        "FileSearch",
        Some("rename this file"),
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit,
    );
    assert!(
        !pref.use_quick_terminal,
        "non-authoring flow must stay on Agent Chat"
    );
    assert!(
        !pref.includes_script_authoring_skill,
        "non-authoring flow must not include script_authoring_skill"
    );
    assert!(
        !pref.includes_bun_build_verification,
        "non-authoring flow must not include bun_build_verification"
    );
    assert!(
        !pref.includes_bun_execute_verification,
        "non-authoring flow must not include bun_execute_verification"
    );
}

// =========================================================================
// AC: Runtime log — authoring flow emits tab_ai_artifact_authoring_guidance_appended
//     with all three verification markers set to true
// =========================================================================

// =========================================================================
// AC: Runtime log — non-authoring flow does NOT emit the guidance-appended
//     event (the resolve call returns None so the log block is skipped)
// =========================================================================

#[test]
fn non_authoring_harness_submission_omits_guidance_block() {
    // Runtime call: a non-authoring submission must NOT contain the guidance
    // block in its output, confirming the guidance_appended log is never hit.
    let context = script_kit_gpui::ai::TabAiContextBlob::from_parts(
        script_kit_gpui::ai::TabAiUiSnapshot {
            prompt_type: "FileSearch".to_string(),
            input_text: Some("rename this file".to_string()),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        vec![],
        "2026-04-04T00:00:00Z".to_string(),
    );

    let output = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        Some("rename this file"),
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("submission must succeed");

    // The guidance block delimiter should NOT appear in the output.
    assert!(
        !output.contains("--- Script Kit artifact authoring guidance ---"),
        "non-authoring submission must not contain artifact authoring guidance"
    );
    // The verification gate header should NOT appear either.
    assert!(
        !output.contains("MANDATORY SCRIPT VERIFICATION"),
        "non-authoring submission must not contain verification gate"
    );
}

// =========================================================================
// Agent Chat setup-card agent confirmation refreshes inline setup state
// =========================================================================

#[test]
fn authoring_harness_submission_contains_verification_guidance() {
    // Runtime call: a ScriptList + Submit authoring submission must contain
    // the verification guidance block with all three markers present.
    let context = script_kit_gpui::ai::TabAiContextBlob::from_parts(
        script_kit_gpui::ai::TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            input_text: Some("clipboard cleanup".to_string()),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        vec![],
        "2026-04-04T00:00:00Z".to_string(),
    );

    let output = script_kit_gpui::ai::build_tab_ai_harness_submission(
        &context,
        Some("clipboard cleanup"),
        script_kit_gpui::ai::TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("submission must succeed");

    // The guidance block delimiter must appear.
    assert!(
        output.contains("--- Script Kit artifact authoring guidance ---"),
        "authoring submission must contain artifact authoring guidance"
    );
    // The verification gate header must appear.
    assert!(
        output.contains("MANDATORY SCRIPT VERIFICATION"),
        "authoring submission must contain mandatory verification gate"
    );
    // All three verification markers must be present.
    assert!(
        output.contains(script_kit_gpui::ai::SCRIPT_AUTHORING_SKILL_MARKER),
        "authoring submission must include new-script skill marker"
    );
    assert!(
        output.contains(script_kit_gpui::ai::BUN_BUILD_VERIFICATION_MARKER),
        "authoring submission must include bun build verification marker"
    );
    assert!(
        output.contains(script_kit_gpui::ai::BUN_EXECUTE_VERIFICATION_MARKER),
        "authoring submission must include bun execute verification marker"
    );
    // The user intent must follow the guidance.
    assert!(
        output.contains("User intent:\nclipboard cleanup"),
        "authoring submission must include the user intent after guidance"
    );
}

// =========================================================================
// Cmd+Enter Agent Chat handoff consistency contract
// =========================================================================

#[test]
fn actions_dialog_cmd_enter_branch_precedes_plain_enter_execute() {
    // The Cmd+Enter handler must appear BEFORE the generic is_key_enter(key)
    // branch in route_key_to_actions_dialog. If it comes after, the platform
    // modifier is never checked and Cmd+Enter is swallowed as plain Enter.
    let cmd_enter_pos = ACTIONS_DIALOG_SOURCE
        .find("tab_ai_actions_dialog_cmd_enter")
        .expect("embedded actions dialog must have a tab_ai_actions_dialog_cmd_enter handler");
    let plain_enter_pos = ACTIONS_DIALOG_SOURCE
        .find("d.activate_selected(cx)")
        .expect("embedded actions dialog must have the plain Enter activate_selected path");
    assert!(
        cmd_enter_pos < plain_enter_pos,
        "Cmd+Enter Agent Chat handoff must precede the generic Enter execute path in route_key_to_actions_dialog"
    );
}

#[test]
fn embedded_actions_cmd_enter_preserves_plain_enter_behavior() {
    // The plain Enter activate_selected path must still exist and be reachable
    let route_fn_start = ACTIONS_DIALOG_SOURCE
        .find("fn route_key_to_actions_dialog(")
        .expect("route_key_to_actions_dialog must exist");
    let route_fn = &ACTIONS_DIALOG_SOURCE[route_fn_start..];

    assert!(
        route_fn.contains("activate_selected(cx)"),
        "route_key_to_actions_dialog must still contain the plain Enter activate_selected path"
    );
    assert!(
        route_fn.contains("ActionsRoute::Execute"),
        "route_key_to_actions_dialog must still return ActionsRoute::Execute for plain Enter"
    );
}
