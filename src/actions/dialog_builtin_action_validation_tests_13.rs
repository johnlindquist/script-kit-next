//! Batch 13 — Builtin action validation tests
//!
//! Focus areas:
//! - format_shortcut_hint edge cases (non-modifier intermediate parts, aliased modifiers)
//! - ScriptInfo mutually-exclusive flags (agent vs script vs scriptlet vs builtin)
//! - Scriptlet context custom action value/has_action propagation
//! - Clipboard save_snippet/save_file universality (text and image)
//! - Path context copy_filename has no shortcut
//! - Note switcher description ellipsis boundary (exactly 60 chars)
//! - Chat context multi-model ordering and checkmark logic
//! - AI command bar actions without shortcuts
//! - CommandBarConfig close flag defaults
//! - Cross-builder shortcut/alias action symmetry
//! - Scriptlet context action verb propagation
//! - Agent context exact action IDs
//! - Deeplink URL in description for scriptlet context
//! - Notes command bar create_quicklink and export actions
//! - Action::new lowercase caching correctness

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_13/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_13/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_13/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_13/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_13/tests_part_05.rs");
}
