use std::fs;

fn read_scroll() -> String {
    fs::read_to_string("scripts/devtools/scroll.ts")
        .expect("failed to read scripts/devtools/scroll.ts")
}

#[test]
fn scroll_receipt_preserves_raw_and_effective_viewport_measurements() {
    let source = read_scroll();
    assert!(
        source.contains("listStateViewportHeight"),
        "scroll receipts must preserve raw GPUI ListState viewport height"
    );
    assert!(
        source.contains("effectiveViewportHeight"),
        "scroll receipts must expose an effective viewport height for proof"
    );
    assert!(
        source.contains("viewportMeasurementSource"),
        "scroll receipts must identify whether measurement came from listState or layout"
    );
    assert!(
        source.contains("listStateViewportUnmeasured"),
        "scroll receipts must explicitly warn when GPUI ListState viewport is zero"
    );
}

#[test]
fn scroll_receipt_does_not_mark_selection_invisible_from_unmeasured_viewport() {
    let source = read_scroll();
    assert!(
        source.contains("selectedRowBounds") || source.contains("missingPrimitive"),
        "when ListState viewport is zero, scroll proof must use selected-row bounds or fail closed"
    );
    assert!(
        source.contains("blocked-by-missing-primitive")
            || source.contains("measurement unavailable"),
        "scroll proof must not turn missing row geometry into a false hidden-row proof"
    );
}
