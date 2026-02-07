//! Batch 17 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~150 tests validating random built-in action behaviors.
//!
//! Categories:
//! 01. Script context exact action count by ScriptInfo type
//! 02. Scriptlet context copy_content action details
//! 03. Path context total action count and primary action
//! 04. Clipboard paste action description content
//! 05. AI command bar shortcut completeness (which actions lack shortcuts)
//! 06. Notes command bar duplicate_note conditional visibility
//! 07. Note switcher empty notes fallback placeholder
//! 08. Chat context model ID format pattern
//! 09. Scriptlet defined action ID prefix invariant
//! 10. Agent context reveal_in_finder and copy_path shortcuts
//! 11. File context exact description strings
//! 12. Path context exact description strings
//! 13. Clipboard text/image macOS action count difference
//! 14. Script run title format includes quotes
//! 15. to_deeplink_name with whitespace variations
//! 16. Action::new pre-computes lowercase fields
//! 17. ActionsDialog::format_shortcut_hint SDK-style shortcuts
//! 18. ActionsDialog::parse_shortcut_keycaps compound shortcuts
//! 19. ActionsDialog::score_action multi-field bonus stacking
//! 20. ActionsDialog::fuzzy_match character ordering requirement
//! 21. build_grouped_items_static with no-section actions
//! 22. coerce_action_selection alternating header-item pattern
//! 23. CommandBarConfig notes_style preset values
//! 24. Clipboard unpin action title and description text
//! 25. New chat actions mixed section sizes
//! 26. Notes command bar browse_notes action details
//! 27. Script context copy_deeplink description contains URL
//! 28. Cross-context all actions are ScriptContext category
//! 29. Action with_icon chaining preserves all fields
//! 30. Script context action stability across flag combinations

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_17/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_17/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_17/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_17/tests_part_04.rs");
}
