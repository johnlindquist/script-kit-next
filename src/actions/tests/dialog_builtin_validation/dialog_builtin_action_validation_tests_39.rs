// Batch 39: Dialog builtin action validation tests
//
// Focuses on:
// - ScriptInfo::with_shortcut constructor and field preservation
// - ScriptInfo::with_shortcut_and_alias constructor combinations
// - ScriptInfo::scriptlet constructor field validation
// - format_shortcut_hint: arrow key and special key conversions
// - format_shortcut_hint: alias variants (return, esc, opt, arrowdown)
// - parse_shortcut_keycaps: multi-char modifier combos
// - builders::format_shortcut_hint vs dialog::format_shortcut_hint
// - Clipboard: ordering of common actions (paste, copy, paste_keep_open)
// - Clipboard: destructive action ordering (delete, delete_multiple, delete_all)
// - Clipboard: image-only OCR position relative to pin/unpin
// - File context: total action count file vs dir on macOS
// - File context: copy_filename shortcut differs from path context
// - Path context: total action count file vs dir
// - Path context: move_to_trash is always last
// - Script context: with_frecency adds reset_ranking
// - Script context: agent has no view_logs but has copy_path
// - Scriptlet context: total action count without custom actions
// - Scriptlet context with custom: custom actions appear after run
// - AI bar: paste_image details
// - AI bar: section ordering matches declaration order
// - Notes: section distribution with selection + no trash + disabled auto
// - Notes: all actions have icons
// - Chat context: model actions come before continue_in_chat
// - Chat context: context_title with model name
// - New chat: last_used IDs use index format
// - New chat: model section actions use Settings icon
// - Note switcher: singular vs plural char count
// - Note switcher: section assignment pinned vs recent
// - coerce_action_selection: all headers returns None
// - build_grouped_items_static: filter_idx in Item matches enumerate order

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_39/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_39/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_39/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_39/tests_part_04.rs");
}
