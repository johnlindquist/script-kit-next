//! Source-level contract for TemplatePrompt automation and actions parity.

const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_SIMULATE_KEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");

fn template_prompt_simulate_key_arm(source: &str) -> &str {
    let start = source
        .find("AppView::TemplatePrompt { entity, id, .. }")
        .expect("TemplatePrompt simulateKey arm should exist");
    let after = &source[start..];
    let end = after
        .find("AppView::ChatPrompt")
        .expect("TemplatePrompt arm should be before ChatPrompt");
    &after[..end]
}

fn force_submit_branch<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .expect("forceSubmit branch should exist");
    &source[start..start + 1_800.min(source.len() - start)]
}

#[test]
fn template_prompt_has_explicit_simulate_key_arm_in_live_and_mirror_dispatchers() {
    // doc-anchor-removed: [[protocol#Query and introspection#TemplatePrompt automation receipts]]
    for source in [
        APP_RUN_SETUP,
        RUNTIME_STDIN,
        RUNTIME_STDIN_MATCH_SIMULATE_KEY,
    ] {
        let arm = template_prompt_simulate_key_arm(source);

        assert!(arm.contains("SimulateKey: Dispatching"));
        assert!(arm.contains("TemplatePrompt"));
        assert!(arm.contains("prompt.submit(cx)"));
        assert!(arm.contains("prompt.next_input(cx)"));
        assert!(arm.contains("prompt.prev_input(cx)"));
        assert!(arm.contains("prompt.handle_backspace(cx)"));
        assert!(arm.contains("prompt.handle_char(ch, cx)"));
        assert!(arm.contains("submit_prompt_response"));
        assert!(arm.contains("cancel_script_execution(ctx)"));
        assert!(arm.contains("dispatch_actions_toggle_for_current_view"));
        assert!(arm.contains("stdin_simulate_key_template_prompt"));
    }
}

#[test]
fn template_prompt_force_submit_is_supported_in_direct_and_batch_paths() {
    let direct = force_submit_branch(PROMPT_HANDLER, "PromptMessage::ForceSubmit");
    let batch = force_submit_branch(PROMPT_HANDLER, "protocol::BatchCommand::ForceSubmit");

    assert!(
        direct.contains("AppView::TemplatePrompt { id, .. } => Some(id.clone())"),
        "direct forceSubmit should support TemplatePrompt"
    );
    assert!(
        batch.contains("AppView::TemplatePrompt { id, .. } => Some(id.clone())"),
        "batch forceSubmit should support TemplatePrompt"
    );
}

#[test]
fn template_prompt_footer_actions_has_live_host_coverage() {
    let footer_start = UI_WINDOW
        .find("if matches!(self.current_view, AppView::TemplatePrompt")
        .expect("TemplatePrompt footer branch should exist");
    let footer = &UI_WINDOW[footer_start..footer_start + 1_600.min(UI_WINDOW.len() - footer_start)];

    assert!(footer.contains("FooterAction::Actions"));
    assert!(ACTIONS_DIALOG
        .contains("AppView::TemplatePrompt { .. } => Some(ActionsDialogHost::TemplatePrompt)"));
    assert!(ACTIONS_DIALOG
        .contains("ActionsDialogHost::TemplatePrompt => FocusRequest::template_prompt()"));
    assert!(ACTIONS_DIALOG.contains("| ActionsDialogHost::TemplatePrompt"));
    assert!(ACTIONS_TOGGLE.contains("ActionsDialogHost::TemplatePrompt => \"TemplatePrompt\""));
    assert!(ACTIONS_TOGGLE.contains("AppView::TemplatePrompt { .. }"));
    assert!(APP_RUN_SETUP.contains("Some(\"templatePrompt\") =>"));
    assert!(APP_RUN_SETUP.contains("Some(ActionsDialogHost::TemplatePrompt)"));
}
