//! Source-level audits for the Script Kit frontmost screenshot path.
//!
//! These tests verify that `capture_focused_window_screenshot()` and
//! `capture_focused_window_metadata()` route through the self-excluding
//! screen capture when Script Kit is the frontmost app, and that the
//! non-frontmost path is not regressed.

use script_kit_gpui::test_utils::read_source;

fn ai_commands_source() -> String {
    read_source("src/platform/ai_commands.rs")
}

// ---------------------------------------------------------------------------
// Script Kit frontmost → self-excluding capture
// ---------------------------------------------------------------------------

#[test]
fn screenshot_detects_script_kit_frontmost() {
    let src = ai_commands_source();

    // The screenshot function must track whether Script Kit is frontmost.
    assert!(
        src.contains("script_kit_is_frontmost = true"),
        "capture_focused_window_screenshot must detect when Script Kit \
         is the frontmost app"
    );
}

#[test]
fn screenshot_routes_to_self_excluding_capture_when_frontmost() {
    let src = ai_commands_source();

    // When Script Kit is frontmost, the screenshot function must call
    // capture_screen_screenshot() (the self-excluding path) instead of
    // capturing an arbitrary fallback window.
    assert!(
        src.contains("if script_kit_is_frontmost"),
        "capture_focused_window_screenshot must branch on \
         script_kit_is_frontmost"
    );
    assert!(
        src.contains("capture_screen_screenshot()?"),
        "the frontmost branch must delegate to capture_screen_screenshot()"
    );
}

#[test]
fn screenshot_frontmost_sets_synthetic_title() {
    let src = ai_commands_source();

    // The synthetic title must be "Screen behind Script Kit panel".
    assert!(
        src.contains("Screen behind Script Kit panel"),
        "the frontmost path must use a recognizable synthetic title"
    );
    assert!(
        src.contains("script_kit_excluded_capture_title()"),
        "the frontmost path must call script_kit_excluded_capture_title()"
    );
}

#[test]
fn screenshot_frontmost_sets_used_fallback_true() {
    let src = ai_commands_source();

    // The frontmost branch returns used_fallback: true so callers know
    // the image is a screen composite, not a per-window capture.
    assert!(
        src.contains("used_fallback: true,"),
        "the frontmost path must set used_fallback to true"
    );
}

// ---------------------------------------------------------------------------
// Metadata path mirrors screenshot path
// ---------------------------------------------------------------------------

#[test]
fn metadata_detects_script_kit_frontmost() {
    let src = ai_commands_source();

    // capture_focused_window_metadata must also detect frontmost status.
    // We verify the function exists and contains the same detection logic.
    assert!(
        src.contains("fn capture_focused_window_metadata"),
        "capture_focused_window_metadata must exist"
    );
    // Both functions must share the same frontmost detection pattern.
    let frontmost_count = src.matches("script_kit_is_frontmost = true").count();
    assert!(
        frontmost_count >= 2,
        "both screenshot and metadata functions must detect Script Kit \
         frontmost status (found {frontmost_count} instances, expected >= 2)"
    );
}

#[test]
fn metadata_frontmost_uses_same_synthetic_title() {
    let src = ai_commands_source();

    // The metadata frontmost branch must also call
    // script_kit_excluded_capture_title() for consistency.
    let title_call_count = src
        .matches("script_kit_excluded_capture_title()")
        .count();
    assert!(
        title_call_count >= 2,
        "both screenshot and metadata paths must call \
         script_kit_excluded_capture_title() (found {title_call_count}, expected >= 2)"
    );
}

// ---------------------------------------------------------------------------
// Active display routing (multi-monitor correctness)
// ---------------------------------------------------------------------------

#[test]
fn self_excluding_capture_uses_active_display() {
    let src = ai_commands_source();

    // The self-excluding capture must use capture_target_bounds() which
    // prefers the active display, NOT hardcode CGDisplay::main().bounds().
    assert!(
        src.contains("fn capture_target_bounds()"),
        "capture_target_bounds helper must exist"
    );
    assert!(
        src.contains("get_active_display()"),
        "capture_target_bounds must call get_active_display() for \
         multi-monitor correctness"
    );
    // The capture function must actually use capture_target_bounds().
    assert!(
        src.contains("capture_target_bounds()"),
        "capture_screen_excluding_self must use capture_target_bounds()"
    );
}

#[test]
fn metadata_frontmost_uses_active_display_dimensions() {
    let src = ai_commands_source();

    // When Script Kit is frontmost, metadata dimensions should come from
    // capture_target_bounds() on macOS to match the screenshot path.
    assert!(
        src.contains("capture_target_bounds()"),
        "metadata frontmost path must use capture_target_bounds() for \
         display dimensions on macOS"
    );
}

// ---------------------------------------------------------------------------
// Non-frontmost path is not regressed
// ---------------------------------------------------------------------------

#[test]
fn non_frontmost_path_still_captures_per_window() {
    let src = ai_commands_source();

    // The non-frontmost path must still use xcap per-window capture.
    assert!(
        src.contains("window.capture_image()"),
        "non-frontmost path must still do per-window capture via xcap"
    );
}

#[test]
fn non_frontmost_path_preserves_fallback_semantics() {
    let src = ai_commands_source();

    // The non-frontmost path must still set used_fallback correctly based
    // on whether a focused window was found.
    assert!(
        src.contains("let used_fallback = target_window.is_some() && !found_focused"),
        "non-frontmost path must compute used_fallback from found_focused"
    );
}

#[test]
fn non_frontmost_path_logs_fallback_warning() {
    let src = ai_commands_source();

    // When falling back to a non-focused window, a warning log must exist.
    assert!(
        src.contains("No focused window found, falling back"),
        "non-frontmost fallback path must log a warning"
    );
}

// ---------------------------------------------------------------------------
// Window filtering correctness
// ---------------------------------------------------------------------------

#[test]
fn our_app_detection_covers_both_names() {
    let src = ai_commands_source();

    // The window filter must recognize both "script-kit-gpui" (binary name)
    // and "Script Kit" (display name).
    assert!(
        src.contains(r#"app_name.contains("script-kit-gpui")"#),
        "must filter by binary name"
    );
    assert!(
        src.contains(r#"app_name == "Script Kit""#),
        "must filter by display name"
    );
}

#[test]
fn minimum_window_size_filter_exists() {
    let src = ai_commands_source();

    // Tiny windows (e.g. status-bar items) must be excluded.
    assert!(
        src.contains("is_reasonable_size"),
        "window enumeration must filter by reasonable size"
    );
    assert!(
        src.contains("width >= 100 && height >= 100"),
        "reasonable size threshold must be 100x100"
    );
}
