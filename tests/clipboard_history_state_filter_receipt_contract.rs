//! Source-level contract for the Run 2 Pass #36
//! `clipboard-history-filter-miss-visible-count-live` user story.
//!
//! Pass #14 recorded `empty-clipboard-state [!]` with four sub-gaps. Two
//! of them were closed earlier in Run 2 at the *contract* level:
//!
//!   - **Sub-gap 3** (`visibleChoiceCount` field missing from StateResult)
//!     was closed by Pass #33 (`tests/state_result_visible_choice_count_contract.rs`)
//!     which pinned the field in `src/protocol/message/variants/query_ops.rs`.
//!
//!   - **Sub-gap 2** (stdin `setFilter` didn't route into the subview's
//!     variant `filter` field) was closed by Pass #34
//!     (`tests/set_filter_routes_to_active_subview_contract.rs`) which
//!     pinned the `write_filter_to_current_subview` router in
//!     `src/app_impl/filter_input_updates.rs`.
//!
//! Pass #36 live-verifies that those two fixes compose correctly on the
//! `ClipboardHistoryView` surface specifically (the exact target of the
//! original `empty-clipboard-state` story): a stdin-driven `setFilter`
//! with a no-match string narrows the visible row set to zero while the
//! total dataset size stays pinned. Receipts from the live run:
//!
//!   - pre-filter    → choiceCount=100, visibleChoiceCount=100
//!   - setFilter "a" → choiceCount=100, visibleChoiceCount=52
//!   - setFilter "zzz_no_match_test_string"
//!                   → choiceCount=100, visibleChoiceCount=0
//!   - setFilter ""  → choiceCount=100, visibleChoiceCount=100
//!
//! The live receipt proves the end-to-end wiring works *today*, but a
//! future mechanical refactor of the `ClipboardHistoryView` arm in
//! `collect_state` (src/prompt_handler/mod.rs around line 2119) could
//! silently regress this by, e.g.:
//!
//!   (a) reading `self.filter_text` (stale on stdin `setFilter` for
//!       subviews — exactly the Pass #14 sub-gap 2 shape) instead of
//!       destructuring the variant's own `filter` field;
//!
//!   (b) putting `filtered_count` in both the `choiceCount` and
//!       `visibleChoiceCount` tuple positions, which would silently
//!       erase the total-dataset receipt and make a filter-miss look
//!       indistinguishable from an empty clipboard (the original story
//!       question `empty-clipboard-state` was trying to answer);
//!
//!   (c) switching the match to a case-sensitive `contains` and breaking
//!       the case-insensitive filter semantics users rely on.
//!
//! This contract test pins the arm's exact shape at source level so any
//! of those regressions is caught by `cargo test` before shipping,
//! independent of whether the live clipboard dataset is populated at
//! verification time.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

#[test]
fn clipboard_history_state_arm_destructures_variant_filter_and_selected_index() {
    // The arm must bind `filter` AND `selected_index` from the variant,
    // not read `self.filter_text` / `self.selected_index`. Reading the
    // app-level fields would reintroduce Pass #14 sub-gap (2): a stdin
    // `setFilter` that lands only in `self.filter_text` (the pre-Pass-#34
    // shape) would make the arm return stale counts.
    let arm_pos = PROMPT_HANDLER
        .find("AppView::ClipboardHistoryView {\n                        filter,")
        .expect(
            "src/prompt_handler/mod.rs must contain a `ClipboardHistoryView` \
             arm that destructures `filter,` on the line immediately after \
             the opening brace. Any other shape (binding via `..` or \
             reading from `self.filter_text`) regresses Pass #14 sub-gap \
             (2) — the variant's own `filter` field is the single source \
             of truth for the narrowed count receipt after Pass #34.",
        );
    let after = &PROMPT_HANDLER[arm_pos..];
    assert!(
        after.contains("selected_index,\n                    } => {"),
        "src/prompt_handler/mod.rs `ClipboardHistoryView` arm must also \
         destructure `selected_index,` on the next line. Without the \
         destructure, the arm would need `self.selected_index` — which \
         is the ScriptList-scoped cursor, not the subview cursor — and \
         the `selectedIndex` receipt would silently track the wrong field."
    );
}

#[test]
fn clipboard_history_state_arm_computes_filtered_count_from_variant_filter_case_insensitively() {
    // The narrowing computation must live inside the arm and key on the
    // destructured `filter` (NOT `self.filter_text`). It must also use
    // `to_lowercase().contains(...)` so the case-insensitive semantics
    // users see in `handle_filter_input_change` carry over to the stdin
    // `setFilter` path and its state receipt.
    let arm_pos = PROMPT_HANDLER
        .find("AppView::ClipboardHistoryView {\n                        filter,")
        .expect("ClipboardHistoryView arm must exist — see sibling contract");
    // Bound the body at the next `AppView::` arm so nothing downstream
    // can accidentally satisfy this contract.
    let body_end_rel = PROMPT_HANDLER[arm_pos..]
        .find("\n                    AppView::AgentChatHistoryView")
        .expect(
            "`ClipboardHistoryView` arm must be immediately followed by \
             `AppView::AgentChatHistoryView` in the getState match — if that \
             ordering changes, amend this contract rather than silently \
             widening the body search.",
        );
    let body = &PROMPT_HANDLER[arm_pos..arm_pos + body_end_rel];

    assert!(
        body.contains("let entries = &self.cached_clipboard_entries;"),
        "`ClipboardHistoryView` getState arm must read the full dataset \
         from `self.cached_clipboard_entries` — this is the value that \
         feeds the `choiceCount` position of the returned tuple, and \
         `empty-clipboard-state` distinguishes \"filter narrowed to zero\" \
         (choiceCount>0, visibleChoiceCount=0) from \"clipboard genuinely \
         empty\" (choiceCount=0, visibleChoiceCount=0) via this receipt."
    );

    assert!(
        body.contains("let filter_lower = filter.to_lowercase();"),
        "`ClipboardHistoryView` getState arm must lowercase the \
         destructured `filter` before matching (`let filter_lower = \
         filter.to_lowercase();`). Lowercasing `self.filter_text` instead \
         would reintroduce the stale-filter bug Pass #34 fixed; \
         switching to a case-sensitive match would break the user-facing \
         semantics established by `handle_filter_input_change`."
    );

    assert!(
        body.contains(".contains(&filter_lower)"),
        "`ClipboardHistoryView` getState arm must narrow via \
         `.contains(&filter_lower)` on the entries. Changing to \
         `.starts_with(...)` or `.eq(...)` would silently tighten \
         filter semantics and break parity with the text-change path \
         (`handle_filter_input_change`), which users expect the stdin \
         `setFilter` receipt to mirror."
    );

    assert!(
        body.contains("e.text_preview.to_lowercase().contains(&filter_lower)"),
        "`ClipboardHistoryView` getState arm must compare against \
         `e.text_preview.to_lowercase()` — the `text_preview` field is \
         what the UI renders and what the user sees narrow. Comparing \
         against `e.full_text` or `e.app_name` here would drift the \
         receipt away from the rendered list and the \
         visibleChoiceCount=0 clause of `empty-clipboard-state` could \
         hit on entries the user cannot see."
    );
}

#[test]
fn clipboard_history_state_arm_returns_dataset_len_and_filtered_count_in_correct_tuple_slots() {
    // The 6-tuple returned by the arm is destructured at
    // `src/prompt_handler/mod.rs:1899` as
    //   (prompt_type, prompt_id, placeholder, input_value,
    //    choice_count, visible_choice_count, selected_index, selected_value)
    //
    // For `empty-clipboard-state`'s corrected acceptance clause, slot 4
    // (`choice_count`) MUST be `entries.len()` and slot 5
    // (`visible_choice_count`) MUST be `filtered_count`. Swapping these
    // would make the narrowing receipt indistinguishable from a genuine
    // empty-clipboard state.
    let arm_pos = PROMPT_HANDLER
        .find("AppView::ClipboardHistoryView {\n                        filter,")
        .expect("ClipboardHistoryView arm must exist — see sibling contract");
    let body_end_rel = PROMPT_HANDLER[arm_pos..]
        .find("\n                    AppView::AgentChatHistoryView")
        .expect("ClipboardHistoryView arm must precede AgentChatHistoryView");
    let body = &PROMPT_HANDLER[arm_pos..arm_pos + body_end_rel];

    // Pin the exact tuple shape: after the `(` and the `"clipboardHistory"`
    // prompt type, two `None,` placeholder slots, then `filter.clone(),`
    // (inputValue), then `entries.len(),` (choiceCount), then
    // `filtered_count,` (visibleChoiceCount), then `*selected_index as i32,`.
    //
    // The literal newline-and-indent between the fields lets us pin
    // positional semantics without being brittle to the surrounding
    // `(` placement — inserting a whitespace-only line between tuple
    // elements would still parse, and this contract would still pass.
    assert!(
        body.contains("\"clipboardHistory\".to_string(),"),
        "arm must return `\"clipboardHistory\"` as the first tuple slot \
         (promptType) — any other string would make `getState.promptType` \
         stop matching the `clipboard-history` `triggerBuiltin` surface."
    );

    // Slot 3 (inputValue) must be the variant's `filter`, not
    // `self.filter_text`. The `.clone()` ensures the field isn't moved.
    assert!(
        body.contains("filter.clone(),"),
        "arm must place `filter.clone(),` in slot 3 (inputValue). \
         Reading `self.filter_text.clone()` instead would reintroduce \
         the Pass #14 sub-gap: on a subview, `self.filter_text` can \
         diverge from the variant's `filter` until `handle_filter_input_change` \
         or `write_filter_to_current_subview` fires."
    );

    // Slot 4 (choiceCount) = entries.len(); Slot 5 (visibleChoiceCount) = filtered_count.
    // Pin the literal ordering between these two lines in the tuple so
    // a transposition would fail this test.
    let entries_len_pos = body.find("entries.len(),").expect(
        "arm must emit `entries.len(),` as slot 4 (choiceCount). If this \
         becomes `filtered_count,` the narrowing receipt collapses: a \
         filter-miss would look identical to an empty clipboard, and \
         `empty-clipboard-state`'s acceptance clause fails at the \
         source-semantic level.",
    );
    let filtered_count_pos = body.find("filtered_count,").expect(
        "arm must emit `filtered_count,` as slot 5 (visibleChoiceCount). \
         Removing or renaming this variable would lose the narrowing \
         receipt that Pass #33 promoted to first-class protocol field \
         and Pass #34's routing makes reachable via stdin.",
    );
    assert!(
        entries_len_pos < filtered_count_pos,
        "`entries.len(),` (slot 4, choiceCount) must appear BEFORE \
         `filtered_count,` (slot 5, visibleChoiceCount) in the returned \
         tuple. Transposing the two flips the semantics: `choiceCount` \
         would track the narrowed count and `visibleChoiceCount` would \
         track the dataset, and `empty-clipboard-state` would no longer \
         be able to distinguish filter-miss from empty-clipboard."
    );

    assert!(
        body.contains("*selected_index as i32,"),
        "arm must emit `*selected_index as i32,` in slot 6 — the cast \
         matches the i32 field in `StateResult.selectedIndex`. Using \
         `self.selected_index` here (instead of the variant's \
         `selected_index`) would cause the selection receipt to drift \
         off the subview's cursor and track the ScriptList cursor on a \
         subview surface."
    );
}
