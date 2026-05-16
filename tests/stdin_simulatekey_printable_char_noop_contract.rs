//! Source-level contract for the Run 9 Pass #3 acceptance of
//! `[?] attacker-simulatekey-noop-on-hidden-non-actions-views` (filed
//! Run 8 Pass #20). The anomaly asked whether printable single-char
//! `simulateKey` events should route into a visible view's filter
//! input. Live repro with `windowVisible:true` + `isFocused:true` +
//! 5× `simulateKey z` on `ClipboardHistoryView` returned
//! `getState.inputValue:""` — FALSIFIED the visibility-gate hypothesis
//! and confirmed the no-op is unconditional, not visibility-gated.
//!
//! Acceptance option (a) from the anomaly menu: pin the existing
//! no-op shape as intentional. The API boundary is: `simulateKey`
//! delivers UI key events (arrows, enter, escape, modifiers) that a
//! view chooses to handle; `setFilter` is the programmatic
//! text-input API. Silently routing printable chars from simulateKey
//! into a filter field would collapse two distinct APIs into one,
//! masking caller bugs (a script that meant to send a real keystroke
//! against a view with no key binding for that char would instead
//! grow the filter string). Callers that want filter text must use
//! `setFilter`, which routes through
//! `write_filter_to_current_subview` and handles all 17 filter-
//! bearing subviews uniformly.
//!
//! Refactor threat: a well-meaning contributor "fixes" the
//! `Unhandled key` log noise by wiring the `_ =>` arm of
//! `AppView::EmojiPickerView` or the `else` branch of
//! `AppView::ClipboardHistoryView` (in either
//! `src/main_entry/runtime_stdin_match_simulate_key.rs` or
//! `src/main_entry/app_run_setup.rs`) to call
//! `set_filter_text_immediate` / `write_filter_to_current_subview`
//! with the pressed character appended. Compilation passes, existing
//! tests pass, but the API boundary above silently erodes. These
//! source-grep asserts catch such a refactor before merge by
//! forbidding the filter-mutation call sites inside the simulateKey
//! dispatcher files.

const CANONICAL_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

/// Returns the bytes of `app_run_setup.rs` scoped to the
/// `ExternalCommand::SimulateKey` arm only. The file is a mega-
/// dispatcher that also contains `ExternalCommand::SetFilter` (which
/// legitimately calls `set_filter_text_immediate`); the forbidden-
/// substring test would false-positive without this slice.
fn app_run_setup_simulatekey_block() -> &'static str {
    let start = APP_RUN_SETUP
        .find("ExternalCommand::SimulateKey { ref key, ref modifiers, .. } => {")
        .expect(
            "app_run_setup.rs must declare ExternalCommand::SimulateKey — refactor \
             that renames this arm must update this test's anchor",
        );
    // Next sibling ExternalCommand variant marks the end of the
    // SimulateKey arm. TriggerAction is the immediate next variant as
    // of this pin.
    let end_offset = APP_RUN_SETUP[start..]
        .find("ExternalCommand::TriggerAction {")
        .expect(
            "app_run_setup.rs must continue with ExternalCommand::TriggerAction \
             after SimulateKey — sibling-variant reorder must update this anchor",
        );
    &APP_RUN_SETUP[start..start + end_offset]
}

fn dispatchers() -> [(&'static str, &'static str); 2] {
    [
        (
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            CANONICAL_SIMULATEKEY,
        ),
        (
            "src/main_entry/app_run_setup.rs (SimulateKey arm)",
            app_run_setup_simulatekey_block(),
        ),
    ]
}

const EMOJI_UNHANDLED: &str = "SimulateKey: Unhandled key '{}' in EmojiPicker";
const CLIPBOARD_UNHANDLED: &str = "SimulateKey: Unhandled key '{}' in ClipboardHistoryView";

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn both_dispatchers_keep_emoji_picker_unhandled_log_shape() {
    // The `_ =>` arm in `AppView::EmojiPickerView` MUST continue to
    // emit the `SimulateKey: Unhandled key '…' in EmojiPicker` log
    // line and return without mutating any filter state. Removing or
    // rewording the log line hides the no-op from audit receipts;
    // replacing the `return` with a filter-mutation call collapses
    // the simulateKey/setFilter API boundary.
    for (name, source) in &dispatchers() {
        assert!(
            source.contains(EMOJI_UNHANDLED),
            "{name} is missing the EmojiPicker `Unhandled key` log \
             line. The no-op on printable chars is the intentional \
             contract (see `attacker-simulatekey-noop-on-hidden-non-\
             actions-views` — Run 9 Pass #3). Callers that want to \
             change the filter must use `setFilter`, not simulateKey."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn both_dispatchers_keep_clipboard_history_unhandled_log_shape() {
    // The `else` branch inside `AppView::ClipboardHistoryView` MUST
    // continue to emit the `SimulateKey: Unhandled key '…' in
    // ClipboardHistoryView` log line. The `if has_cmd && key_lower
    // == "k"` arm is the only handled case on this view; everything
    // else is a documented no-op.
    for (name, source) in &dispatchers() {
        assert!(
            source.contains(CLIPBOARD_UNHANDLED),
            "{name} is missing the ClipboardHistoryView `Unhandled \
             key` log line. Run 9 Pass #3 pins this shape as \
             intentional no-op behavior on printable single-char \
             simulateKey events."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn dispatchers_do_not_route_simulatekey_chars_into_view_filter() {
    // The simulateKey dispatcher files MUST NOT call any of the
    // filter-mutation APIs. These are the sinks used by the
    // `setFilter` stdin command (via `write_filter_to_current_subview`
    // in `src/app_impl/filter_input_updates.rs`). A dispatcher that
    // reaches for these names is routing keystrokes into a filter —
    // precisely the API boundary erosion this Pin defends against.
    //
    // Note: if future work intentionally wants simulateKey to drive
    // filter input for a specific view (e.g., a "raw keyboard"
    // debugging mode), that work should (a) remove this Pin in the
    // same commit, (b) update the anomaly closure note in
    // `audits/afk/stories.md`, and (c) update the rationale below.
    // The point of the Pin is to force that conversation.
    for (name, source) in &dispatchers() {
        for forbidden in [
            "set_filter_text_immediate",
            "write_filter_to_current_subview",
            "sync_builtin_query_state",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must not call `{forbidden}` — that is the \
                 setFilter sink; routing simulateKey chars through \
                 it collapses the simulateKey/setFilter API \
                 boundary. See `attacker-simulatekey-noop-on-\
                 hidden-non-actions-views` in \
                 `audits/afk/stories.md` for the rationale."
            );
        }
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn emoji_picker_unhandled_arm_returns_without_side_effect() {
    // Inside the `_ =>` arm that emits the EmojiPicker `Unhandled
    // key` log, the very next statement MUST be `return;`. A
    // refactor that drops the `return;` and lets control fall
    // through to the scroll/notify code below would mutate view
    // state on a key the arm claimed was unhandled — a silent
    // behavior change that breaks the no-op contract.
    for (name, source) in &dispatchers() {
        let anchor = source
            .find(EMOJI_UNHANDLED)
            .unwrap_or_else(|| panic!("{name} lost the EmojiPicker anchor"));
        // Scope: 100 bytes from the log anchor. The `return;` sits
        // on the line immediately after the log emission; 100 bytes
        // survives a reasonable reformat but catches a fall-through.
        let window = &source[anchor..(anchor + 150).min(source.len())];
        assert!(
            window.contains("return;"),
            "{name} EmojiPicker `_ =>` arm must `return;` \
             immediately after the Unhandled-key log — a fall-\
             through would silently mutate view state on keys the \
             arm already disclaimed"
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn clipboard_history_else_branch_has_no_return_or_mutation() {
    // Inside the `else` branch that emits the ClipboardHistoryView
    // `Unhandled key` log, the arm MUST end at the closing brace
    // of the `else` with no additional statement between the log
    // call and the `}`. A contributor that adds a statement here
    // (e.g., to route into a filter) is violating the no-op Pin.
    //
    // The check: find the Unhandled log line, then scan forward
    // for the first `}` — there MUST NOT be a semicolon (statement
    // terminator) other than the log macro's own before that brace.
    for (name, source) in &dispatchers() {
        let anchor = source
            .find(CLIPBOARD_UNHANDLED)
            .unwrap_or_else(|| panic!("{name} lost the ClipboardHistory anchor"));
        // Scan from the log anchor forward 200 bytes for a close
        // brace. Within that window, only the log call's own
        // trailing `);` should appear — no extra statements.
        let window = &source[anchor..(anchor + 200).min(source.len())];
        // Count `;` occurrences — the log call contributes exactly
        // one. Anything higher means a statement was inserted.
        // (The call site uses `logging::log("STDIN", &format!(...));`
        // which contributes a single `;`.)
        let semi_count = window.matches(';').count();
        assert!(
            semi_count <= 1,
            "{name} ClipboardHistoryView `else` branch has \
             {semi_count} semicolons after the Unhandled-key log \
             — expected at most 1 (the log call's own). An extra \
             statement inside the `else` is likely a filter-\
             mutation step that violates the no-op contract. \
             Window: `{window}`"
        );
    }
}
