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
        "target: { type: \"kind\", kind: \"actionsDialog\", index: 0 }",
        "template_prompt_actions_unavailable",
        "panel_only_actions_dialog",
        "semanticId.startsWith(\"action:\") || row.semanticId.startsWith(\"choice:\")",
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
fn template_prompt_stress_opens_template_without_wrong_parse_wait() {
    let send_fn_start = SCENARIO
        .find("const send = async (payload: Record<string, unknown>, name: string)")
        .expect("TemplatePrompt stress must keep a local send helper");
    let send_fn = &SCENARIO[send_fn_start
        ..SCENARIO[send_fn_start..]
            .find("const start = await runTool")
            .map(|offset| send_fn_start + offset)
            .expect("TemplatePrompt send helper must end before session start")];

    for token in [
        "const shouldAwaitParse = payload.type !== \"template\"",
        "if (shouldAwaitParse)",
        "\"--await-parse\"",
    ] {
        assert!(
            send_fn.contains(token),
            "TemplatePrompt send helper must gate parse waits with {token}"
        );
    }
}

#[test]
fn actions_dialog_automation_elements_use_cached_semantic_rows() {
    const COLLECTOR: &str = include_str!("../src/windows/automation_surface_collector.rs");
    const ACTION_WINDOW: &str = include_str!("../src/actions/window.rs");

    for token in [
        "actions_dialog_semantic_cache",
        "upsert_actions_dialog_snapshot",
        "remove_actions_dialog_snapshot",
        "collect_cached_actions_dialog_snapshot(&resolved.id)",
    ] {
        assert!(
            COLLECTOR.contains(token),
            "ActionsDialog automation collector must pin cached semantic rows with {token}"
        );
    }

    for token in [
        "upsert_actions_dialog_snapshot(",
        "remove_actions_dialog_snapshot(\"actions-dialog\")",
    ] {
        assert!(
            ACTION_WINDOW.contains(token),
            "ActionsDialog window lifecycle must publish/remove semantic snapshots with {token}"
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
