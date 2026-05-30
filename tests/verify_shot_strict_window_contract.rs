//! Source-level contract for strict non-ACP screenshot capture.

const VERIFY_SHOT: &str = include_str!("../scripts/agentic/verify-shot.ts");

#[test]
fn verify_shot_parses_strict_window_flag() {
    assert!(
        VERIFY_SHOT.contains("arg === \"--strict-window\""),
        "verify-shot must parse --strict-window as a boolean flag"
    );
    assert!(
        VERIFY_SHOT.contains("opts.strictWindow = true"),
        "--strict-window must set opts.strictWindow"
    );
}

#[test]
fn strict_window_extends_acp_identity_proof() {
    assert!(
        VERIFY_SHOT.contains(
            "const strictWindowProof = opts.strictWindow === true || hasAcpAssertions(opts);"
        ),
        "strict window proof must apply to general captures, not only ACP assertions"
    );
}

#[test]
fn strict_window_requires_exact_target_or_window_id() {
    assert!(
        VERIFY_SHOT
            .contains("opts.strictWindow === true && !targetJson && captureWindowId == null"),
        "strict mode must reject captures without target-json or capture-window-id"
    );
    assert!(
        VERIFY_SHOT.contains("Strict window capture requires --target-json or --capture-window-id"),
        "strict rejection must explain the missing exact target"
    );
}

#[test]
fn strict_window_cannot_use_runtime_capture_fallback() {
    let strict_expr = VERIFY_SHOT
        .find("const strictWindowProof = opts.strictWindow === true || hasAcpAssertions(opts);")
        .expect("strictWindowProof expression must exist");
    let fallback = VERIFY_SHOT
        .find("captureRouting = \"runtime-capture-window\"")
        .expect("runtime fallback branch must exist for non-strict captures");
    assert!(
        fallback > strict_expr,
        "runtime fallback must be gated by the strictWindowProof value"
    );
    assert!(
        VERIFY_SHOT.contains("} else if (!strictWindowProof) {"),
        "runtime capture fallback must only run when strictWindowProof is false"
    );
}

#[test]
fn strict_window_prefers_exact_native_mcp_capture_before_screencapture() {
    let native_route = VERIFY_SHOT
        .find("captureNativeWindowViaMcp")
        .expect("verify-shot must have a native MCP capture helper");
    let window_helper = VERIFY_SHOT
        .find("scripts/agentic/window.ts")
        .expect("verify-shot must keep the window.ts fallback");

    assert!(
        native_route < window_helper,
        "exact native MCP capture should be attempted before the screencapture/window.ts path"
    );
    assert!(
        VERIFY_SHOT.contains("computer/capture_native_window")
            && VERIFY_SHOT.contains("native-window-mcp")
            && VERIFY_SHOT.contains("native-xcap"),
        "native capture receipts must preserve route and method identity"
    );
}

#[test]
fn strict_window_keeps_blank_png_audit_as_infra_failure() {
    assert!(
        VERIFY_SHOT.contains("contentAudit.blank"),
        "PNG content audit must still inspect blank screenshots"
    );
    assert!(
        VERIFY_SHOT.contains("Screenshot pixel audit rejected a blank/black image"),
        "blank or black screenshots must remain infrastructure failures"
    );
}

#[test]
fn strict_window_records_structured_visual_evidence_tiers() {
    assert!(
        VERIFY_SHOT.contains("interface CaptureAttemptReceipt")
            && VERIFY_SHOT.contains("interface VisualEvidenceReceipt")
            && VERIFY_SHOT.contains("countsAsOsScreenshotEvidence")
            && VERIFY_SHOT.contains("countsAsAppRenderEvidence"),
        "verify-shot must expose machine-readable visual evidence tiers instead of only error strings"
    );
    assert!(
        VERIFY_SHOT.contains("macos-windowserver-capture-blocked")
            && VERIFY_SHOT.contains("blank-image-rejected")
            && VERIFY_SHOT.contains("gpui-readback-unavailable"),
        "visual evidence tiers must distinguish OS capture blockers from app-render readback blockers"
    );
}

#[test]
fn auto_visual_source_attempts_render_readback_after_os_blocker() {
    let os_attempt = VERIFY_SHOT
        .find("buildOsVisualEvidence(screenshotResult, capturePlan)")
        .expect("OS visual evidence classification must be built");
    let render_attempt = VERIFY_SHOT
        .find("captureRenderReadbackViaMcp(targetJson, renderOutPath, inspection)")
        .expect("render readback fallback must exist");
    assert!(
        os_attempt < render_attempt,
        "auto visual-source must classify OS capture before attempting app-render readback"
    );
    assert!(
        VERIFY_SHOT.contains("--visual-source")
            && VERIFY_SHOT.contains("computer/capture_render_window")
            && VERIFY_SHOT.contains("gpui-render-readback"),
        "verify-shot must expose the render-readback proof path"
    );
}

#[test]
fn render_readback_is_not_os_screenshot_proof() {
    assert!(
        VERIFY_SHOT.contains("countsAsOsScreenshotEvidence: false")
            && VERIFY_SHOT.contains("countsAsAppRenderEvidence: captured")
            && VERIFY_SHOT.contains("App-rendered GPUI pixels only"),
        "app-render readback must not be counted as OS screenshot/native compositor proof"
    );
    assert!(
        VERIFY_SHOT.contains("capture.errorCode")
            && VERIFY_SHOT.contains("runtime_unavailable")
            && VERIFY_SHOT.contains("unknown_tool"),
        "render readback receipts must preserve top-level tool errors as unsupported proof, not null failures"
    );
}

#[test]
fn strict_window_has_target_bound_screen_rect_fallback() {
    assert!(
        VERIFY_SHOT.contains("\"screen-rect-from-inspection\""),
        "strict fallback route must be explicitly receipted"
    );
    assert!(
        VERIFY_SHOT.contains("resolvedBounds"),
        "inspection must preserve screen-space resolvedBounds for target-bound fallback"
    );
    assert!(
        VERIFY_SHOT.contains("captureScreenRectFromInspection"),
        "verify-shot must isolate screen-rect fallback behind inspection geometry"
    );
    assert!(
        VERIFY_SHOT.contains("\"screencapture\"")
            && VERIFY_SHOT.contains("\"-R\"")
            && VERIFY_SHOT.contains("strict-window-helper-failed")
            && VERIFY_SHOT.contains("blank-primary-capture"),
        "screen-rect fallback must use screencapture -R and receipt both failure paths"
    );
    assert!(
        VERIFY_SHOT.contains("captureRouting !== \"screen-rect-from-inspection\""),
        "blank primary captures should attempt the screen-rect fallback only once"
    );
}

#[test]
fn strict_screen_rect_fallback_uses_native_window_bounds_before_automation_bounds() {
    assert!(
        VERIFY_SHOT.contains("computer/get_native_window")
            && VERIFY_SHOT.contains("computer/list_native_windows.bounds"),
        "strict screen-rect fallback must resolve CoreGraphics/native bounds before automation bounds"
    );
    assert!(
        VERIFY_SHOT.contains("automationResolvedBounds"),
        "fallback receipt must preserve automation bounds for coordinate-space audits"
    );
    assert!(
        VERIFY_SHOT.contains("screenRectFallback"),
        "verify-shot receipts must expose fallback source, bounds, native window id, and errors"
    );
}

#[test]
fn strict_window_resolves_ambiguous_target_by_inspection_pid() {
    assert!(
        VERIFY_SHOT.contains("resolveNativeWindowIdFromInspection"),
        "ambiguous target screenshots should recover a native window id from inspectAutomationWindow pid"
    );
    assert!(
        VERIFY_SHOT.contains("inspection.pid"),
        "inspection pid must be preserved for native-window disambiguation"
    );
    assert!(
        VERIFY_SHOT.contains("computer/list_native_windows")
            && VERIFY_SHOT.contains("app.pid !== inspection.pid")
            && VERIFY_SHOT.contains("captureSelectionCandidate?.status === \"candidate\""),
        "pid-based fallback must query native windows, filter by owner pid, and prefer capture candidates"
    );
    assert!(
        VERIFY_SHOT.contains("verify_shot_native_window_resolved_from_pid"),
        "pid-based native-window recovery must emit a diagnostic receipt"
    );
}

#[test]
fn strict_window_skips_show_for_attached_popup_exact_capture() {
    assert!(
        VERIFY_SHOT.contains("function isAttachedPopupWindowKind"),
        "verify-shot must classify attached popup window kinds"
    );
    assert!(
        VERIFY_SHOT.contains("ActionsDialog") && VERIFY_SHOT.contains("PromptPopup"),
        "attached popup handling must include ActionsDialog and PromptPopup kinds"
    );
    assert!(
        VERIFY_SHOT.contains("const skipShowForAttachedPopup"),
        "attached popup exact captures must explicitly decide whether to skip show"
    );
    assert!(
        VERIFY_SHOT.contains("if (!skipShowForAttachedPopup)"),
        "show before capture must be bypassed for already-inspected attached popups"
    );
}

#[test]
fn popup_missing_crop_bounds_is_top_level_infra_error() {
    let popup_error = VERIFY_SHOT
        .find("let popupCaptureInfraError = false;")
        .expect("popupCaptureInfraError must be declared");
    let has_infra = VERIFY_SHOT
        .find("const hasInfraError =")
        .expect("hasInfraError must be computed");
    assert!(
        popup_error < has_infra,
        "popup crop validation must happen before top-level status is computed"
    );
    assert!(
        VERIFY_SHOT.contains("popupCaptureInfraError = true;"),
        "missing attached popup crop bounds must set popupCaptureInfraError"
    );
    assert!(
        VERIFY_SHOT.contains("popupCaptureInfraError;"),
        "top-level hasInfraError must include popupCaptureInfraError"
    );
}

#[test]
fn render_visual_source_does_not_require_os_screenshot_success() {
    assert!(
        VERIFY_SHOT.contains("visualSource === \"render\"")
            && VERIFY_SHOT.contains("wantsRenderVisual")
            && VERIFY_SHOT.contains("wantsOsVisual"),
        "render-only visual source must be modeled separately from OS screenshot capture"
    );
    assert!(
        VERIFY_SHOT.contains("renderInfraError")
            && VERIFY_SHOT.contains("countsAsOsScreenshotEvidence: false")
            && VERIFY_SHOT.contains("countsAsAppRenderEvidence")
            && VERIFY_SHOT.contains("countsAsOffscreenRenderEvidence: false"),
        "render-only receipts must fail or pass on render evidence, not OS screenshot evidence"
    );
    assert!(
        VERIFY_SHOT.contains("localPixelAudit")
            && VERIFY_SHOT.contains("nonBlankPixels")
            && VERIFY_SHOT.contains("notOsScreenshot"),
        "render readback receipts must audit local pixels and state their proof limitation"
    );
}
