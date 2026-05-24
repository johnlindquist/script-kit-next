const DOUBLE_COMMAND: &str =
    include_str!("../../src/platform/accessibility/double_modifier_trigger.rs");

#[test]
fn double_command_state_machine_ignores_combined_shortcuts() {
    assert!(DOUBLE_COMMAND.contains("DoubleCommandState"));
    assert!(DOUBLE_COMMAND.contains("CombinedShortcut"));
    assert!(DOUBLE_COMMAND.contains("NonModifierKey"));
    assert!(DOUBLE_COMMAND.contains("DoubleCommandOutcome::Trigger"));
}
