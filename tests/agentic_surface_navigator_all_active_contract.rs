//! Source-level contract for the combined active image-library sweep.
//!
//! The all-active group runs every currently promoted navigator matrix without
//! promoting candidate surfaces or weakening per-family dispatch.

const NAVIGATOR: &str = include_str!("../scripts/agentic/surface-navigator.ts");

#[test]
fn all_active_group_combines_active_matrices() {
    assert!(
        NAVIGATOR.contains("| \"all-active\""),
        "surface navigator must define the all-active group"
    );
    assert!(
        NAVIGATOR.contains("opts.group === \"all-active\"")
            && NAVIGATOR.contains("...selectedCases(\"all\").map")
            && NAVIGATOR.contains("...selectedAttachedPopupCases(\"all\").map"),
        "all-active selection must combine active filterable-main and attached-popup matrices"
    );
    assert!(
        !NAVIGATOR.contains("PROMPT_POPUP_FIXTURE_MATRIX")
            && !NAVIGATOR.contains("prompt-popup-on-agent_chat-chat-slash-candidate"),
        "all-active must not introduce candidate matrix selection"
    );
}

#[test]
fn all_active_entries_carry_source_group() {
    assert!(
        NAVIGATOR.contains("interface SelectedNavigatorCase")
            && NAVIGATOR.contains("sourceGroup: SourceSurfaceGroup")
            && NAVIGATOR.contains("sourceGroup: receipt.sourceGroup")
            && NAVIGATOR.contains("sourceGroup: selected.sourceGroup"),
        "all-active list, receipts, and manifests must identify each case source group"
    );
}

#[test]
fn all_active_dispatch_routes_by_source_group() {
    let dispatch = NAVIGATOR
        .find("async function runNavigatorCase")
        .expect("navigator must define case dispatch");
    let dispatch_source = &NAVIGATOR[dispatch..];
    let main_start = dispatch_source
        .find("async function main")
        .expect("navigator must define main after dispatch");
    let dispatch_body = &dispatch_source[..main_start];
    assert!(
        dispatch_body.contains("selected.sourceGroup === \"attached-popup\""),
        "combined dispatch must route by per-case source group"
    );
    assert!(
        !dispatch_body.contains("opts.group === \"attached-popup\""),
        "combined dispatch must not route by global opts.group"
    );
}

#[test]
fn all_active_fresh_sessions_include_source_group() {
    assert!(
        NAVIGATOR.contains("`${opts.session}-${selected.sourceGroup}-${entry.viewName}`"),
        "fresh all-active sweeps must include source group in session names"
    );
}

#[test]
fn all_active_individual_case_selection_fails_closed() {
    assert!(
        NAVIGATOR.contains("--group all-active currently supports --case all only"),
        "all-active must fail closed instead of silently resolving duplicate future ids"
    );
}
