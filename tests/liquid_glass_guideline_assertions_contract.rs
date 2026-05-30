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

#[test]
fn layout_visual_audit_rejects_zero_radius_placeholders() {
    let layout =
        fs::read_to_string("scripts/devtools/layout.ts").expect("failed to read layout.ts");
    let proof = fs::read_to_string("scripts/devtools/liquid-glass-proof.ts")
        .expect("failed to read liquid-glass-proof.ts");
    assert!(
        layout.contains("function hasPositiveRadius"),
        "layout.ts must validate radius values, not only radius field presence"
    );
    assert!(
        layout.contains("entry > 0") && layout.contains("value > 0"),
        "layout.ts must reject zero-radius placeholders in guideline assertions"
    );
    assert!(
        layout.contains("!hasPositiveRadius(style.cornerRadius)")
            && layout.contains("!hasPositiveRadius(style.radius)"),
        "cornerRadiusTokens failures must be based on positive Liquid Glass radii"
    );
    assert!(
        !proof.contains("REQUIRED_POSITIVE_RADIUS_NODE_NAMES"),
        "proof matrix must not hide zero-radius styled nodes behind a hard-coded name whitelist"
    );
}

#[test]
fn shared_prompt_layout_nodes_use_liquid_glass_radius_tokens() {
    let layout = fs::read_to_string("src/app_layout/build_layout_info.rs")
        .expect("failed to read build_layout_info.rs");

    for node in ["ContentArea", "ScriptList", "PreviewPanel"] {
        let start = layout
            .find(&format!("LayoutComponentInfo::new(\"{node}\""))
            .unwrap_or_else(|| panic!("{node} layout node should exist"));
        let node_source = &layout[start..];
        let end = node_source
            .find(".with_visual_token")
            .unwrap_or_else(|| panic!("{node} should declare visual metadata"));
        assert!(
            node_source[..end].contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
            "{node} must use the shared Liquid Glass panel radius token"
        );
        assert!(
            !node_source[..end].contains("Some(0.0)"),
            "{node} must not satisfy guideline proof with a zero-radius placeholder"
        );
    }

    for node in [
        "DivContent",
        "SelectChoices",
        "EnvPromptContent",
        "TerminalContent",
    ] {
        let start = layout
            .find(&format!("\"{node}\""))
            .unwrap_or_else(|| panic!("{node} layout node should exist"));
        let node_source = &layout[start..];
        let end = node_source
            .find("return LayoutInfo")
            .unwrap_or_else(|| panic!("{node} should return after visual metadata"));
        assert!(
            node_source[..end].contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
            "{node} must use the shared Liquid Glass panel radius token"
        );
        assert!(
            !node_source[..end].contains("Some(0.0)"),
            "{node} must not satisfy guideline proof with a zero-radius placeholder"
        );
    }
}
