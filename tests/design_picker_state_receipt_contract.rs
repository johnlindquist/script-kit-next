//! Phase 4 — Design Picker state-receipt contract.
//!
//! Source-audit that pins the `design.*` automation receipt fields
//! exposed via `getState`/`kit/state`. The agentic matrix script and
//! the Cmd+1 cycle path both depend on these fields existing under the
//! `design` envelope inside `Message::StateResult`.

use std::fs;

#[test]
// doc-anchor-removed: [[verification#Design Picker persistence]]
fn state_receipt_exposes_design_persistence_fields() {
    let variants = fs::read_to_string("src/protocol/message/variants/query_ops.rs")
        .expect("query_ops variants module must be readable");
    assert!(
        variants.contains("rename = \"design\""),
        "StateResult must declare a `design` envelope (camelCase rename)"
    );

    let receipt = fs::read_to_string("src/render_builtins/design_picker.rs")
        .expect("design_picker.rs must be readable");
    assert!(
        receipt.contains("fn design_state_receipt"),
        "design_state_receipt helper must exist"
    );
    for field in [
        "activeId",
        "persistedActiveId",
        "fallbackApplied",
        "currentVariant",
    ] {
        assert!(
            receipt.contains(field),
            "design_state_receipt must emit `{field}`"
        );
    }

    let handler = fs::read_to_string("src/prompt_handler/mod.rs")
        .expect("prompt_handler/mod.rs must be readable");
    assert!(
        handler.contains("design_state_receipt()"),
        "GetState path must populate the design receipt"
    );
}

#[test]
// doc-anchor-removed: [[verification#Design Picker persistence]]
fn state_result_constructor_carries_design_arg() {
    let constructors = fs::read_to_string("src/protocol/message/constructors/query_ops.rs")
        .expect("query_ops constructors must be readable");
    let start = constructors
        .find("pub fn state_result(")
        .expect("state_result constructor must exist");
    let end = constructors[start..]
        .find("Message::StateResult")
        .expect("state_result constructor body must reference Message::StateResult");
    let signature = &constructors[start..start + end];
    assert!(
        signature.contains("design: Option<serde_json::Value>"),
        "state_result constructor must accept `design: Option<serde_json::Value>`"
    );
}
