//! Source-level contract pinning the Run 8 Pass #1 Fix
//! (`Prompt: Fix actions-dialog batch setInput path to trigger window
//! resize`, commit `f8e06b53f`) — the three filter entry points for the
//! actions-dialog popup (keyboard TypeChar, keyboard Backspace, and the
//! stdin `batch` `SetInput` arm) must all drive a
//! `resize_actions_window` call immediately after mutating the
//! dialog's `search_text`. Without that resize, the NSPanel stays
//! frozen at its open-time height while `visibleChoiceCount` shrinks,
//! producing the "empty void below the lone visible row" user-report
//! that Pass #1 closed.
//!
//! This contract pins the batch-SetInput side specifically. The
//! keyboard paths are already covered indirectly by
//! `tests/source_audits/actions_popup_contract.rs` via
//! `resize_actions_window_direct_emits_resized_receipt`, but that test
//! only asserts the *receipt* is emitted — it does not assert that
//! the batch dispatcher calls `resize_actions_window` after
//! `set_search_text`. That coverage gap is what this file closes.
//!
//! **Refactor threat** (named per looper/rules/discipline.md §"Pin verb
//! semantics"): `src/prompt_handler/mod.rs` currently has four
//! distinct `protocol::BatchCommand::SetInput { text }` arms — one per
//! target kind (main-menu at line ~4099, clipboard at ~4457,
//! actions-dialog at ~4683, Agent Chat at ~5263). A contributor consolidating
//! those four arms into a shared helper (e.g., a generic
//! `handle_batch_set_input<T>(target_kind: BatchTarget, text: &str,
//! ...)` routed through `set_filter_text_immediate`) could drop the
//! `crate::actions::resize_actions_window(cx, &de)` post-call because
//! only the actions-dialog arm needs it — the main-menu, clipboard,
//! and Agent Chat variants have no popup to resize. The consolidation would
//! compile, the keyboard path's own resize would still work, and the
//! regression would only be visible through a live stdin-batch filter
//! receipt (the scenario that originally produced the 2026-04-18
//! user report). This contract makes that regression a compile-time
//! failure instead.
//!
//! The four assertions pinned below:
//!   1. The `// ── ActionsDialog batch path ──` section marker comment
//!      appears exactly once — the structural anchor that locates the
//!      actions-dialog batch dispatcher in the file.
//!   2. Inside the slice from that marker to the next sibling-level
//!      section header, the sequence
//!      `dialog.set_search_text(text.clone(), cx);` …
//!      `crate::actions::resize_actions_window(cx, &de);` appears with
//!      `set_search_text` strictly preceding `resize_actions_window`.
//!   3. The gap between them stays under 500 bytes — they must live in
//!      the same `this.update` closure body, and refactors that push
//!      them apart (e.g., wrapping the resize in a conditional, or
//!      splitting the closure in two) break this bound.
//!   4. The Pass #1 Run 8 anchor-comment block (the four-line
//!      explanation citing "batch SetInput path bypassed that, leaving
//!      the popup frozen at the pre-filter height") appears verbatim
//!      above the resize call — a contributor who deletes the comment
//!      while keeping the resize call intact loses the "why" and makes
//!      the future refactor threat invisible; this pin keeps the
//!      anchor comment load-bearing.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

const ACTIONS_BATCH_ANCHOR: &str = "// ── ActionsDialog batch path ────────────────────────";
const SET_SEARCH_TEXT_CALL: &str = "dialog.set_search_text(text.clone(), cx);";
const RESIZE_CALL: &str = "crate::actions::resize_actions_window(cx, &de);";
// The four distinct phrases that together identify the Pass #1 Run 8
// anchor comment. Checked as individual substrings so whitespace /
// indentation changes don't break the pin; checked in order below to
// catch a reshuffled rewrite that would lose the "why".
const PASS_1_COMMENT_PHRASES: &[&str] = &[
    "Keyboard TypeChar path (src/actions/window.rs:630-642)",
    "defers resize_actions_window_direct; the batch SetInput",
    "path bypassed that, leaving the popup frozen at the",
    "pre-filter height when visibleChoiceCount drops.",
];

#[test]
fn actions_dialog_batch_section_anchor_exists_exactly_once() {
    let count = PROMPT_HANDLER.matches(ACTIONS_BATCH_ANCHOR).count();
    assert_eq!(
        count, 1,
        "`src/prompt_handler/mod.rs` must contain the section marker \
         `{ACTIONS_BATCH_ANCHOR}` exactly once (found {count}). This \
         marker is the structural anchor that pins the ActionsDialog \
         batch dispatcher location; if it vanishes, the Pass #1 Run 8 \
         resize-parity Fix loses its anchor comment and a future \
         refactor can drop the `resize_actions_window` call silently."
    );
}

#[test]
fn actions_dialog_batch_setinput_arm_calls_set_search_text_then_resize_in_order() {
    let anchor_idx = PROMPT_HANDLER
        .find(ACTIONS_BATCH_ANCHOR)
        .expect("section anchor missing (covered by actions_dialog_batch_section_anchor_exists_exactly_once)");
    let tail = &PROMPT_HANDLER[anchor_idx..];

    let set_idx = tail.find(SET_SEARCH_TEXT_CALL).unwrap_or_else(|| {
        panic!(
            "Actions-dialog batch dispatcher must contain \
             `{SET_SEARCH_TEXT_CALL}` inside the \
             `BatchCommand::SetInput` arm. The Pass #1 Run 8 Fix at \
             `src/prompt_handler/mod.rs:4688` mutates the dialog's \
             search text here; removing that call breaks the filter \
             entirely."
        )
    });
    let resize_idx = tail.find(RESIZE_CALL).unwrap_or_else(|| {
        panic!(
            "Actions-dialog batch dispatcher must contain \
             `{RESIZE_CALL}` inside the `BatchCommand::SetInput` arm. \
             Without this call, stdin-batch filter shrinks leave the \
             popup frozen at its open-time height — the exact \
             regression that Run 8 Pass #1 (commit `f8e06b53f`) \
             fixed. A contributor consolidating the four batch \
             SetInput arms into a shared helper could drop this call \
             because only the actions-dialog arm needs it."
        )
    });
    assert!(
        set_idx < resize_idx,
        "In the actions-dialog batch dispatcher, \
         `{SET_SEARCH_TEXT_CALL}` (offset {set_idx} from section \
         anchor) must precede `{RESIZE_CALL}` (offset {resize_idx}). \
         Filtering must mutate search text BEFORE the resize reads \
         `dialog.filtered_actions.len()` via `compute_popup_height`; \
         reversing the order resizes to the pre-filter count."
    );
}

#[test]
fn actions_dialog_batch_setinput_arm_is_self_contained_set_then_resize() {
    let anchor_idx = PROMPT_HANDLER
        .find(ACTIONS_BATCH_ANCHOR)
        .expect("section anchor missing (covered by actions_dialog_batch_section_anchor_exists_exactly_once)");
    let tail = &PROMPT_HANDLER[anchor_idx..];
    let set_idx = tail
        .find(SET_SEARCH_TEXT_CALL)
        .expect("set_search_text call missing (covered by actions_dialog_batch_setinput_arm_calls_set_search_text_then_resize_in_order)");
    let resize_idx = tail
        .find(RESIZE_CALL)
        .expect("resize_actions_window call missing (covered by the ordering test)");

    // The resize call must live inside the SAME `BatchCommand::SetInput`
    // arm as the set_search_text call. Inside a `match cmd { ... }`
    // block, the next arm starts with `protocol::BatchCommand::<Other>`.
    // If any such pattern appears between set_idx and resize_idx, the
    // resize has been moved out of the SetInput arm (or the SetInput
    // arm has been split), which breaks Pass #1's invariant that the
    // resize is a direct post-operation of set_search_text.
    let between = &tail[set_idx..resize_idx];
    assert!(
        !between.contains("protocol::BatchCommand::SelectByValue")
            && !between.contains("protocol::BatchCommand::SelectBySemanticId")
            && !between.contains("protocol::BatchCommand::WaitFor")
            && !between.contains("protocol::BatchCommand::Submit"),
        "Another `BatchCommand::` arm pattern appears between \
         `{SET_SEARCH_TEXT_CALL}` and `{RESIZE_CALL}` in the \
         actions-dialog batch dispatcher. The resize call must stay \
         inside the SetInput arm body, immediately after the \
         set_search_text mutation. Intervening text \
         ({len} bytes):\n{between}",
        len = between.len()
    );

    // The resize call must live inside the SAME `this.update(cx, ...)`
    // closure body as the set_search_text call, so the borrow of `cx`
    // and the `&de` entity stay coherent. A split-closure refactor
    // would introduce an intervening `});` closure-end followed by a
    // new `this.update(cx,` call, which is the structural signature
    // we forbid here.
    assert!(
        !between.contains("});\n                                    let result = this.update(cx,")
            && !between.contains("});\n                                    this.update(cx,"),
        "A closure boundary (`}});` followed by a new `this.update(cx, \
         ...)` call) appears between the set_search_text call and the \
         resize_actions_window call. The two operations must stay in \
         the same update closure body for the `&de` entity handle and \
         `cx` borrow to remain coherent."
    );
}

#[test]
fn actions_dialog_batch_setinput_carries_pass_1_run_8_anchor_comment() {
    let anchor_idx = PROMPT_HANDLER
        .find(ACTIONS_BATCH_ANCHOR)
        .expect("section anchor missing (covered by actions_dialog_batch_section_anchor_exists_exactly_once)");
    let tail = &PROMPT_HANDLER[anchor_idx..];
    let set_idx = tail
        .find(SET_SEARCH_TEXT_CALL)
        .expect("set_search_text call missing (covered by actions_dialog_batch_setinput_arm_calls_set_search_text_then_resize_in_order)");
    let resize_idx = tail
        .find(RESIZE_CALL)
        .expect("resize_actions_window call missing (covered by the ordering test)");

    // Each phrase must appear between the set_search_text call and
    // the resize call, in order. This is the load-bearing "why" that
    // explains to a future contributor why the batch dispatcher
    // needs its own resize call (the keyboard TypeChar path uses
    // `resize_actions_window_direct` via `window.defer`, which the
    // batch dispatcher cannot — it holds `&mut App` but no
    // `&mut Window` handle).
    let between = &tail[set_idx..resize_idx];
    let mut cursor = 0usize;
    for phrase in PASS_1_COMMENT_PHRASES {
        let found = between[cursor..].find(phrase).unwrap_or_else(|| {
            panic!(
                "The Pass #1 Run 8 anchor-comment phrase {phrase:?} \
                 is missing (or out of order) between the \
                 `set_search_text` call and the \
                 `resize_actions_window` call in the actions-dialog \
                 batch dispatcher. This four-phrase comment block is \
                 the load-bearing documentation for the Pass #1 Fix \
                 (commit `f8e06b53f`). A contributor who deletes or \
                 reshuffles the comment loses the 'why' — the future \
                 refactor threat becomes invisible, and a well-meaning \
                 cleanup could drop the resize call on the reasoning \
                 that 'the keyboard path handles resizing'. All four \
                 phrases must appear in order between the two calls."
            )
        });
        cursor += found + phrase.len();
    }
}
