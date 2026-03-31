//! Integration tests for Tab AI harness submission modes.
//!
//! Validates that `build_tab_ai_harness_submission` correctly distinguishes
//! between PasteOnly (stage context) and Submit (immediate turn) modes,
//! and that the harness session lifecycle supports warm reuse.

use script_kit_gpui::ai::{
    build_tab_ai_harness_submission, validate_tab_ai_harness_config, HarnessConfig,
    TabAiContextBlob, TabAiFieldStatus, TabAiHarnessSubmissionMode, TabAiInvocationReceipt,
    TabAiSuggestedIntentSpec, TabAiUiSnapshot, TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION,
};

fn make_context(prompt_type: &str, input_text: Option<&str>) -> TabAiContextBlob {
    TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: prompt_type.to_string(),
            input_text: input_text.map(|s| s.to_string()),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        vec![],
        "2026-03-29T07:07:06Z".to_string(),
    )
}

// =========================================================================
// PasteOnly mode: stages context without auto-submitting
// =========================================================================

#[test]
fn paste_only_without_intent_omits_sentinel() {
    let context = make_context("FileSearch", Some("readme"));
    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    assert!(submission.contains("Script Kit context"));
    assert!(
        !submission.contains("Await the user's next terminal input."),
        "PasteOnly mode must not append the wait sentinel"
    );
    assert!(
        !submission.contains("User intent:"),
        "PasteOnly without intent must not contain intent section"
    );
}

#[test]
fn paste_only_with_intent_includes_intent() {
    let context = make_context("ScriptList", None);
    let submission = build_tab_ai_harness_submission(
        &context,
        Some("open settings"),
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    assert!(submission.contains("User intent:\nopen settings"));
    assert!(
        !submission.contains("Await the user's next terminal input."),
        "PasteOnly with intent must not append the wait sentinel"
    );
}

#[test]
fn paste_only_stages_context_block_with_schema_version() {
    let context = make_context("ScriptList", Some("hello"));
    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("should build");

    assert!(
        submission.starts_with("Script Kit context"),
        "staged context must start with the flat labeled header"
    );
    assert!(
        submission.contains("prompt type: ScriptList"),
        "staged context must include the UI snapshot prompt type"
    );
}

// =========================================================================
// Submit mode: full turn behavior
// =========================================================================

#[test]
fn submit_without_intent_includes_sentinel() {
    let context = make_context("ScriptList", None);
    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    assert!(submission.contains("Script Kit context"));
    assert!(
        submission.contains("Await the user's next terminal input."),
        "Submit mode without intent must append the wait sentinel"
    );
}

#[test]
fn submit_with_intent_includes_intent_and_omits_sentinel() {
    let context = make_context("FileSearch", Some("readme"));
    let submission = build_tab_ai_harness_submission(
        &context,
        Some("rename this file"),
        TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    assert!(submission.contains("User intent:\nrename this file"));
    assert!(
        !submission.contains("Await the user's next terminal input."),
        "Submit with intent must not append the wait sentinel"
    );
}

// =========================================================================
// Edge cases: blank intent, whitespace-only
// =========================================================================

#[test]
fn blank_intent_treated_as_none() {
    let context = make_context("ScriptList", None);

    let paste = build_tab_ai_harness_submission(
        &context,
        Some("   "),
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("should build");
    assert!(!paste.contains("User intent:"));
    assert!(!paste.contains("Await the user"));

    let submit = build_tab_ai_harness_submission(
        &context,
        Some("   "),
        TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("should build");
    assert!(!submit.contains("User intent:"));
    assert!(submit.contains("Await the user's next terminal input."));
}

#[test]
fn empty_string_intent_treated_as_none() {
    let context = make_context("ScriptList", None);
    let paste = build_tab_ai_harness_submission(
        &context,
        Some(""),
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("should build");
    assert!(!paste.contains("User intent:"));
    assert!(!paste.contains("Await the user"));
}

// =========================================================================
// Context blob content in submissions
// =========================================================================

#[test]
fn submission_includes_input_text_from_context() {
    let context = make_context("FileSearch", Some("my-query"));
    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("should build");

    assert!(
        submission.contains("my-query"),
        "submission must include the input text from the context blob"
    );
}

#[test]
fn submission_includes_prompt_type_from_context() {
    let context = make_context("ClipboardHistory", None);
    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("should build");

    assert!(
        submission.contains("ClipboardHistory"),
        "submission must include the prompt type from the context blob"
    );
}

// =========================================================================
// Harness config validation
// =========================================================================

#[test]
fn default_config_targets_claude_code() {
    let config = HarnessConfig::default();
    assert_eq!(
        config.command, "claude",
        "default harness must target Claude Code"
    );
}

#[test]
fn config_validation_accepts_known_binary() {
    let config = HarnessConfig {
        command: "sh".to_string(),
        ..Default::default()
    };
    assert!(
        validate_tab_ai_harness_config(&config).is_ok(),
        "known binary must pass validation"
    );
}

#[test]
fn config_validation_rejects_empty_command() {
    let config = HarnessConfig {
        command: "".to_string(),
        ..Default::default()
    };
    let err = validate_tab_ai_harness_config(&config).expect_err("empty command must fail");
    assert!(
        err.contains("harness.json"),
        "must mention config file: {err}"
    );
}

#[test]
fn config_validation_rejects_nonexistent_binary() {
    let config = HarnessConfig {
        command: "nonexistent-xyz-42".to_string(),
        ..Default::default()
    };
    let err = validate_tab_ai_harness_config(&config).expect_err("missing CLI must fail");
    assert!(
        err.contains("not found on PATH"),
        "must mention PATH: {err}"
    );
}

// =========================================================================
// Session reuse contract (source-text assertions)
// =========================================================================

const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");

#[test]
fn open_harness_terminal_saves_return_view_before_switching() {
    // The harness terminal must save the originating view BEFORE switching current_view.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    let save_pos = open_fn_body
        .find("tab_ai_harness_return_view")
        .expect("must save the return view");
    let switch_pos = open_fn_body
        .find("self.current_view = AppView::QuickTerminalView")
        .expect("must switch to QuickTerminalView");

    assert!(
        save_pos < switch_pos,
        "return view must be saved BEFORE switching to QuickTerminalView"
    );
}

#[test]
fn warm_reentry_does_not_respawn_pty() {
    // ensure_tab_ai_harness_terminal returns early when existing session is alive.
    let ensure_fn_start = TAB_AI_MODE_SOURCE
        .find("fn ensure_tab_ai_harness_terminal(")
        .expect("ensure_tab_ai_harness_terminal must exist");
    let ensure_fn_body = &TAB_AI_MODE_SOURCE[ensure_fn_start..];
    let next_fn = ensure_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(ensure_fn_body.len());
    let ensure_fn_body = &ensure_fn_body[..next_fn];

    // Must check existing session before spawning new PTY
    let alive_check_pos = ensure_fn_body
        .find("is_alive()")
        .expect("must check is_alive()");
    let new_spawn_pos = ensure_fn_body
        .find("TermPrompt::with_height")
        .expect("must have PTY spawn path");

    assert!(
        alive_check_pos < new_spawn_pos,
        "alive check must come before PTY spawn — reuse existing session first"
    );
}

// =========================================================================
// Hints block regression: receipt + suggestions → <scriptKitHints>
// =========================================================================

fn sample_invocation_receipt(prompt_type: &str) -> TabAiInvocationReceipt {
    TabAiInvocationReceipt {
        schema_version: TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION,
        prompt_type: prompt_type.to_string(),
        input_status: TabAiFieldStatus::Captured,
        focus_status: TabAiFieldStatus::Captured,
        elements_status: TabAiFieldStatus::Captured,
        element_count: 5,
        warning_count: 0,
        has_focus_target: true,
        has_input_text: false,
        degradation_reasons: vec![],
        rich: true,
    }
}

#[test]
fn paste_only_includes_hints_block_when_receipt_and_suggestions_provided() {
    let context = make_context("FileSearch", Some("readme"));
    let receipt = sample_invocation_receipt("FileSearch");
    let suggestions = vec![
        TabAiSuggestedIntentSpec::new("Summarize", "summarize this file"),
        TabAiSuggestedIntentSpec::new("Rename", "rename this file"),
    ];

    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        Some(&receipt),
        &suggestions,
    )
    .expect("submission should build");

    assert!(
        submission.contains("<scriptKitHints>"),
        "PasteOnly with receipt+suggestions must include the hints block"
    );
    assert!(
        submission.contains("</scriptKitHints>"),
        "hints block must be properly closed"
    );
    assert!(
        submission.contains("\"promptType\": \"FileSearch\""),
        "hints block must include the invocation receipt prompt type"
    );
    assert!(
        submission.contains("\"intent\": \"summarize this file\""),
        "hints block must include suggested intents"
    );
    assert!(
        submission.contains("\"intent\": \"rename this file\""),
        "hints block must include all suggested intents"
    );
    assert!(
        submission.ends_with('\n'),
        "PasteOnly submission must end on a fresh line"
    );
}

#[test]
fn paste_only_omits_hints_block_when_no_receipt_or_suggestions() {
    let context = make_context("ScriptList", None);

    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    assert!(
        !submission.contains("<scriptKitHints>"),
        "PasteOnly without receipt/suggestions must NOT include hints block"
    );
    assert!(
        submission.contains("Script Kit context"),
        "context block must still be present"
    );
}

#[test]
fn submit_mode_also_includes_hints_block_when_provided() {
    let context = make_context("ScriptList", None);
    let receipt = sample_invocation_receipt("ScriptList");
    let suggestions = vec![TabAiSuggestedIntentSpec::new(
        "Open settings",
        "open settings",
    )];

    let submission = build_tab_ai_harness_submission(
        &context,
        None,
        TabAiHarnessSubmissionMode::Submit,
        None,
        Some(&receipt),
        &suggestions,
    )
    .expect("submission should build");

    assert!(
        submission.contains("<scriptKitHints>"),
        "Submit mode with receipt must include hints block"
    );
    assert!(
        submission.contains("Await the user's next terminal input."),
        "Submit mode without intent must still append wait sentinel after hints"
    );
}

#[test]
fn hints_block_appears_between_context_and_intent() {
    let context = make_context("FileSearch", Some("readme"));
    let receipt = sample_invocation_receipt("FileSearch");
    let suggestions = vec![TabAiSuggestedIntentSpec::new("Summarize", "summarize this")];

    let submission = build_tab_ai_harness_submission(
        &context,
        Some("rename this file"),
        TabAiHarnessSubmissionMode::PasteOnly,
        None,
        Some(&receipt),
        &suggestions,
    )
    .expect("submission should build");

    let context_start = submission
        .find("Script Kit context")
        .expect("context block must exist");
    let hints_start = submission
        .find("<scriptKitHints>")
        .expect("hints block must exist");
    let intent_start = submission.find("User intent:").expect("intent must exist");

    assert!(
        context_start < hints_start,
        "hints block must come after the context block"
    );
    assert!(
        hints_start < intent_start,
        "hints block must come before the user intent"
    );
}

// =========================================================================
// Readiness gate: source-level regression for output-based wait
// =========================================================================

#[test]
fn readiness_gate_checks_has_received_output_not_was_cold_start() {
    // The readiness check method must be based on has_received_output,
    // NOT on was_cold_start. This ensures prewarmed sessions that haven't
    // printed their prompt yet still trigger the readiness wait.
    let readiness_fn_start = TAB_AI_MODE_SOURCE
        .find("fn tab_ai_harness_needs_readiness_wait(")
        .expect("tab_ai_harness_needs_readiness_wait must exist");
    let readiness_fn_body = &TAB_AI_MODE_SOURCE[readiness_fn_start..];
    let next_fn = readiness_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(readiness_fn_body.len());
    let readiness_fn_body = &readiness_fn_body[..next_fn];

    assert!(
        readiness_fn_body.contains("has_received_output"),
        "readiness gate must check has_received_output"
    );
    assert!(
        !readiness_fn_body.contains("was_cold_start"),
        "readiness gate must NOT check was_cold_start — it must be output-based"
    );
}

#[test]
fn open_harness_terminal_calls_readiness_check_before_injection() {
    // The open function must call the output-based readiness check,
    // not rely on was_cold_start for the wait decision.
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("tab_ai_harness_needs_readiness_wait"),
        "open path must use the output-based readiness check method"
    );

    // Ensure wait_for_readiness is passed to inject, not was_cold_start
    let readiness_pos = open_fn_body
        .find("wait_for_readiness")
        .expect("wait_for_readiness must be used in open path");
    let inject_pos = open_fn_body
        .find("inject_tab_ai_harness_submission")
        .expect("must call inject_tab_ai_harness_submission");
    assert!(
        readiness_pos < inject_pos,
        "readiness check must happen before injection call"
    );
}

// =========================================================================
// Zero-intent open: suggested intents + receipt passed to submission
// =========================================================================

#[test]
fn open_harness_terminal_passes_receipt_and_suggestions_to_submission() {
    // The open function must pass invocation_receipt and suggested_intents
    // to build_tab_ai_harness_submission, not just None/&[].
    let open_fn_start = TAB_AI_MODE_SOURCE
        .find("fn open_tab_ai_harness_terminal_from_request(")
        .expect("open_tab_ai_harness_terminal_from_request must exist");
    let open_fn_body = &TAB_AI_MODE_SOURCE[open_fn_start..];
    let next_fn = open_fn_body[1..]
        .find("\n    fn ")
        .unwrap_or(open_fn_body.len());
    let open_fn_body = &open_fn_body[..next_fn];

    assert!(
        open_fn_body.contains("resolved.invocation_receipt"),
        "open path must pass invocation_receipt from resolved context to submission builder"
    );
    assert!(
        open_fn_body.contains("resolved.suggested_intents"),
        "open path must pass suggested_intents from resolved context to submission builder"
    );
}

// =========================================================================
// Deferred capture fields in harness submission text
// =========================================================================

#[test]
fn harness_submission_contains_source_type_screenshot_and_apply_back_hint() {
    use script_kit_gpui::ai::{TabAiApplyBackHint, TabAiSourceType};

    let blob = TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        vec![],
        "2026-03-30T15:30:00Z".to_string(),
    )
    .with_deferred_capture_fields(
        Some(TabAiSourceType::ClipboardEntry),
        Some("/tmp/tab-ai-clip.png".to_string()),
        Some(TabAiApplyBackHint {
            action: "copyToClipboard".to_string(),
            target_label: Some("Clipboard".to_string()),
        }),
    );

    let submission = build_tab_ai_harness_submission(
        &blob,
        Some("Summarize this"),
        TabAiHarnessSubmissionMode::Submit,
        None,
        None,
        &[],
    )
    .expect("submission should build");

    // --- Assert against the rendered flat labeled context ---

    // The submission must start with the flat header
    assert!(
        submission.contains("Script Kit context"),
        "submission must contain the flat context header"
    );

    // sourceType must appear as a flat labeled line
    assert!(
        submission.contains("source type: ClipboardEntry"),
        "rendered submission must contain 'source type: ClipboardEntry'"
    );

    // screenshotPath must appear as a flat labeled line
    assert!(
        submission.contains("screenshot path: /tmp/tab-ai-clip.png"),
        "rendered submission must contain 'screenshot path: /tmp/tab-ai-clip.png'"
    );

    // applyBackHint must appear as labeled lines
    assert!(
        submission.contains("apply back action: copyToClipboard"),
        "rendered submission must contain 'apply back action: copyToClipboard'"
    );
    assert!(
        submission.contains("apply back target: Clipboard"),
        "rendered submission must contain 'apply back target: Clipboard'"
    );

    // The user intent must appear after the context block
    let intent_pos = submission
        .find("User intent:\nSummarize this")
        .expect("user intent must be present");
    let context_pos = submission
        .find("Script Kit context")
        .expect("context header must exist");
    assert!(
        intent_pos > context_pos,
        "user intent must appear after the context block"
    );
}

// =========================================================================
// Quick-submit planner → harness submission regression
// =========================================================================

#[test]
fn submission_includes_quick_submit_hint_and_synthesized_intent() {
    use script_kit_gpui::ai::{plan_tab_ai_quick_submit, TabAiQuickSubmitSource};

    let plan = plan_tab_ai_quick_submit(TabAiQuickSubmitSource::Fallback, "git status")
        .expect("plan should exist for shell command");

    let context = make_context("ScriptList", Some("git status"));
    let submission = build_tab_ai_harness_submission(
        &context,
        Some(&plan.synthesized_intent),
        TabAiHarnessSubmissionMode::Submit,
        Some(&plan),
        None,
        &[],
    )
    .expect("submission should build");

    assert!(
        submission.contains("\"kind\": \"shellCommand\""),
        "hints block must include quick-submit kind as shellCommand"
    );
    assert!(
        submission.contains("Command:\ngit status"),
        "synthesized intent must include the original command"
    );
    assert!(
        submission.contains("User intent:"),
        "submission must include User intent section"
    );
    assert!(
        submission.contains("<scriptKitHints>"),
        "submission must include the hints block with quick-submit metadata"
    );
}

#[test]
fn submission_quick_submit_visual_ask_uses_full_screen_capture() {
    use script_kit_gpui::ai::{plan_tab_ai_quick_submit, TabAiQuickSubmitSource};

    let plan = plan_tab_ai_quick_submit(
        TabAiQuickSubmitSource::Fallback,
        "what's wrong with this UI?",
    )
    .expect("plan should exist for visual ask");

    let context = make_context("ScriptList", None);
    let submission = build_tab_ai_harness_submission(
        &context,
        Some(&plan.synthesized_intent),
        TabAiHarnessSubmissionMode::Submit,
        Some(&plan),
        None,
        &[],
    )
    .expect("submission should build");

    assert!(
        submission.contains("\"kind\": \"visualAsk\""),
        "hints block must include quick-submit kind as visualAsk"
    );
    assert!(
        submission.contains("\"captureKind\": \"fullScreen\""),
        "hints block must include captureKind as fullScreen"
    );
}

#[test]
fn submission_quick_submit_dictation_source_preserved_in_hints() {
    use script_kit_gpui::ai::{plan_tab_ai_quick_submit, TabAiQuickSubmitSource};

    let plan = plan_tab_ai_quick_submit(
        TabAiQuickSubmitSource::Dictation,
        "rewrite this to sound calmer",
    )
    .expect("plan should exist for text transform");

    let context = make_context("ScriptList", None);
    let submission = build_tab_ai_harness_submission(
        &context,
        Some(&plan.synthesized_intent),
        TabAiHarnessSubmissionMode::Submit,
        Some(&plan),
        None,
        &[],
    )
    .expect("submission should build");

    assert!(
        submission.contains("\"source\": \"dictation\""),
        "hints block must preserve the dictation source"
    );
    assert!(
        submission.contains("\"kind\": \"textTransform\""),
        "hints block must include quick-submit kind as textTransform"
    );
    assert!(
        submission.contains("\"captureKind\": \"selectedText\""),
        "text transform must use selectedText capture kind"
    );
}
