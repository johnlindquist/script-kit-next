//! Source-level contract for durable image-library screenshot proofs.
//!
//! A screenshot library entry must include a PNG plus machine-readable proof
//! that the image came from the intended automation window and final state.

const NAVIGATOR: &str = include_str!("../scripts/agentic/surface-navigator.ts");

#[test]
fn navigator_promotes_main_kind_to_exact_target_before_capture() {
    assert!(
        NAVIGATOR.contains("inspectAutomationWindow"),
        "navigator must inspect the runtime window after surface entry"
    );
    assert!(
        NAVIGATOR.contains("\"automationInspectResult\""),
        "exact target promotion must use the typed inspect response"
    );
    assert!(
        NAVIGATOR.contains("targetJson: { type: \"id\""),
        "navigator must promote kind targets to exact id targets"
    );
    assert!(
        NAVIGATOR.contains("osWindowId"),
        "navigator must preserve native osWindowId for strict screenshot routing"
    );
}

#[test]
fn navigator_passes_exact_target_and_capture_window_id_to_verify_shot() {
    assert!(
        NAVIGATOR.contains("\"--target-json\""),
        "strict capture must pass a target json"
    );
    assert!(
        NAVIGATOR.contains("JSON.stringify(resolved.targetJson)"),
        "strict capture must pass the promoted exact target, not the entry kind target"
    );
    assert!(
        NAVIGATOR.contains("\"--capture-window-id\""),
        "strict capture must pass the native capture window id"
    );
    assert!(
        NAVIGATOR.contains("String(resolved.osWindowId)"),
        "strict capture must use the inspected native osWindowId"
    );
    assert!(
        NAVIGATOR.contains("\"--strict-window\""),
        "image-library capture must fail closed on wrong-window ambiguity"
    );
}

#[test]
fn navigator_rechecks_state_and_elements_immediately_before_capture() {
    let pre_capture = NAVIGATOR
        .find("\"pre-capture-state-and-elements\"")
        .expect("missing pre-capture gate");
    let screenshot = NAVIGATOR
        .rfind("\"strict-screenshot\"")
        .expect("missing strict screenshot step");
    assert!(
        pre_capture < screenshot,
        "final state/elements gate must run before screenshot capture"
    );
}

#[test]
fn navigator_writes_sidecar_receipts_and_manifest() {
    assert!(
        NAVIGATOR.contains("writeFileSync"),
        "navigator must persist proof artifacts"
    );
    assert!(
        NAVIGATOR.contains("receiptOutPath("),
        "navigator must write one sidecar receipt per PNG"
    );
    assert!(
        NAVIGATOR.contains("manifestOutPath("),
        "navigator must write the image-library manifest"
    );
    assert!(
        NAVIGATOR.contains("manifest.json"),
        "default manifest path must be .notes/image-library/manifest.json"
    );
    assert!(
        NAVIGATOR.contains("buildManifest("),
        "manifest contents must be built from case receipts"
    );
}

#[test]
fn manifest_carries_capture_identity_and_content_audit() {
    for field in [
        "surfaceClass",
        "sourceGroup",
        "windowKind",
        "captureTarget",
        "popupCapture",
        "contentAudit",
        "resolvedTarget",
        "finalObservation",
        "preCaptureInspection",
        "preCaptureElements",
        "passedCases",
        "failedCases",
    ] {
        assert!(
            NAVIGATOR.contains(field),
            "manifest must carry proof field {field}"
        );
    }
}

#[test]
fn manifest_carries_attached_popup_crop_proof() {
    for token in [
        "parent_capture_with_crop",
        "popupCapture.targetBounds",
        "popupCapture?.strategy",
    ] {
        assert!(
            NAVIGATOR.contains(token),
            "attached-popup manifest path must preserve crop proof token {token}"
        );
    }
}

#[test]
fn manifest_carries_attached_popup_host_fixture_proof() {
    for field in [
        "hostFixture",
        "hostSetup",
        "hostObservation",
        "hostResolvedTarget",
    ] {
        assert!(
            NAVIGATOR.contains(field),
            "manifest must carry hosted attached-popup proof field {field}"
        );
    }
}

#[test]
fn manifest_records_fresh_per_case_sweep_mode() {
    assert!(
        NAVIGATOR.contains("freshPerCase: opts.freshPerCase"),
        "manifest must record whether screenshot cases were isolated"
    );
    assert!(
        NAVIGATOR.contains("`${opts.session}-${selected.sourceGroup}-${entry.viewName}`"),
        "fresh per-case sweeps must include source group in distinct short-lived sessions"
    );
}
