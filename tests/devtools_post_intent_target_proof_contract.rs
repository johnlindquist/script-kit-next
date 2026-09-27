const ACT: &str = include_str!("../scripts/devtools/act.ts");
// targets.ts is the CLI; strict target identity is resolved in the shared lib.
const TARGETS: &str = concat!(
    include_str!("../scripts/devtools/targets.ts"),
    include_str!("../scripts/devtools/lib/target-identity.ts"),
);
const FOCUS: &str = include_str!("../scripts/devtools/focus.ts");
const SCHEMA: &str = include_str!("../scripts/devtools/schema.ts");

#[test]
fn route_changing_act_requires_post_intent_target_proof() {
    for needle in [
        "requiresPostIntentTargetProof",
        "expectedPostTargetForIntent",
        "waitForPostIntentTarget",
        "postIntentTargetProof",
        "agent-chat-route",
        "\"--main\", \"--strict\", \"--surface\", \"AgentChat\"",
        "post-intent target did not resolve",
    ] {
        assert!(
            ACT.contains(needle),
            "act.ts missing post-intent proof marker: {needle}"
        );
    }
}

#[test]
fn agent_chat_route_cannot_be_green_only_because_source_is_live() {
    for needle in [
        "requiresPostIntentTargetProof(args)",
        "postIntentTargetProof",
        "postIntentTargetProof?.classification !== \"ok\"",
        "blocked-by-target-ambiguity",
    ] {
        assert!(
            ACT.contains(needle),
            "act.ts must reject false-green route proof: {needle}"
        );
    }
}

#[test]
fn targets_exposes_strict_surface_mismatch_details() {
    for needle in [
        "strictTargetMismatch",
        "surfaceCandidates",
        "expectedSurfaceKind",
        "actualCandidates",
        "actualValues",
        "mismatchReason",
        "expected-surface-not-found",
        "listedWindow.semanticSurface",
    ] {
        assert!(
            TARGETS.contains(needle),
            "targets.ts missing mismatch diagnostic: {needle}"
        );
    }
}

#[test]
fn focus_preserves_full_target_receipt_on_target_identity_failure() {
    for needle in [
        "receipts:",
        "requestedTarget: targetReceipt.requestedTarget ?? { selector }",
        "target: targetReceipt.resolvedTarget ?? null",
        "transaction: targetReceipt.transaction",
        "target: { classification: targetReceipt.classification ?? null }",
    ] {
        assert!(
            FOCUS.contains(needle),
            "focus.ts must preserve target receipt: {needle}"
        );
    }
}

#[test]
fn schema_names_post_intent_and_strict_mismatch_fields() {
    for needle in [
        "resolvedTarget.strictTargetMismatch",
        "postIntentTargetProof",
    ] {
        assert!(
            SCHEMA.contains(needle),
            "schema.ts missing proof field: {needle}"
        );
    }
}
