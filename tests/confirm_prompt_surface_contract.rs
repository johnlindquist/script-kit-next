const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");
const ELEMENTS_SOURCE: &str = include_str!("../src/app_layout/collect_elements.rs");
const STDIN_SOURCE: &str = include_str!("../src/stdin_commands/mod.rs");
const UI_WINDOW_SOURCE: &str = include_str!("../src/app_impl/ui_window.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

#[test]
fn open_confirm_prompt_stdin_fixture_is_registered() {
    assert!(
        STDIN_SOURCE.contains("OpenConfirmPrompt")
            && STDIN_SOURCE.contains("\"openConfirmPrompt\""),
        "openConfirmPrompt should be registered as a deterministic DevTools fixture opener"
    );
    assert!(
        STDIN_SOURCE.contains("test_external_command_open_confirm_prompt_deserialization"),
        "openConfirmPrompt should have a parser contract"
    );
}

#[test]
fn confirm_prompt_uses_standard_height_window_sizing() {
    let sizing_arm = source_between(
        UI_WINDOW_SOURCE,
        "AppView::ConfirmPrompt { .. } =>",
        "AppView::MiniPrompt",
    );
    assert!(
        sizing_arm.contains("ViewType::DivPrompt"),
        "ConfirmPrompt renders a STANDARD_HEIGHT shell and should size as a DivPrompt container"
    );
}

#[test]
fn confirm_prompt_layout_exposes_content_stack_footer_and_buttons() {
    let branch = source_between(
        LAYOUT_SOURCE,
        "if matches!(self.current_view, AppView::ConfirmPrompt",
        "// Header",
    );

    for component in [
        "ConfirmPromptContent",
        "ConfirmPromptStack",
        "ConfirmPromptTitle",
        "ConfirmPromptBody",
        "ConfirmPromptFooter",
        "ConfirmPromptConfirmButton",
        "ConfirmPromptCancelButton",
    ] {
        assert!(
            branch.contains(component),
            "getLayoutInfo must expose {component} for ConfirmPrompt proof"
        );
    }

    for token in [
        "LIQUID_GLASS_WINDOW_RADIUS_PX",
        "LIQUID_GLASS_PANEL_RADIUS_PX",
        "LIQUID_GLASS_COMPACT_RADIUS_PX",
    ] {
        assert!(
            branch.contains(token) || LAYOUT_SOURCE.contains(token),
            "ConfirmPrompt layout receipts must include {token}"
        );
    }
}

#[test]
fn confirm_prompt_elements_expose_footer_actions() {
    let branch = source_between(
        ELEMENTS_SOURCE,
        "AppView::ConfirmPrompt",
        "AppView::WebcamView",
    );
    assert!(
        branch.contains("confirm-prompt")
            && branch.contains("confirm")
            && branch.contains("cancel")
            && branch.contains("selectable = Some(true)"),
        "ConfirmPrompt getElements should expose panel plus confirm/cancel footer buttons"
    );
}
