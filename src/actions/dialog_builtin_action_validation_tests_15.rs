//! Batch 15 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~170 tests covering fresh angles:
//! - to_deeplink_name with non-Latin Unicode (Arabic, Thai, Devanagari)
//! - Clipboard image macOS-exclusive actions
//! - Notes combined-flag interactions
//! - Chat context boundary states
//! - New chat section guarantees
//! - Note switcher description fallback hierarchy
//! - AI command bar per-section ID enumeration
//! - Action builder overwrite semantics
//! - CommandBarConfig preset field comparison matrix
//! - Cross-context category uniformity
//! - Clipboard exact action counts on macOS
//! - Path primary-action insertion position
//! - File title quoting
//! - ScriptInfo::with_all field completeness
//! - Ordering idempotency (double-call determinism)

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_15/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_15/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_15/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_15/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_15/tests_part_05.rs");
}
