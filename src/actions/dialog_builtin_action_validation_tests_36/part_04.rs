
#[test]
fn coerce_selection_beyond_bounds_clamped() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![GroupedActionItem::Item(0), GroupedActionItem::Item(1)];
    // Index 100 should clamp to last = 1
    assert_eq!(coerce_action_selection(&rows, 100), Some(1));
}

#[test]
fn coerce_selection_header_then_item() {
    use crate::actions::dialog::GroupedActionItem;
    let rows = vec![
        GroupedActionItem::SectionHeader("S".into()),
        GroupedActionItem::Item(0),
    ];
    // Landing on header at 0, search down → finds Item at 1
    assert_eq!(coerce_action_selection(&rows, 0), Some(1));
}

// =====================================================================
// 29. score_action: combined bonuses max scenario
// =====================================================================

#[test]
fn score_action_prefix_plus_desc_plus_shortcut() {
    let action = Action::new(
        "edit",
        "Edit Script",
        Some("Edit the script file".into()),
        ActionCategory::ScriptContext,
    )
    .with_shortcut("⌘E");
    let score = ActionsDialog::score_action(&action, "edit");
    // prefix(100) + desc contains "edit"(15) + shortcut probably no match = 115
    assert!(score >= 115, "Expected ≥115, got {}", score);
}

#[test]
fn score_action_contains_only() {
    let action = Action::new(
        "copy_edit",
        "Copy Edit Path",
        None,
        ActionCategory::ScriptContext,
    );
    let score = ActionsDialog::score_action(&action, "edit");
    assert!(score >= 50 && score < 100, "Expected 50-99, got {}", score);
}

#[test]
fn score_action_no_match() {
    let action = Action::new("test", "Hello World", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "xyz");
    assert_eq!(score, 0);
}

#[test]
fn score_action_empty_search_prefix_match() {
    let action = Action::new("test", "Anything", None, ActionCategory::ScriptContext);
    let score = ActionsDialog::score_action(&action, "");
    // Empty string is prefix of everything
    assert!(score >= 100, "Expected ≥100, got {}", score);
}

// =====================================================================
// 30. Cross-context: ProtocolAction close/visibility defaults and SDK action ID format
// =====================================================================

#[test]
fn protocol_action_sdk_id_matches_name() {
    // SDK actions use name as ID
    let pa = ProtocolAction {
        name: "My Custom Action".into(),
        description: Some("desc".into()),
        shortcut: None,
        value: Some("val".into()),
        has_action: true,
        visible: None,
        close: None,
    };
    // Simulate conversion (as done in set_sdk_actions)
    let action = Action::new(
        pa.name.clone(),
        pa.name.clone(),
        pa.description.clone(),
        ActionCategory::ScriptContext,
    );
    assert_eq!(action.id, "My Custom Action");
}

#[test]
fn protocol_action_shortcut_converted_via_format() {
    let formatted = ActionsDialog::format_shortcut_hint("cmd+shift+c");
    assert_eq!(formatted, "⌘⇧C");
}

#[test]
fn protocol_action_sdk_icon_is_none() {
    // SDK actions don't currently have icons
    let action = Action::new(
        "sdk_action",
        "SDK Action",
        None,
        ActionCategory::ScriptContext,
    );
    assert!(action.icon.is_none());
}

#[test]
fn protocol_action_sdk_section_is_none() {
    // SDK actions don't currently have sections
    let action = Action::new(
        "sdk_action",
        "SDK Action",
        None,
        ActionCategory::ScriptContext,
    );
    assert!(action.section.is_none());
}
