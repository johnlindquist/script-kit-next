// Batch 41: Dialog builtin action validation tests
//
// Focuses on:
// - fuzzy_match: empty needle and empty haystack behavior
// - fuzzy_match: subsequence order enforcement
// - score_action: description-only match yields exactly 15
// - score_action: shortcut-only match yields exactly 10
// - score_action: combined prefix + description + shortcut max score
// - builders format_shortcut_hint: simpler .replace chain vs dialog
// - builders format_shortcut_hint: unknown keys pass through
// - parse_shortcut_keycaps: all modifier symbols individually
// - parse_shortcut_keycaps: empty string produces empty vec
// - Clipboard: share action details (shortcut, title, position)
// - Clipboard: attach_to_ai action details
// - Clipboard: image open_with is macOS only
// - File context: primary action ID differs file vs dir
// - File context: all IDs unique within context
// - Path context: open_in_terminal shortcut and desc
// - Path context: move_to_trash desc differs file vs dir
// - Script context: with shortcut yields update_shortcut + remove_shortcut
// - Script context: with alias yields update_alias + remove_alias
// - Script context: agent has edit_script with "Edit Agent" title, desc mentions agent
// - Script context: total action count varies by type
// - Scriptlet context: add_shortcut when no shortcut, add_alias when no alias
// - Scriptlet context: reset_ranking only when is_suggested
// - AI bar: delete_chat shortcut and icon
// - AI bar: new_chat shortcut and icon
// - Notes: format action details
// - Notes: selection+trash yields subset of actions
// - Chat context: model with current_model gets checkmark
// - Chat context: multiple models ordering
// - New chat: section ordering across last_used, presets, models
// - count_section_headers: items without sections produce 0 headers

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_41/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_41/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_41/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_41/tests_part_04.rs");
}
