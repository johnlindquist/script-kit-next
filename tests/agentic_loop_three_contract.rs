//! Source-level contract for third-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const TARGET_THREAD: &str = include_str!("../scripts/agentic/target-thread.ts");

#[test]
fn index_help_exposes_loop_three_recipes() {
    for name in [
        "template-prompt-automation-parity-stress",
        "current-app-commands-frontmost-stress",
        "actions-captured-subject-frame-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
    }
}

#[test]
fn template_prompt_stress_pins_state_first_runtime_receipts() {
    for token in [
        "template-prompt-automation-parity-stress",
        "templatePrompt",
        "input:template-source",
        "activePopupContract",
        "usedGetState: true",
        "usedGetElements: true",
        "usedBatch: true",
        "usedSimulateKey: true",
        "batch.forceSubmit",
        "template_prompt_force_submit_failed",
        "TemplatePrompt",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "TemplatePrompt stress receipt must pin {token}"
        );
    }
}

#[test]
fn current_app_commands_stress_pins_frontmost_and_shared_filtering() {
    for token in [
        "current-app-commands-frontmost-stress",
        "missing_current_app_commands_frontmost_receipt",
        "currentAppCommands",
        "builtin/do-in-current-app",
        "Do in Current Command",
        "current_app_commands_filtered_entries",
        "frontmostSnapshot",
        "wrongAppExecutionBlocked",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Current App Commands stress receipt must pin {token}"
        );
    }
}

#[test]
fn actions_captured_subject_stress_pins_subject_frame_and_focus_restore() {
    for token in [
        "actions-captured-subject-frame-stress",
        "missing_actions_captured_subject_receipt",
        "actionsCapturedSubject",
        "subjectStableKey",
        "pendingSubjectFrame",
        "executeSubjectStableKey",
        "reReadCurrentSelection",
        "focusRestoredTo",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Actions captured-subject stress receipt must pin {token}"
        );
    }
}
