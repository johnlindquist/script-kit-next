//! Batch 7: Dialog builtin action validation tests
//!
//! Focuses on novel edge cases and cross-cutting invariants not covered in batches 1-6:
//!
//! 1. format_shortcut_hint (dialog.rs version) edge cases: unknown keys, single modifier,
//!    double-plus, empty string, mixed-case modifiers
//! 2. score_action with Unicode: diacritics, CJK, emoji in title/desc
//! 3. Note switcher description rendering boundary: exactly 60 chars, 59, 61, 0
//! 4. Clipboard combined flag matrix: pinned×image, pinned×text, unpinned×image, unpinned×text
//! 5. Chat context model ID generation format consistency
//! 6. Notes command bar icon presence for every action
//! 7. New chat action ordering within each section
//! 8. Agent actions exclude view_logs
//! 9. Script vs scriptlet action set symmetric difference
//! 10. Deeplink URL in description format
//! 11. AI command bar shortcut uniqueness
//! 12. Notes command bar shortcut uniqueness
//! 13. Path context action ordering: primary first, trash last
//! 14. Clipboard action shortcut format (all use symbol notation)
//! 15. Score_action with whitespace-only query
//! 16. fuzzy_match with repeated characters
//! 17. build_grouped_items_static with single-item single-section
//! 18. coerce_action_selection with alternating header-item pattern
//! 19. parse_shortcut_keycaps with empty string and multi-byte
//! 20. CommandBarConfig close flags independence
//! 21. Action constructor with empty strings
//! 22. ScriptInfo scriptlet flag exclusivity with agent
//! 23. Notes command bar action count bounds per flag state
//! 24. Chat model display_name in title
//! 25. New chat model_id in action ID
//! 26. Clipboard delete_all description mentions "pinned"
//! 27. File context all actions have ScriptContext category
//! 28. Path context copy_path and copy_filename always present
//! 29. Cross-context ID namespace separation
//! 30. Action title_lower invariant across all builder functions

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_7/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_7/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_7/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_7/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_7/tests_part_05.rs");
    include!("dialog_builtin_action_validation_tests_7/tests_part_06.rs");
}
