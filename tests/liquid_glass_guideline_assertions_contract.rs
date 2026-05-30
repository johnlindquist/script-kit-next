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

/// The `appleDocumented` bucket must encode Apple's documented numeric
/// guidelines as real per-node deviation math, NOT the previous
/// `exactAppleRadiusConstants: null` placeholder. Apple publishes formulas
/// (capsule = height/2, concentric child = parent - inset) and a few hard
/// spacing constants; the engine compares measured nodes against those.
#[test]
fn apple_documented_bucket_emits_numeric_conformance_not_null_placeholder() {
    let layout =
        fs::read_to_string("scripts/devtools/layout.ts").expect("failed to read layout.ts");

    assert!(
        !layout.contains("exactAppleRadiusConstants: null"),
        "appleDocumented must not ship a null radius-constants placeholder; encode Apple's concentric/capsule formulas + deviation math instead"
    );
    assert!(
        layout.contains("appleGuidelineConformance")
            && layout.contains("apple-guideline-constants"),
        "layout.ts must wire the Apple-guideline conformance engine"
    );
    for needle in ["cornerGeometry", "conformanceScore", "backingScaleFactor"] {
        assert!(
            layout.contains(needle),
            "appleDocumented conformance must expose {needle}"
        );
    }
}

/// The Apple-guideline constants table must be provenance-tagged (so each value
/// is defensible) and must encode Apple's actually-documented formulas/constants
/// — capsule radius = height/2, concentric child = parent - inset, the 60pt
/// regular-button center distance and 16pt minimum gap, and the ~12pt bezel
/// padding heuristic — with copyright-safe paraphrased summaries (no copied
/// Apple prose).
#[test]
fn apple_guideline_constants_are_provenance_tagged_and_documented() {
    let constants = fs::read_to_string("scripts/devtools/apple-guideline-constants.ts")
        .expect("failed to read apple-guideline-constants.ts");

    for field in [
        "confidence",
        "normativeStrength",
        "copyrightSafeSummary",
        "appleReference",
        "GuidelineConfidence",
    ] {
        assert!(
            constants.contains(field),
            "guideline constants must carry provenance field {field}"
        );
    }
    for metric in [
        "shape.capsule.radius",
        "shape.concentric.childRadius",
        "layout.regularButton.centerDistance",
        "layout.regularButton.minimumGap",
        "layout.bezelElement.padding",
        "macos.window.toolbarRadius.nativeBaseline",
    ] {
        assert!(
            constants.contains(metric),
            "guideline constants must define Apple metric {metric}"
        );
    }
    // Apple's documented formulas / hard numbers.
    assert!(
        constants.contains("radiusPt = heightPt / 2"),
        "must encode Apple's capsule radius = height/2 formula"
    );
    assert!(
        constants.contains("childRadiusPt = max(0, parentRadiusPt - separationPt)"),
        "must encode Apple's concentric radius formula"
    );
    assert!(
        constants.contains("minPt: 60") && constants.contains("minPt: 16"),
        "must encode Apple's 60pt center-distance and 16pt minimum-gap constants"
    );
    // Deviation classification vocabulary the proof matrix and tests rely on.
    for token in [
        "withinBand",
        "nearBand",
        "outOfBand",
        "unmeasured",
        "deltaPt",
    ] {
        assert!(
            constants.contains(token),
            "conformance engine must classify deviations with {token}"
        );
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
    assert!(
        !layout.contains("REQUIRED_POSITIVE_RADIUS_NODE_NAMES"),
        "layout audit must not hide zero-radius styled nodes behind a hard-coded name whitelist"
    );
}

fn snippet_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"));
    &source[start_index..end_index]
}

/// The corner-radius audit must decide "is this node a rounded surface?" from
/// its node TYPE, not from a hard-coded node-name list. Both audit layers
/// (`layout.ts` cornerRadiusFailures and `liquid-glass-proof.ts`
/// nodesWithMissingPositiveRadius) must share the same type-only predicate so
/// they cannot drift, and Other/text nodes (bare labels, 1px dividers) must
/// stay out of the radius-bearing set.
#[test]
fn corner_radius_audit_is_type_based_not_name_whitelisted() {
    let layout =
        fs::read_to_string("scripts/devtools/layout.ts").expect("failed to read layout.ts");
    let proof = fs::read_to_string("scripts/devtools/liquid-glass-proof.ts")
        .expect("failed to read liquid-glass-proof.ts");

    let layout_predicate = snippet_between(
        &layout,
        "const RADIUS_BEARING_NODE_TYPES",
        "function rectFrom",
    );
    let proof_predicate = snippet_between(
        &proof,
        "const RADIUS_BEARING_NODE_TYPES",
        "function nodesWithMissingPositiveRadius",
    );

    for predicate in [layout_predicate, proof_predicate] {
        assert!(
            predicate.contains("RADIUS_BEARING_NODE_TYPES.has(type)"),
            "radius-bearing classification must be type-based"
        );
        for node_type in [
            "\"area\"",
            "\"button\"",
            "\"card\"",
            "\"container\"",
            "\"header\"",
            "\"input\"",
            "\"list\"",
            "\"listitem\"",
            "\"panel\"",
            "\"prompt\"",
            "\"window\"",
        ] {
            assert!(
                predicate.contains(node_type),
                "radius-bearing type set must include {node_type}"
            );
        }
        assert!(
            !predicate.contains("\"other\"") && !predicate.contains("\"text\""),
            "Other/Text nodes must not be radius-bearing surfaces"
        );
        assert!(
            !predicate.contains("KitStore")
                && !predicate.contains("GenericFilterable")
                && !predicate.contains("REQUIRED_POSITIVE_RADIUS_NODE_NAMES"),
            "radius audit must not use surface-name carveouts"
        );
        assert!(
            !predicate.contains("/Area|Content|Panel"),
            "radius audit should not use the old name-regex heuristic"
        );
    }

    assert!(
        layout.contains("if (!isRadiusBearingNode(node)) return false;"),
        "layout.ts cornerRadiusFailures must skip only non-radius-bearing node types"
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

#[test]
fn creation_feedback_content_nodes_use_liquid_glass_panel_radius() {
    let layout = fs::read_to_string("src/app_layout/build_layout_info.rs")
        .expect("failed to read build_layout_info.rs");

    for node in [
        "CreationFeedbackIntro",
        "CreationFeedbackPathSection",
        "CreationFeedbackActions",
    ] {
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
    }
}

#[test]
fn actions_list_layout_node_uses_liquid_glass_panel_radius() {
    let dialog =
        fs::read_to_string("src/actions/dialog.rs").expect("failed to read actions/dialog.rs");

    let start = dialog
        .find("\"ActionsList\"")
        .expect("ActionsList layout node should exist");
    let node_source = &dialog[start..];
    let end = node_source
        .find(".with_visual_token")
        .expect("ActionsList should declare visual metadata");
    assert!(
        node_source[..end].contains("Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX)"),
        "ActionsList must use the shared Liquid Glass panel radius token"
    );
    assert!(
        !node_source[..end].contains("Some(0.0)"),
        "ActionsList must not satisfy guideline proof with a zero-radius placeholder"
    );
}
