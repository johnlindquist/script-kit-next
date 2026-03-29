//! Integration tests for Tab AI harness submission modes.
//!
//! Validates that `build_tab_ai_harness_submission` correctly distinguishes
//! between PasteOnly (stage context) and Submit (immediate turn) modes.

use script_kit_gpui::ai::{
    build_tab_ai_harness_submission, TabAiContextBlob, TabAiHarnessSubmissionMode, TabAiUiSnapshot,
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
