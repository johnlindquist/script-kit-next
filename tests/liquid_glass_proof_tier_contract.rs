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
            && PROOF_MATRIX.contains("imageDiffProof"),
        "Liquid Glass proof matrix must separate OS screenshot, app-render, offscreen, numeric, and image-diff proof tiers"
    );
}

#[test]
fn proof_matrix_does_not_promote_windowserver_blockers_to_visual_proof() {
    assert!(
        PROOF_MATRIX.contains("macos-windowserver-capture-blocked")
            && PROOF_MATRIX.contains("countsAsOsScreenshotEvidence")
            && PROOF_MATRIX.contains("countsAsAppRenderEvidence")
            && PROOF_MATRIX.contains(
                "WindowServer-blocked captures cannot become false visual evidence"
            ),
        "Liquid Glass proof matrix must classify WindowServer screenshot blockers without promoting them to visual proof"
    );
}

#[test]
fn proof_matrix_summary_reports_visual_tier_debt() {
    assert!(
        PROOF_MATRIX.contains("appRenderFailedSurfaceCount")
            && PROOF_MATRIX.contains("appRenderMissingSurfaceCount")
            && PROOF_MATRIX.contains("offscreenRenderFailedSurfaceCount")
            && PROOF_MATRIX.contains("offscreenRenderMissingSurfaceCount")
            && PROOF_MATRIX.contains("visualTierDebtSurfaceCount")
            && PROOF_MATRIX.contains("explicit visual-tier debt")
            && PROOF_MATRIX.contains("attempted app-render proof and failed or returned unsupported"),
        "Liquid Glass proof matrix summary must expose failed or missing visual proof tiers instead of hiding them behind overall surface status"
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
            && PROOF_MATRIX.contains("nextEvidenceNeeded")
            && PROOF_MATRIX.contains("recommendedNextAction")
            && PROOF_MATRIX.contains("capture-blocker")
            && PROOF_MATRIX.contains("missing-proof-tier"),
        "Liquid Glass proof matrix must emit an ordered work queue for remaining surface proof debt"
    );
}
