//! Source audit tests pinning the `hide_main_window_helper` resize-to-default
//! behaviour added by `audit(tool-scriptlist-hide-bounds-reset-fix)`.
//!
//! When the main window is hidden while a subview (fileSearch/emoji picker) had
//! grown it past its default mini dimensions, `listAutomationWindows` must
//! report the restored 480×440 bounds during the hidden phase. The hide helper
//! achieves this by calling `resize_to_mini_main_window_sync` after it has
//! already re-keyed the automation surface to `"scriptList"`, then re-syncing
//! the automation registry so the new bounds are picked up.

use super::read_source as read;

const WINDOW_VISIBILITY_PATH: &str = "src/main_sections/window_visibility.rs";

fn hide_helper_body(content: &str) -> &str {
    let fn_start = content
        .find("fn hide_main_window_helper(")
        .expect("Expected hide_main_window_helper function in window_visibility.rs");
    let fn_end = content[fn_start + 1..]
        .find("\nfn ")
        .map(|p| fn_start + 1 + p)
        .unwrap_or(content.len());
    &content[fn_start..fn_end]
}

#[test]
fn hide_main_window_helper_resizes_to_mini_default_when_in_mini_mode() {
    let content = read(WINDOW_VISIBILITY_PATH);
    let body = hide_helper_body(&content);

    assert!(
        body.contains("resize_to_mini_main_window_sync("),
        "hide_main_window_helper must call resize_to_mini_main_window_sync so listAutomationWindows reports 480×440 during the hidden phase"
    );
    assert!(
        body.contains("MainWindowMode::Mini"),
        "hide helper resize must be gated on MainWindowMode::Mini so Full-mode sizing is not clobbered on hide"
    );
}

#[test]
fn hide_resize_runs_after_surface_rekey_not_before() {
    let content = read(WINDOW_VISIBILITY_PATH);
    let body = hide_helper_body(&content);

    let rekey_pos = body
        .find(r#"update_automation_semantic_surface("main", Some("scriptList""#)
        .expect("hide helper must re-key automation surface to scriptList");
    let resize_pos = body
        .find("resize_to_mini_main_window_sync(")
        .expect("hide helper must call resize_to_mini_main_window_sync");

    assert!(
        rekey_pos < resize_pos,
        "Resize must run AFTER the surface re-key so the subsequent automation sync stamps a scriptList entry with 480×440 bounds (not a stale subview surface with stale bounds)"
    );
}

#[test]
fn hide_helper_resyncs_automation_window_after_resize() {
    let content = read(WINDOW_VISIBILITY_PATH);
    let body = hide_helper_body(&content);

    let resize_pos = body
        .find("resize_to_mini_main_window_sync(")
        .expect("hide helper must call resize_to_mini_main_window_sync");
    let remainder = &body[resize_pos..];

    assert!(
        remainder.contains("sync_main_automation_window(current_main_automation_bounds(), false, false)"),
        "hide helper must re-sync the automation registry AFTER resize so the new 480×440 bounds propagate to listAutomationWindows. Without the resync, automation keeps the pre-resize bounds (e.g. 750×500) even though the NSWindow itself has already been resized."
    );
}

#[test]
fn hide_helper_does_not_resize_before_saving_position() {
    let content = read(WINDOW_VISIBILITY_PATH);
    let body = hide_helper_body(&content);

    let save_pos = body
        .find("save_main_position_for_display(")
        .expect("hide helper must save position for next show");
    let resize_pos = body
        .find("resize_to_mini_main_window_sync(")
        .expect("hide helper must call resize_to_mini_main_window_sync");

    assert!(
        save_pos < resize_pos,
        "Position save must capture the pre-resize bounds so next show() can restore the user's existing window position/size; the resize must happen AFTER save to avoid collapsing persisted bounds to 480×440"
    );
}
