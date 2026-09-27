//! Source-level contract for TemplatePrompt automation and actions parity.

const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/app_impl/actions_dialog.rs");
const ACTIONS_TOGGLE: &str = include_str!("../src/app_impl/actions_toggle.rs");
const UI_WINDOW: &str = include_str!("../src/app_impl/ui_window.rs");

fn force_submit_branch<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .expect("forceSubmit branch should exist");
    &source[start..start + 1_800.min(source.len() - start)]
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
