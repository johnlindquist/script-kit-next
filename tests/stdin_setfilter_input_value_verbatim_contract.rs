//! Source-level contract for the setFilter → inputValue verbatim-echo path.
//!
//! Background: Run 4 Pass #8 (attacker probe) observed that a
//! 10 000-character `setFilter.text` payload produced a 10 000-character
//! `getState.inputValue` response with no truncation marker. Filed as
//! `[?] stdin-setfilter-inputvalue-unbounded`. Acceptance criterion (from
//! audits/afk/stories.md): either bound `inputValue.length`, or
//! explicitly document the no-bound contract and pin the stdin line cap
//! (`MAX_STDIN_COMMAND_BYTES` = 16 KiB) as the sole gate. Run 8 Pass #23
//! picked option (b) — document + pin the existing behavior as an
//! intentional contract, because introducing a cap would silently change
//! callers that legitimately pass long filters and race the MAX-bytes
//! stdin line cap anyway.
//!
//! Refactor threat: a well-meaning contributor adds a
//! `text.truncate(4096)` or `text.chars().take(N)` step inside
//! `set_filter_text_immediate`, or adds an equivalent cap to
//! `current_input_value`, thinking it's a "safety" fix. That silently
//! breaks every automation caller that relies on `inputValue` echoing
//! its full stdin payload. These asserts catch such a refactor before
//! merge.

const FILTER_INPUT_UPDATES: &str = include_str!("../src/app_impl/filter_input_updates.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");

#[test]
fn set_filter_text_immediate_doc_comment_pins_verbatim_contract() {
    // The doc comment at set_filter_text_immediate must name the
    // verbatim-echo contract and the 16 KiB stdin cap. A future refactor
    // that silently drops or weakens it (e.g. rewords to remove the
    // MAX_STDIN_COMMAND_BYTES reference) trips this assert.
    let sig_idx = FILTER_INPUT_UPDATES
        .find("pub(crate) fn set_filter_text_immediate(")
        .expect(
            "filter_input_updates.rs must keep fn set_filter_text_immediate — \
             this is the stdin setFilter writer",
        );
    // Doc comment must appear in the ~1600 bytes immediately preceding
    // the signature.
    let window_start = sig_idx.saturating_sub(1600);
    let doc_window = &FILTER_INPUT_UPDATES[window_start..sig_idx];
    assert!(
        doc_window.contains("Verbatim-echo contract"),
        "set_filter_text_immediate's doc comment must keep the \
         \"Verbatim-echo contract\" header so audit readers find it by \
         grep"
    );
    assert!(
        doc_window.contains("MAX_STDIN_COMMAND_BYTES"),
        "set_filter_text_immediate's doc comment must reference \
         MAX_STDIN_COMMAND_BYTES so the single bound is discoverable \
         from the setFilter writer"
    );
    assert!(
        doc_window.contains("stdin-setfilter-inputvalue-unbounded"),
        "set_filter_text_immediate's doc comment must cite the anomaly \
         slug so future contributors find the historical rationale"
    );
}

#[test]
fn set_filter_text_immediate_does_not_truncate_before_store() {
    // Between the function signature and the `self.filter_text =
    // text.clone();` assignment, no truncation or cap may appear. A
    // refactor that adds `text.truncate(N)`, `text = text[..N].into()`,
    // or `text.chars().take(N)` between these points silently breaks
    // the verbatim contract.
    let sig_idx = FILTER_INPUT_UPDATES
        .find("pub(crate) fn set_filter_text_immediate(")
        .expect("set_filter_text_immediate must exist");
    let store_idx = FILTER_INPUT_UPDATES[sig_idx..]
        .find("self.filter_text = text.clone();")
        .map(|off| sig_idx + off)
        .expect(
            "set_filter_text_immediate must store `self.filter_text = \
             text.clone();` early in the body — this is the verbatim pin",
        );
    let body = &FILTER_INPUT_UPDATES[sig_idx..store_idx];
    for forbidden in [".truncate(", ".chars().take(", ".char_indices().take("] {
        assert!(
            !body.contains(forbidden),
            "set_filter_text_immediate must not contain `{forbidden}` \
             before storing self.filter_text — that would silently cap \
             the verbatim payload callers depend on"
        );
    }
}

#[test]
fn current_input_value_returns_filter_text_clone_for_script_list() {
    // The ScriptList arm of current_input_value must be a bare
    // `self.filter_text.clone()`. Any wrapper (`.chars().take(N).collect()`,
    // `&self.filter_text[..N]`) breaks the verbatim-echo contract on
    // the read side.
    let fn_idx = PROMPT_HANDLER
        .find("fn current_input_value(&self) -> String {")
        .expect(
            "prompt_handler/mod.rs must keep fn current_input_value — it \
             is the sole reader that produces getState.inputValue",
        );
    let body_window = &PROMPT_HANDLER[fn_idx..fn_idx + 400];
    assert!(
        body_window.contains("AppView::ScriptList => self.filter_text.clone(),"),
        "current_input_value's ScriptList arm must return \
         `self.filter_text.clone()` unconditionally — a cap or \
         transformation here silently breaks the verbatim contract \
         pinned at set_filter_text_immediate"
    );
}

#[test]
fn max_stdin_command_bytes_is_sixteen_kib() {
    // The verbatim-echo contract names MAX_STDIN_COMMAND_BYTES = 16 KiB
    // as the sole bound. If the cap is ever changed, the doc comments
    // at set_filter_text_immediate AND current_input_value must be
    // updated in the same commit. Pinning the 16 KiB literal here
    // forces that cross-file consistency check.
    assert!(
        STDIN_COMMANDS.contains("const MAX_STDIN_COMMAND_BYTES: usize = 16 * 1024;"),
        "stdin_commands/mod.rs must keep MAX_STDIN_COMMAND_BYTES = \
         16 * 1024 — the setFilter verbatim-echo contract in \
         filter_input_updates.rs references this literal cap"
    );
}
