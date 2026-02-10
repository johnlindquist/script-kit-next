// Batch 10: Builtin action validation tests
//
// 155 tests across 30 categories focusing on:
// - Clipboard frontmost_app_name propagation and isolation
// - Script action exact counts per flag combination
// - Scriptlet ordering guarantees with custom actions
// - AI command bar exact shortcut/icon values
// - Notes command bar exact icon/shortcut/section values
// - Path context exact shortcut values
// - File context exact description strings
// - FileType variants have no effect on file actions
// - Chat model checkmark logic and ID format
// - New chat provider_display_name propagation
// - Clipboard exact description strings
// - Script context with custom verbs
// - ActionsDialogConfig field defaults
// - ActionCategory PartialEq
// - Agent description content keywords
// - Cross-context frecency reset consistency

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_10/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_10/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_10/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_10/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_10/tests_part_05.rs");
}
