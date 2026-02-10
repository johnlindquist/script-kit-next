// Batch 6: Built-in action behavioral validation tests
//
// 160+ tests validating action invariants NOT covered in batches 1-5.
// Focus areas:
// - ScriptInfo impossible flag combinations (is_script+is_scriptlet, is_script+is_agent)
// - Action verb propagation across all contexts
// - Deeplink description URL format for various name patterns
// - Clipboard entry edge cases (empty preview, long preview, special app names)
// - Chat context scaling (many models, duplicate providers, empty display names)
// - Notes info systematic boolean combos (all 8 permutations w/ section labels)
// - Note switcher mixed pinned/unpinned ordering and sections
// - New chat with partial sections (empty presets, empty models, etc.)
// - Combined score stacking (title+desc+shortcut all matching)
// - build_grouped_items_static consecutive same-section (no duplicate headers)
// - coerce_action_selection all-headers edge case
// - format_shortcut_hint (ActionsDialog version) comprehensive coverage
// - Path context long names and special chars
// - File context all FileType variants
// - Action builder chaining immutability
// - CommandBarConfig default field values
// - Scriptlet context action count vs script context action count comparison
// - Agent ScriptInfo with full flag set (shortcut+alias+frecency)

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_6/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_6/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_6/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_6/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_6/tests_part_05.rs");
    include!("dialog_builtin_action_validation_tests_6/tests_part_06.rs");
}
