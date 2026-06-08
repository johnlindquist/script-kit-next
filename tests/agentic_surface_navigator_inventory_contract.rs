//! Source-level contract for the Surface Navigator inventory audit.

const AUDIT: &str = include_str!("../scripts/agentic/surface-navigator-inventory-audit.ts");
const FILTERABLE_MATRIX: &str = include_str!("../scripts/agentic/filterable-surface-matrix.ts");
const ATTACHED_MATRIX: &str = include_str!("../scripts/agentic/attached-popup-surface-matrix.ts");

#[test]
fn inventory_audit_compares_live_surfaces_to_both_matrices() {
    assert!(AUDIT.contains("SURFACE_NAVIGATOR_EXEMPTIONS"));
    assert!(AUDIT.contains("FILTERABLE_SURFACE_MATRIX"));
    assert!(AUDIT.contains("ATTACHED_POPUP_SURFACE_MATRIX"));
    assert!(AUDIT.contains("automationSemanticSurface"));
}

#[test]
fn inventory_audit_fails_on_missing_stale_or_fallback_surfaces() {
    assert!(AUDIT.contains("is missing from matrices"));
    assert!(AUDIT.contains("no longer appears live"));
    assert!(AUDIT.contains("collector_used_current_view_fallback"));
}

#[test]
fn filterable_matrix_includes_settings_and_kit_store_visible_rows() {
    for case in [
        "settings-visible-rows",
        "kit-store-browse-visible-rows",
        "kit-store-installed-visible-rows",
    ] {
        assert!(
            FILTERABLE_MATRIX.contains(case),
            "missing matrix case {case}"
        );
    }
}

#[test]
fn attached_popup_matrix_still_declares_prompt_popup_family() {
    assert!(ATTACHED_MATRIX.contains("prompt-popup-on-agent_chat-chat-slash"));
    assert!(ATTACHED_MATRIX.contains("expectedAutomationWindowId: \"agent_chat-mention-popup\""));
}
