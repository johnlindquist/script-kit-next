//! Batch 8: Dialog builtin action validation tests
//!
//! Focuses on combined scenarios, interaction matrices, and boundary conditions
//! not covered in batches 1-7:
//!
//! 1. Action verb in primary title format ("{verb} \"{name}\"") across all verbs
//! 2. Clipboard pin×contentType×app combined matrix
//! 3. Scriptlet context custom actions with frecency interaction
//! 4. Path context special characters in names
//! 5. Note switcher Unicode/emoji titles
//! 6. Chat context partial state combinations
//! 7. AI command bar description keyword validation
//! 8. Notes command bar section label transitions
//! 9. New chat duplicate providers
//! 10. Deeplink description URL format validation
//! 11. Score_action boundary thresholds (exact 100/50/25 boundaries)
//! 12. build_grouped_items interleaved section/no-section
//! 13. coerce_action_selection complex patterns
//! 14. parse_shortcut_keycaps compound symbol sequences
//! 15. CommandBarConfig notes_style detailed fields
//! 16. Cross-builder action count comparisons
//! 17. Action builder chaining order independence
//! 18. Clipboard destructive action ordering stability
//! 19. File context title includes exact filename
//! 20. Notes info all-true/all-false edge cases
//! 21. ScriptInfo agent flag interactions with frecency chaining
//! 22. Agent actions: no view_logs, has copy_content
//! 23. Builtin with full optional fields (shortcut+alias+frecency)
//! 24. Path context dir vs file action count equality
//! 25. Multiple scriptlet custom actions ordering
//! 26. Chat model checkmark exact match only
//! 27. Note switcher empty/placeholder title
//! 28. Action with_section/with_icon chaining order independence
//! 29. Clipboard delete_multiple description content
//! 30. Deeplink name consecutive special characters

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_8/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_8/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_8/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_8/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_8/tests_part_05.rs");
    include!("dialog_builtin_action_validation_tests_8/tests_part_06.rs");
}
