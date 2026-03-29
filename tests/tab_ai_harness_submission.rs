//! Integration tests for Tab AI harness submission modes.
//!
//! Validates that `build_tab_ai_harness_submission` correctly distinguishes
//! between PasteOnly (stage context) and Submit (immediate turn) modes,
//! and that the harness session lifecycle supports warm reuse.

use script_kit_gpui::ai::{
    build_tab_ai_harness_submission, validate_tab_ai_harness_config, HarnessConfig,
    TabAiContextBlob, TabAiHarnessSubmissionMode, TabAiUiSnapshot,
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
    let submission =
        build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::PasteOnly)
            .expect("submission should build");

    assert!(submission.contains("<scriptKitContext schemaVersion=\"1\">"));
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
    let submission =
        build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::PasteOnly)
            .expect("should build");

    assert!(
        submission.starts_with("<scriptKitContext schemaVersion=\"1\">"),
        "staged context must start with the schema-versioned XML block"
    );
    assert!(
        submission.contains("</scriptKitContext>"),
        "staged context must close the XML block"
    );
    assert!(
        submission.contains("\"promptType\": \"ScriptList\""),
        "staged context must include the UI snapshot prompt type"
    );
}

// =========================================================================
// Submit mode: full turn behavior
// =========================================================================

#[test]
fn submit_without_intent_includes_sentinel() {
    let context = make_context("ScriptList", None);
    let submission =
        build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::Submit)
            .expect("submission should build");

    assert!(submission.contains("<scriptKitContext schemaVersion=\"1\">"));
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
    )
    .expect("should build");
    assert!(!paste.contains("User intent:"));
    assert!(!paste.contains("Await the user"));

    let submit =
        build_tab_ai_harness_submission(&context, Some("   "), TabAiHarnessSubmissionMode::Submit)
            .expect("should build");
    assert!(!submit.contains("User intent:"));
    assert!(submit.contains("Await the user's next terminal input."));
}

#[test]
fn empty_string_intent_treated_as_none() {
    let context = make_context("ScriptList", None);
    let paste =
        build_tab_ai_harness_submission(&context, Some(""), TabAiHarnessSubmissionMode::PasteOnly)
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
    let submission =
        build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::PasteOnly)
            .expect("should build");

    assert!(
        submission.contains("my-query"),
        "submission must include the input text from the context blob"
    );
}

#[test]
fn submission_includes_prompt_type_from_context() {
    let context = make_context("ClipboardHistory", None);
    let submission =
        build_tab_ai_harness_submission(&context, None, TabAiHarnessSubmissionMode::PasteOnly)
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
        .find("fn open_tab_ai_harness_terminal(")
        .expect("open_tab_ai_harness_terminal must exist");
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
