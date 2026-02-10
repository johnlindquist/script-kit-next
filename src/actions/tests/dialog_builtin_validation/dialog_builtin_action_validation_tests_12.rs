//! Batch 12: Dialog Built-in Action Validation Tests
//!
//! 150+ tests across 30 categories validating random behaviors of
//! built-in action window dialogs.
//!
//! Focus areas for this batch:
//! - CommandBarConfig preset field validation (all presets, all fields)
//! - Clipboard text vs image action count delta and exact IDs
//! - Scriptlet context with_custom shortcut/alias dynamic action switching
//! - Agent context description keyword content
//! - Note switcher multi-note sorting and section assignment
//! - AI command bar section-to-action-count mapping
//! - Path context description substring matching
//! - Chat context multi-model ID generation patterns
//! - Score_action stacking with multi-field matches
//! - build_grouped_items_static with alternating sections
//! - Cross-context action description non-empty invariant
//! - format_shortcut_hint chaining (multiple modifiers in sequence)
//! - Action builder with_icon and with_section field isolation
//! - ScriptInfo with_is_script constructor
//! - Deeplink name with mixed Unicode scripts
//! - parse_shortcut_keycaps special symbol recognition
//! - Clipboard destructive action shortcut exact values
//! - File context macOS action count (file vs dir)
//! - New chat action ID prefix patterns
//! - Notes command bar section-to-action mapping

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_12/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_12/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_12/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_12/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_12/tests_part_05.rs");
}
