//! Source-level contract for Liquid Glass proof tier accounting.

const PROOF_MATRIX: &str = include_str!("../scripts/devtools/liquid-glass-proof.ts");

#[test]
fn proof_matrix_separates_visual_numeric_and_diff_tiers() {
    assert!(
        PROOF_MATRIX.contains("type ProofTiers")
            && PROOF_MATRIX.contains("osScreenshotProof")
            && PROOF_MATRIX.contains("appRenderProof")
            && PROOF_MATRIX.contains("offscreenRenderProof")
            && PROOF_MATRIX.contains("numericProof")
            && PROOF_MATRIX.contains("guidelineProof")
            && PROOF_MATRIX.contains("imageDiffProof"),
        "Liquid Glass proof matrix must separate OS screenshot, app-render, offscreen, numeric, guideline, and image-diff proof tiers"
    );
}

#[test]
fn proof_matrix_does_not_promote_windowserver_blockers_to_visual_proof() {
    assert!(
        PROOF_MATRIX.contains("visualEvidence.source === \"os-window-capture\"")
            && PROOF_MATRIX.contains("visualEvidence.available === false")
            && PROOF_MATRIX.contains("countsAsOsScreenshotEvidence")
            && PROOF_MATRIX.contains("countsAsAppRenderEvidence")
            && PROOF_MATRIX.contains(
                "proofTiers separate OS screenshots from GPUI app-render proof"
            ),
        "Liquid Glass proof matrix must classify WindowServer screenshot blockers without promoting them to visual proof"
    );
}

#[test]
fn proof_matrix_summary_reports_visual_tier_debt() {
    assert!(
        PROOF_MATRIX.contains("appRenderFailedSurfaceCount")
            && PROOF_MATRIX.contains("appRenderBlockedSurfaceCount")
            && PROOF_MATRIX.contains("appRenderMissingSurfaceCount")
            && PROOF_MATRIX.contains("offscreenRenderFailedSurfaceCount")
            && PROOF_MATRIX.contains("offscreenRenderMissingSurfaceCount")
            && PROOF_MATRIX.contains("guidelineFailedSurfaceCount")
            && PROOF_MATRIX.contains("guidelineMissingSurfaceCount")
            && PROOF_MATRIX.contains("visualTierDebtSurfaceCount")
            && PROOF_MATRIX.contains("explicit visual-tier debt")
            && PROOF_MATRIX.contains("failing Tahoe guideline assertions")
            && PROOF_MATRIX.contains("attempted app-render proof but GPUI render readback was unavailable or unsupported"),
        "Liquid Glass proof matrix summary must expose blocked, failed, or missing visual proof tiers instead of hiding them behind overall surface status"
    );
}

#[test]
fn proof_matrix_guideline_assertions_gate_strong_proof() {
    assert!(
        PROOF_MATRIX.contains("function guidelineProof")
            && PROOF_MATRIX.contains("guidelineAssertionFailureCount")
            && PROOF_MATRIX.contains("numeric-proof-guideline-failed")
            && PROOF_MATRIX.contains("numeric-proof-missing-guideline-assertions")
            && PROOF_MATRIX.contains("tiers.guidelineProof === \"pass\""),
        "Liquid Glass strong-proof must require passing Tahoe guideline assertions"
    );
}

#[test]
fn proof_matrix_classification_fails_when_visual_tier_debt_remains() {
    assert!(
        PROOF_MATRIX.contains(
            "summary.missingProofSurfaceCount === 0 && summary.visualTierDebtSurfaceCount === 0 && summary.surfaceProofDebtCount === 0 ? \"ok\" : \"incomplete\"",
        ),
        "Liquid Glass proof matrix classification must stay incomplete while any contract surface is not strong-proof or explicit visual-tier debt remains"
    );
}

#[test]
fn proof_matrix_lists_surface_and_visual_tier_debt() {
    assert!(
        PROOF_MATRIX.contains("visualTierDebtSurfaces")
            && PROOF_MATRIX.contains("surfaceProofDebtSurfaces")
            && PROOF_MATRIX.contains("surfaceProofDebtCount")
            && PROOF_MATRIX.contains("failedTiers")
            && PROOF_MATRIX.contains("contract surfaces are not yet strong-proof"),
        "Liquid Glass proof matrix must list exact surfaces and tiers that still block exhaustive proof"
    );
}

#[test]
fn proof_matrix_emits_ordered_debt_work_queue() {
    assert!(
        PROOF_MATRIX.contains("proofDebtWorkQueue")
            && PROOF_MATRIX.contains("proofDebtWorkQueueCount")
            && PROOF_MATRIX.contains("OUTSIDE_IN_SURFACE_PRIORITY")
            && PROOF_MATRIX.contains("outsideInPriority")
            && PROOF_MATRIX.contains("priorityGroup")
            && PROOF_MATRIX.contains("window-container")
            && PROOF_MATRIX.contains("nextEvidenceNeeded")
            && PROOF_MATRIX.contains("guidelineProof")
            && PROOF_MATRIX.contains("recommendedNextAction")
            && PROOF_MATRIX.contains("capture-blocker")
            && PROOF_MATRIX.contains("missing-proof-tier"),
        "Liquid Glass proof matrix must emit an outside-in ordered work queue for remaining surface proof debt"
    );
}

#[test]
fn proof_matrix_has_numeric_plus_app_render_status_without_strong_promotion() {
    assert!(
        PROOF_MATRIX.contains("numeric-plus-app-render-proof-os-screenshot-blocked")
            && PROOF_MATRIX.contains("numeric-plus-app-render-proof-missing-os-screenshot")
            && PROOF_MATRIX.contains("numeric-proof-app-render-attempted-failed")
            && PROOF_MATRIX.contains("numeric-proof-app-render-blocked"),
        "app-render proof must create an intermediate status instead of promoting to strong-proof"
    );
    assert!(
        PROOF_MATRIX.contains("function usableAppRenderEvidence")
            && PROOF_MATRIX.contains("countsAsOsScreenshotEvidence === false")
            && PROOF_MATRIX.contains("pixelAudit.blank === false"),
        "app-render evidence must be nonblank and must not count as OS screenshot evidence"
    );
    assert!(
        PROOF_MATRIX.contains("app-render/readback images do not count as OS screenshot evidence"),
        "app-render PNGs must not be counted as OS screenshot artifacts"
    );
}
