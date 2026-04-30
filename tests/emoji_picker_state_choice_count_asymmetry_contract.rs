//! Source-level contract for the Run 9 Pass #6 Fix of
//! `fix-emojipicker-state-choicecount-collapses-with-filter`.
//!
//! Pre-fix, the `AppView::EmojiPickerView` arm of `collect_state`
//! (`src/prompt_handler/mod.rs`) computed a single `filtered_count`
//! from `crate::emoji::search_emojis(filter)` and returned it in
//! BOTH the `choice_count` and `visible_choice_count` slots of the
//! state tuple. Live receipts on session pid 75585 confirmed the
//! collapse: `triggerBuiltin emoji` + `getState` → `choiceCount:296,
//! visibleChoiceCount:296` (correct, empty filter), but `setFilter
//! "heart"` + `getState` → `choiceCount:24, visibleChoiceCount:24`
//! — both shrank with the filter, violating the Run 9 Pass #2
//! contract pinned in `lat.md/protocol.md` §"Query and introspection":
//!
//!   > `stateResult` carries both `choiceCount` (total dataset) and
//!   > `visibleChoiceCount` (filter-aware). Automation harnesses
//!   > verifying empty-filter-result acceptance clauses must key on
//!   > `visibleChoiceCount`, never `choiceCount` — the latter is
//!   > unchanged by the active filter and so cannot drop to zero on
//!   > a filter miss.
//!
//! The fix computes two distinct counts directly in the arm:
//!   - `dataset_count` iterates over `crate::emoji::EMOJIS` filtered
//!     only by the active `selected_category` (independent of the
//!     text filter). This is the "total dataset" under the Pass #2
//!     invariant.
//!   - `visible_count` is the prior `search_emojis(filter)` +
//!     `selected_category` count — the narrowing the user sees.
//!
//! Live post-fix receipts (session pid 75585, after `cargo build`
//! hot-swap):
//!   - setFilter ""       → choiceCount:296, visibleChoiceCount:296
//!   - setFilter "heart"  → choiceCount:296, visibleChoiceCount:24
//!
//! This contract pins the arm shape so a future "helpful" refactor
//! cannot silently reintroduce the collapse. The refactor threat is
//! concrete: a contributor consolidating the two iterators into one
//! via a single `search_emojis_within_category` helper might cache
//! the call and return the same number for both slots, OR replace
//! both slots with `filtered_count` to "simplify" the arm.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");

/// Returns the byte range of the `AppView::EmojiPickerView` arm
/// inside `collect_state`. End is the next sibling `AppView` arm.
fn emoji_picker_state_arm() -> &'static str {
    let start = PROMPT_HANDLER
        .find("AppView::EmojiPickerView {\n                        filter,")
        .expect(
            "src/prompt_handler/mod.rs must contain an \
             `AppView::EmojiPickerView` arm inside `collect_state` \
             destructuring `filter,` on the line after the opening \
             brace. Any other shape (binding via `..` or reading \
             `self.filter_text`) would break the Run 9 Pass #6 Fix \
             that pins `choiceCount` to the unfiltered dataset size.",
        );
    // The next sibling arm is `AppView::WebcamView`. If that ordering
    // ever changes, update this end anchor.
    let end_rel = PROMPT_HANDLER[start..]
        .find("\n                    AppView::WebcamView")
        .expect(
            "`AppView::EmojiPickerView` state arm must be immediately \
             followed by `AppView::WebcamView` — sibling-variant \
             reorder must update this contract's end anchor.",
        );
    &PROMPT_HANDLER[start..start + end_rel]
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn emoji_picker_state_arm_derives_dataset_count_from_unfiltered_emojis() {
    // The `choice_count` slot MUST be populated from a `dataset_count`
    // binding that iterates over `crate::emoji::EMOJIS` and filters
    // ONLY by `selected_category` — not by `filter`. Otherwise the
    // Pass #2 invariant "choiceCount is unchanged by the active
    // filter" is violated: `setFilter "heart"` would drop `choiceCount`
    // from 296 to ~24, exactly the pre-fix bug.
    let body = emoji_picker_state_arm();
    assert!(
        body.contains("let dataset_count = crate::emoji::EMOJIS"),
        "EmojiPickerView state arm must bind `dataset_count` from \
         `crate::emoji::EMOJIS` directly. Any `search_emojis(filter)` \
         call in the dataset-count binding would drift `choiceCount` \
         with the text filter — the pre-Pass-#6 bug shape."
    );
    assert!(
        !body.contains("let dataset_count = crate::emoji::search_emojis("),
        "EmojiPickerView state arm must NOT derive `dataset_count` \
         from `search_emojis(filter)` — that is the filtered count \
         and belongs in the `visible_count` slot only."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn emoji_picker_state_arm_derives_visible_count_from_search_emojis() {
    // The `visible_choice_count` slot MUST be populated from a
    // `visible_count` binding that calls `search_emojis(filter)` and
    // then applies the same `selected_category` narrowing as the
    // dataset count. Losing the `search_emojis(filter)` call here
    // would silently make `visibleChoiceCount` match the dataset
    // size, collapsing the asymmetry in the other direction.
    let body = emoji_picker_state_arm();
    assert!(
        body.contains("let visible_count = crate::emoji::search_emojis(filter)"),
        "EmojiPickerView state arm must bind `visible_count` via \
         `crate::emoji::search_emojis(filter)`. This is the single \
         filter-aware accessor and matches the shape of the \
         collect_elements arm in `src/app_layout/collect_elements.rs`."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn emoji_picker_state_arm_tuple_slots_are_dataset_then_visible() {
    // The state tuple slots for EmojiPicker are, in order:
    //   ("emojiPicker".to_string(), Some("emoji-picker".to_string()),
    //    None, filter.clone(), <choice_count>, <visible_choice_count>,
    //    *selected_index as i32, None)
    // The Pass #6 Fix pins `dataset_count` into slot 5 (choice_count)
    // and `visible_count` into slot 6 (visible_choice_count). A
    // refactor that swaps them would flip the Pass #2 invariant and
    // silently tell callers their filter widened the dataset.
    let body = emoji_picker_state_arm();
    // Look for the contiguous substring; whitespace-insensitive match
    // is overkill — the arm is a literal tuple construction.
    assert!(
        body.contains("filter.clone(),\n                            dataset_count,\n                            visible_count,"),
        "EmojiPickerView state arm must construct the tuple with \
         `filter.clone(), dataset_count, visible_count,` contiguous \
         in that order. Any other slot order would silently break \
         the Pass #2 invariant `visibleChoiceCount <= choiceCount` \
         or collapse it back to the pre-fix single-count shape."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn emoji_picker_state_arm_forbids_single_count_tuple_shape() {
    // The pre-fix shape was a single `filtered_count` binding
    // appearing TWICE in the tuple (slots 5 and 6 both reading
    // `filtered_count`). Pin that specific regression: the arm
    // must not contain `filtered_count,\n...filtered_count,`.
    let body = emoji_picker_state_arm();
    assert!(
        !body.contains("filtered_count,\n                            filtered_count,"),
        "EmojiPickerView state arm must not re-use a single \
         `filtered_count` binding in both tuple slots — that is \
         the pre-Pass-#6 bug shape where `choiceCount` drifted \
         with the text filter. Use distinct `dataset_count` and \
         `visible_count` bindings."
    );
}
