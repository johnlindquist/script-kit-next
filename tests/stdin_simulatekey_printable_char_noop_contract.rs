//! Source-level contract for the Run 9 Pass #3 acceptance of
//! `[?] attacker-simulatekey-noop-on-hidden-non-actions-views` (filed
//! Run 8 Pass #20). The anomaly asked whether printable single-char
//! `simulateKey` events should route into a visible view's filter
//! input.
//!
//! Collapsed in Run 13 into a single unified helper at `src/app_impl/simulate_key_dispatch.rs`.
//! This test verifies that the live dispatcher sites delegate to the helper,
//! and that the helper retains the no-op shape for printable chars on
//! non-actions views.

const CANONICAL_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const SIMULATE_KEY_HELPER: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        "src/main_entry/runtime_stdin_match_simulate_key.rs",
        CANONICAL_SIMULATEKEY,
    ),
    ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
];

const EMOJI_UNHANDLED: &str = "SimulateKey: Unhandled key '{}' in EmojiPicker";
const CLIPBOARD_UNHANDLED: &str = "SimulateKey: Unhandled key '{}' in ClipboardHistoryView";

#[test]
fn both_dispatchers_keep_emoji_picker_unhandled_log_shape() {
    // Both dispatchers must delegate to dispatch_simulate_key helper
    for (name, source) in DISPATCHERS {
        assert!(
            source.contains("view.dispatch_simulate_key("),
            "{name} MUST delegate to view.dispatch_simulate_key"
        );
    }

    // Helper must contain the EmojiPicker unhandled log shape
    assert!(
        SIMULATE_KEY_HELPER.contains(EMOJI_UNHANDLED),
        "simulate_key_dispatch.rs is missing the EmojiPicker `Unhandled key` log line"
    );
}

#[test]
fn both_dispatchers_keep_clipboard_history_unhandled_log_shape() {
    // Helper must contain the ClipboardHistoryView unhandled log shape
    assert!(
        SIMULATE_KEY_HELPER.contains(CLIPBOARD_UNHANDLED),
        "simulate_key_dispatch.rs is missing the ClipboardHistoryView `Unhandled key` log line"
    );
}

#[test]
fn dispatchers_do_not_route_simulatekey_chars_into_view_filter() {
    // Neither the helper nor the dispatchers should call filter-mutation APIs within simulateKey
    for (name, source) in &[
        ("src/app_impl/simulate_key_dispatch.rs", SIMULATE_KEY_HELPER),
        (
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            CANONICAL_SIMULATEKEY,
        ),
    ] {
        for forbidden in [
            "set_filter_text_immediate",
            "write_filter_to_current_subview",
            "sync_builtin_query_state",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must not call `{forbidden}`"
            );
        }
    }
}

#[test]
fn emoji_picker_unhandled_arm_returns_without_side_effect() {
    let anchor = SIMULATE_KEY_HELPER
        .find(EMOJI_UNHANDLED)
        .unwrap_or_else(|| panic!("simulate_key_dispatch.rs lost the EmojiPicker anchor"));
    // The Unhandled-key log call ends at the first `;` after the anchor;
    // the very next statement must be `return;` (indentation-independent).
    let after_log = &SIMULATE_KEY_HELPER[anchor..];
    let semi = after_log
        .find(';')
        .expect("EmojiPicker Unhandled-key log must terminate with a statement");
    assert!(
        after_log[semi + 1..].trim_start().starts_with("return;"),
        "EmojiPicker `_ =>` arm in simulate_key_dispatch.rs must `return;` immediately after the Unhandled-key log"
    );
}

#[test]
fn clipboard_history_else_branch_has_no_return_or_mutation() {
    let anchor = SIMULATE_KEY_HELPER
        .find(CLIPBOARD_UNHANDLED)
        .unwrap_or_else(|| panic!("simulate_key_dispatch.rs lost the ClipboardHistory anchor"));
    let window = &SIMULATE_KEY_HELPER[anchor..(anchor + 200).min(SIMULATE_KEY_HELPER.len())];
    let semi_count = window.matches(';').count();
    assert!(
        semi_count <= 1,
        "ClipboardHistoryView `else` branch in simulate_key_dispatch.rs has {semi_count} semicolons after the Unhandled-key log"
    );
}
