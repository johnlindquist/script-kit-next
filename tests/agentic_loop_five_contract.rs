//! Source-level contract for fifth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const TARGET_THREAD: &str = include_str!("../scripts/agentic/target-thread.ts");

#[test]
fn index_help_exposes_loop_five_recipes() {
    for name in [
        "clipboard-history-portal-range-stress",
        "browser-tabs-cache-identity-stress",
        "scroll-selection-reanchor-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
    }
}

#[test]
fn clipboard_portal_stress_pins_range_and_roundtrip_receipts() {
    for token in [
        "clipboard-history-portal-range-stress",
        "missing_clipboard_portal_range_receipt",
        "clipboardPortal",
        "kit://clipboard-history?id=agentic",
        "exactRangeReplacement",
        "hostRefusalReceipt",
        "wrongHostAccepted",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Clipboard portal range stress must pin {token}"
        );
    }
}

#[test]
fn browser_cache_stress_pins_cache_only_identity() {
    for token in [
        "browser-tabs-cache-identity-stress",
        "missing_browser_cache_identity_receipt",
        "browserCache",
        "cacheOnly: true",
        "browserActivated: false",
        "dedupeKey",
        "staleCacheRejected",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Browser cache identity stress must pin {token}"
        );
    }
}

#[test]
fn scroll_reanchor_stress_pins_visible_selection_receipts() {
    for token in [
        "scroll-selection-reanchor-stress",
        "missing_scroll_selection_reanchor_receipt",
        "scrollSelection",
        "afterWheelSelectedSemanticId",
        "afterDragSelectedSemanticId",
        "visibleRowStillSelected",
        "footerOcclusionSafe",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "Scroll selection reanchor stress must pin {token}"
        );
    }
}
