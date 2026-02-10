// =============================================================================
// Dialog Built-in Action Validation Tests — Batch 18
//
// 30 categories of tests validating random built-in actions from dialog windows.
// Each category tests a specific behavior, field, or invariant.
//
// Run with:
//   cargo test --lib actions::dialog_builtin_action_validation_tests_18
// =============================================================================

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_18/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_18/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_18/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_18/tests_part_04.rs");
}
