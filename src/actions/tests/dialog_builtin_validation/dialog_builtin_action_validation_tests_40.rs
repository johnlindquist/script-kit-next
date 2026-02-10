// Batch 40: Dialog builtin action validation tests
//
// Focuses on:
// - ScriptInfo::with_action_verb_and_shortcut: field validation
// - ScriptInfo: is_agent manual override after construction
// - Action::with_shortcut_opt: Some vs None behavior
// - Action::with_icon and with_section chaining
// - Clipboard: text entry total action count on macOS
// - Clipboard: image entry total action count on macOS
// - Clipboard: pinned text entry has clipboard_unpin ID
// - Clipboard: unpinned text entry has clipboard_pin ID
// - File context: dir has no quick_look but has open_with on macOS
// - File context: file primary title format uses quoted name
// - Path context: all action IDs are snake_case
// - Path context: open_in_editor desc mentions $EDITOR
// - Script context: scriptlet is_scriptlet true has edit_scriptlet
// - Script context: builtin has exactly 4 actions when no shortcut/alias
// - Script context: primary title uses action_verb
// - Scriptlet context: with_custom run_script is first action
// - Scriptlet context: with_custom copy_deeplink URL uses to_deeplink_name
// - AI bar: toggle_shortcuts_help details
// - AI bar: change_model has no shortcut
// - AI bar: unique IDs across all 12 actions
// - Notes: export action requires selection and not trash
// - Notes: browse_notes always present
// - Chat context: copy_response only when has_response
// - Chat context: clear_conversation only when has_messages
// - New chat: empty lists produce zero actions
// - New chat: preset IDs use preset_{id} format
// - Note switcher: empty notes produces no_notes action
// - Note switcher: preview with relative_time has separator
// - ProtocolAction: with_value sets value and has_action false
// - format_shortcut_hint: dialog vs builders produce different results

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_40/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_40/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_40/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_40/tests_part_04.rs");
}
