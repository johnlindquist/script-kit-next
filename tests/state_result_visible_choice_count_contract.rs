//! Source-level contract for the Run 2 Pass #33
//! `tool-state-result-visible-choice-count-contract` user story.
//!
//! Pass #14 (Run 1) surfaced that the `empty-clipboard-state` story's
//! acceptance clause `getState.choiceCount=0` was structurally
//! unreachable: `choiceCount` reports the total dataset (e.g. 100
//! clipboard items) regardless of the active filter, so an empty
//! filter result can never drive `choiceCount` to zero. The correct
//! field is `visibleChoiceCount`, which reflects the view's actual
//! rendered count post-filter.
//!
//! `visibleChoiceCount` already exists on `StateResult` at the
//! protocol layer (see `src/protocol/message/variants/query_ops.rs`
//! — the field is declared between `choice_count` and
//! `selected_index`), but nothing in the source-level test suite pins
//! (a) its existence, (b) its camelCase JSON rename, or (c) its
//! distinct semantics from `choice_count`. A mechanical refactor —
//! or a well-intentioned "cleanup" that merges the two fields —
//! could silently break every automation harness that currently
//! keys on `visibleChoiceCount` to detect filter-empty states.
//!
//! This contract test pins all three invariants so the protocol
//! surface for filter-aware counts is stable across refactors,
//! closing Pass #14 sub-gap (3) of `empty-clipboard-state` at the
//! protocol-contract level.

const QUERY_OPS_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const QUERY_OPS_CONSTRUCTORS: &str =
    include_str!("../src/protocol/message/constructors/query_ops.rs");

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn state_result_variant_declares_choice_count_and_visible_choice_count() {
    // Both fields must exist side-by-side in the enum variant with
    // the exact field names `choice_count` and `visible_choice_count`.
    // Merging them into a single field (e.g. dropping `choice_count`
    // and keeping only the filter-aware value) would regress every
    // automation harness that still reports both for diagnostic
    // purposes — the story's acceptance receipt needs BOTH fields
    // to prove filter was applied (total > 0, visible == 0).
    assert!(
        QUERY_OPS_VARIANTS.contains("choice_count: usize,"),
        "src/protocol/message/variants/query_ops.rs `StateResult` variant \
         must declare `choice_count: usize,` (the total dataset count, \
         unaffected by filter). This field is required so automation \
         can distinguish \"dataset empty\" (both zero) from \"filter \
         matched nothing\" (total > 0, visible == 0)."
    );
    assert!(
        QUERY_OPS_VARIANTS.contains("visible_choice_count: usize,"),
        "src/protocol/message/variants/query_ops.rs `StateResult` variant \
         must declare `visible_choice_count: usize,` (the filter-aware \
         count that reflects the view's actual rendered choices). This \
         is the field automation harnesses MUST use to verify \
         `empty-filter-result` style acceptance clauses — `choice_count` \
         alone is structurally wrong for that purpose."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn state_result_renames_both_count_fields_to_camel_case_json() {
    // Automation clients consume the stateResult JSON — the Rust
    // field names are implementation detail but the JSON field names
    // are the actual protocol contract. A rename on either side
    // silently breaks clients.
    assert!(
        QUERY_OPS_VARIANTS.contains("#[serde(rename = \"choiceCount\")]"),
        "src/protocol/message/variants/query_ops.rs `StateResult.choice_count` \
         must be renamed to JSON field `\"choiceCount\"` (camelCase). The \
         agentic-testing harness keys on this exact JSON path; a rename \
         would invalidate every automation fixture without a compile-time \
         signal."
    );
    assert!(
        QUERY_OPS_VARIANTS.contains("#[serde(rename = \"visibleChoiceCount\")]"),
        "src/protocol/message/variants/query_ops.rs `StateResult.visible_choice_count` \
         must be renamed to JSON field `\"visibleChoiceCount\"` (camelCase). \
         Same contract logic as `choiceCount` — this is the JSON path \
         the empty-clipboard-state story's corrected acceptance clause \
         keys on."
    );
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn state_result_constructor_accepts_both_counts_as_distinct_args() {
    // `Message::state_result(...)` is the single choke-point where
    // the two counts are populated. Collapsing them into a single
    // argument would let a future refactor silently set
    // `visible_choice_count = choice_count` (erasing filter info)
    // without a type error.
    assert!(
        QUERY_OPS_CONSTRUCTORS.contains("choice_count: usize,"),
        "src/protocol/message/constructors/query_ops.rs `Message::state_result` \
         must accept `choice_count: usize` as a distinct parameter. Merging \
         into `visible_choice_count` only would hide filter information \
         from every automation caller."
    );
    assert!(
        QUERY_OPS_CONSTRUCTORS.contains("visible_choice_count: usize,"),
        "src/protocol/message/constructors/query_ops.rs `Message::state_result` \
         must accept `visible_choice_count: usize` as a distinct parameter. \
         This is the field the `empty-clipboard-state` story's \
         acceptance clause keys on — removing it from the constructor \
         signature forces a downgrade to `choice_count`-only, which \
         the Pass #14 gap report proved is structurally unreachable."
    );
    assert!(
        QUERY_OPS_CONSTRUCTORS.contains("            visible_choice_count,"),
        "src/protocol/message/constructors/query_ops.rs `Message::state_result` \
         must forward the `visible_choice_count` parameter into the \
         `Message::StateResult` struct literal. A regression that \
         drops this line would silently zero the field for every \
         automation query."
    );
}
