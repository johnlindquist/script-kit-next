//! Source-level contract for Rust/Bun screenshot pixel audit parity.

const RUST_CAPTURE: &str = include_str!("../../src/platform/screenshots_window_open.rs");
const VERIFY_SHOT: &str = include_str!("../../scripts/agentic/verify-shot.ts");
const MATRIX: &str = include_str!("../../scripts/agentic/verify-shot-blank-rejection-matrix.ts");

#[test]
fn rust_and_bun_pixel_audits_expose_same_fields() {
    for field in [
        "unique_bucket_count",
        "mean_luma",
        "max_luma",
        "non_black_ratio",
    ] {
        assert!(
            RUST_CAPTURE.contains(field),
            "Rust PixelAudit must expose {field}"
        );
    }

    for field in ["uniqueBucketCount", "meanLuma", "maxLuma", "nonBlackRatio"] {
        assert!(
            VERIFY_SHOT.contains(field) && MATRIX.contains(field),
            "Bun verify-shot and fixture matrix must expose {field}"
        );
    }
}

#[test]
fn rust_and_bun_use_the_same_blank_rejection_thresholds() {
    for literal in [
        "unique_bucket_count <= 1",
        "unique_bucket_count <= 2",
        "mean_luma < 5.0",
        "non_black_ratio < 0.001",
        "max_luma < 16.0",
    ] {
        assert!(
            RUST_CAPTURE.contains(literal),
            "Rust audit must keep threshold literal {literal}"
        );
    }

    for literal in [
        "uniqueBucketCount <= 1",
        "uniqueBucketCount <= 2",
        "meanLuma < 5.0",
        "nonBlackRatio < 0.001",
        "maxLuma < 16.0",
    ] {
        assert!(
            VERIFY_SHOT.contains(literal) && MATRIX.contains(literal),
            "Bun audit and fixture matrix must keep threshold literal {literal}"
        );
    }
}

#[test]
fn fixture_matrix_covers_blank_solid_and_valid_dark_cases() {
    for case in [
        "transparent",
        "solid-black",
        "solid-white",
        "solid-gray",
        "valid-dark-ui",
    ] {
        assert!(MATRIX.contains(case), "fixture matrix must include {case}");
    }
}
