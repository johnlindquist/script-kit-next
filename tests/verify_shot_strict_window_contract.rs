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
