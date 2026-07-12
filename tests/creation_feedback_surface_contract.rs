const BUTTON_TYPES_SOURCE: &str = include_str!("../src/components/button/types.rs");
const LAYOUT_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");
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
fn shared_button_radius_uses_liquid_glass_compact_token() {
    assert!(
        BUTTON_TYPES_SOURCE
            .contains("BUTTON_RADIUS_PX: f32 = crate::ui::chrome::LIQUID_GLASS_COMPACT_RADIUS_PX"),
        "shared buttons should use the compact Liquid Glass radius token"
    );
    assert!(
        BUTTON_TYPES_SOURCE.contains("BUTTON_BORDER_WIDTH_PX: f32 = 1.0"),
        "shared buttons should use a subtle 1px Liquid Glass border"
    );
}

#[test]
fn creation_feedback_uses_standard_height_window_sizing() {
    let sizing_arm = source_between(
        UI_WINDOW_SOURCE,
        "AppView::CreationFeedback { .. } =>",
        "AppView::ScriptIssuesView",
    );
    assert!(
        sizing_arm.contains("ViewType::DivPrompt"),
        "CreationFeedback renders a STANDARD_HEIGHT shell and should size as a DivPrompt container"
    );
}

#[test]
fn creation_feedback_layout_exposes_window_panel_path_and_action_geometry() {
    let branch = source_between(
        LAYOUT_SOURCE,
        "if matches!(self.current_view, AppView::CreationFeedback",
        "if matches!(self.current_view, AppView::ConfirmPrompt",
    );

    for component in [
        "CreationFeedbackPanel",
        "CreationFeedbackArtifactSection",
        "CreationFeedbackArtifactPathSurface",
        "CreationFeedbackVerificationSection",
        "CreationFeedbackVerificationStatusSurface",
        "CreationFeedbackReceiptSection",
        "CreationFeedbackReceiptPathSurface",
        "CreationFeedbackReceiptStatusSurface",
        "CreationFeedbackArtifactActions",
        "CreationFeedbackReceiptActions",
        "CreationFeedbackRevealButton",
        "CreationFeedbackCopyButton",
        "CreationFeedbackEditButton",
        "CreationFeedbackRunButton",
        "CreationFeedbackCopyReceiptButton",
        "CreationFeedbackOpenReceiptButton",
    ] {
        assert!(
            branch.contains(component),
            "getLayoutInfo must expose {component} for Feedback proof"
        );
    }

    for token in [
        "LIQUID_GLASS_WINDOW_RADIUS_PX",
        "LIQUID_GLASS_PANEL_RADIUS_PX",
        "LIQUID_GLASS_CONTROL_RADIUS_PX",
        "LIQUID_GLASS_COMPACT_RADIUS_PX",
    ] {
        assert!(
            branch.contains(token) || LAYOUT_SOURCE.contains(token),
            "Feedback layout receipts must include {token}"
        );
    }
    assert!(
        branch.contains("let panel_y = content_top + FEEDBACK_PADDING_Y;")
            && branch.contains(".with_parent(\"MainViewMain\")"),
        "CreationFeedback detail receipts should live below and inside MainViewMain"
    );
}

#[test]
fn open_creation_feedback_stdin_fixture_is_registered() {
    assert!(
        STDIN_SOURCE.contains("OpenCreationFeedback")
            && STDIN_SOURCE.contains("\"openCreationFeedback\""),
        "openCreationFeedback should be registered as a deterministic DevTools fixture opener"
    );
    assert!(
        STDIN_SOURCE.contains("test_external_command_open_creation_feedback_deserialization"),
        "openCreationFeedback should have a parser contract"
    );
    assert!(
        STDIN_SOURCE.contains("receiptPath")
            && STDIN_SOURCE.contains("receiptStatus")
            && STDIN_SOURCE.contains("verificationStatus"),
        "openCreationFeedback should expose receipt and verification fixture fields"
    );
}
