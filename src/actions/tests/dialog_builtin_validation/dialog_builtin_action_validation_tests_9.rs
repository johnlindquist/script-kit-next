// Batch 9: Dialog builtin action validation tests
//
// Focuses on areas not exhaustively covered in batches 1-8:
//
// 1. AI command bar expanded actions (branch_from_last, export_markdown, toggle_shortcuts_help)
// 2. File context macOS-specific actions (quick_look, open_with, show_info)
// 3. Clipboard macOS-specific actions (quick_look, open_with, annotate/upload)
// 4. CommandBarConfig struct field validation
// 5. format_shortcut_hint alias coverage (meta, super, opt, esc, return, arrowdown/left/right)
// 6. Cross-context shortcut symbol consistency (all shortcuts use symbol chars)
// 7. Action verb formatting with special characters in names
// 8. Notes command bar conditional section groups (section strings)
// 9. ScriptInfo mixed agent+scriptlet flag precedence
// 10. Clipboard save_snippet/save_file always present for both text and image
// 11. AI command bar section completeness (12 actions across 6 sections)
// 12. Path context open_in_finder/editor/terminal descriptions
// 13. Note switcher empty notes placeholder action
// 14. New chat action icon/section consistency
// 15. Score_action with multi-word queries
// 16. build_grouped_items_static with Headers style multi-section
// 17. coerce_action_selection with single-item rows
// 18. parse_shortcut_keycaps with multi-char sequences
// 19. Deeplink name with Unicode characters preservation
// 20. File context open title includes quoted name
// 21. Clipboard share/attach_to_ai always present
// 22. Notes command bar icon name validation
// 23. Chat context continue_in_chat always present
// 24. Scriptlet context copy_content always present
// 25. Agent actions: has copy_content, edit title says "Edit Agent"
// 26. Cross-context action count stability
// 27. Action with_shortcut_opt preserves existing fields
// 28. ActionsDialogConfig default values
// 29. SectionStyle and SearchPosition enum values
// 30. Clipboard action IDs all prefixed with "clipboard_"

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_9/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_9/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_9/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_9/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_9/tests_part_05.rs");
}
