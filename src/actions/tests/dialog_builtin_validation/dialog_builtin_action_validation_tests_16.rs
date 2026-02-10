//! Batch 16 – Dialog Builtin Action Validation Tests
//!
//! 30 categories, ~155 tests covering fresh angles:
//! - Scriptlet context with custom actions ordering relative to built-ins
//! - Clipboard share/attach_to_ai universality across content types
//! - Agent context copy_content description substring
//! - Path context open_in_terminal/open_in_editor shortcuts
//! - File context shortcut consistency between file and directory
//! - Notes command bar section count per flag combo
//! - Chat context continue_in_chat shortcut value
//! - AI command bar export section single action
//! - New chat preset icon propagation
//! - Note switcher pinned+current combined state
//! - format_shortcut_hint modifier keyword normalization edge cases
//! - to_deeplink_name numeric and underscore handling
//! - score_action empty query behaviour
//! - fuzzy_match case sensitivity
//! - build_grouped_items_static single-item input
//! - coerce_action_selection single-item input
//! - CommandBarConfig close flag independence
//! - Action::new description_lower None when description is None
//! - Action builder chain ordering (icon before section, section before shortcut)
//! - ScriptInfo with_action_verb preserves defaults
//! - Script context agent flag produces edit_script with "Edit Agent" title
//! - Cross-context shortcut format consistency (all use Unicode symbols)
//! - Clipboard paste_keep_open shortcut value
//! - Path context copy_filename has no shortcut
//! - File context open_with macOS shortcut
//! - Notes format shortcut exact value
//! - AI command bar icon name correctness
//! - Script context run title format
//! - Ordering consistency across repeated calls

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_16/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_16/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_16/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_16/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_16/tests_part_05.rs");
}
