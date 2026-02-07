//! Batch 38: Dialog builtin action validation tests
//!
//! Focuses on constructor variations, under-tested builder paths, and integration edges:
//! - ScriptInfo constructor variants (with_action_verb, with_all, with_is_script)
//! - Clipboard save_snippet, save_file, and frontmost_app_name dynamic title
//! - Notes duplicate_note, find_in_note, copy_note_as details
//! - AI command bar export_markdown and submit action specifics
//! - Chat context empty models and model ID format
//! - New chat last_used icon BoltFilled, model icon Settings
//! - Note switcher current note "• " prefix and preview trimming
//! - count_section_headers edge cases
//! - WindowPosition enum variants and defaults
//! - ProtocolAction::with_value constructor
//! - score_action with whitespace and special character searches
//! - Cross-context description keyword validation

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_38/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_38/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_38/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_38/tests_part_04.rs");
}
