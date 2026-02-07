//! Batch 5: Built-in action behavioral validation tests
//!
//! 150+ tests validating action invariants NOT covered in batches 1-4.
//! Focus areas:
//! - Note switcher description rendering (preview truncation, relative time combos)
//! - Clipboard action position invariants beyond first/last
//! - AI command bar section item counts
//! - build_grouped_items_static section transitions
//! - Large-scale stress (many notes, models, presets)
//! - Cross-function ScriptInfo consistency
//! - Action description content keywords
//! - Score_action with cached lowercase fields
//! - Scriptlet with_custom multiple custom actions ordering
//! - CommandBarConfig equality and field access patterns

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_5/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_5/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_5/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_5/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_5/tests_part_05.rs");
}
