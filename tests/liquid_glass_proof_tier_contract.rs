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
