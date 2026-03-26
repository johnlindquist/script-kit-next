use script_kit_gpui::test_utils::read_source;

/// Verify that the recommendation profile stays lightweight by not capturing
/// focused-window screenshots on the typing path.
#[test]
fn recommendation_profile_stays_lightweight() {
    let types_source = read_source("src/context_snapshot/types.rs");

    assert!(
        types_source.contains("include_focused_window: false"),
        "recommendation() must set include_focused_window to false — \
         the current focused-window provider takes a screenshot, which is \
         too expensive for the typing path"
    );

    // Menu bar is also excluded from the recommendation profile.
    assert!(
        types_source.contains("include_menu_bar: false"),
        "recommendation() must skip menu bar capture for typing-path latency"
    );
}

/// Verify that the preflight system uses the dedicated recommendation profile
/// rather than a hand-rolled or full-capture option set.
#[test]
fn preflight_uses_recommendation_profile() {
    let preflight_source = read_source("src/ai/window/context_preflight.rs");

    assert!(
        preflight_source.contains("CaptureContextOptions::recommendation()"),
        "context preflight must use CaptureContextOptions::recommendation() — \
         do not inline option fields or substitute a different profile"
    );
}

/// Document that focused-window capture is still screenshot-based.
/// When a metadata-only provider is added, this assertion should be removed
/// and `include_focused_window` can be re-enabled in `recommendation()`.
#[test]
fn focused_window_provider_is_still_screenshot_based() {
    let platform_source = read_source("src/platform/ai_commands.rs");

    assert!(
        platform_source.contains("capture_focused_window_screenshot"),
        "this audit assumes focused-window capture is still screenshot-based; \
         remove this assertion once a metadata-only provider exists and \
         re-enable include_focused_window in the recommendation profile"
    );
}
