//! Batch 14 — Builtin action validation tests
//!
//! Focus areas:
//! - Path context action ordering and position guarantees
//! - Clipboard OCR/open_with mutual exclusivity between text/image
//! - ScriptInfo constructor field isolation (with_all sets exactly 6 params)
//! - AI command bar exact icon-to-section exhaustive mapping
//! - Notes command bar section-action membership
//! - Note switcher title rendering (Untitled Note, bullet prefix, pinned override)
//! - Chat context edge: zero models with both flags false
//! - Scriptlet context custom action insertion ordering with multiple H3 actions
//! - File context macOS-only action count delta
//! - Cross-context description non-emptiness and keyword matching
//! - build_grouped_items_static with mixed sections and alternating patterns
//! - coerce_action_selection with interleaved headers
//! - score_action stacking: title + description + shortcut all match
//! - fuzzy_match Unicode subsequence
//! - parse_shortcut_keycaps with slash and number inputs
//! - to_deeplink_name with numeric-only and empty-after-strip inputs
//! - Action with_shortcut_opt Some vs None chaining
//! - CommandBarConfig notes_style field completeness
//! - Clipboard destructive ordering invariant across pin states
//! - Global actions always empty

#[cfg(test)]
mod tests {
    include!("dialog_builtin_action_validation_tests_14/tests_part_01.rs");
    include!("dialog_builtin_action_validation_tests_14/tests_part_02.rs");
    include!("dialog_builtin_action_validation_tests_14/tests_part_03.rs");
    include!("dialog_builtin_action_validation_tests_14/tests_part_04.rs");
    include!("dialog_builtin_action_validation_tests_14/tests_part_05.rs");
}
