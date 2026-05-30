//! Source-level contract for Liquid Glass guideline assertion buckets.

use std::fs;

#[test]
fn layout_visual_audit_emits_guideline_assertion_buckets() {
    let layout =
        fs::read_to_string("scripts/devtools/layout.ts").expect("failed to read layout.ts");
    for needle in [
        "guidelineAssertions",
        "appleDocumented",
        "projectLocal",
        "buttonCenterDistance",
        "macosMinimumHitSize",
        "macosMinimumVisualSize",
        "materialLayering",
        "colorAdaptivity",
        "cornerRadiusTokens",
        "paddingTokens",
        "spacingTokens",
    ] {
        assert!(layout.contains(needle), "layout.ts must expose {needle}");
    }
}
