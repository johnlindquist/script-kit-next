//! Source audit tests pinning the hidden ScriptList reset resize-to-default
//! behaviour added by `audit(tool-scriptlist-hide-bounds-reset-fix)`.
//!
//! When the main window is hidden while a subview (fileSearch/emoji picker) had
//! grown it past its default mini dimensions, the hidden reset path must restore
//! the default mini ScriptList bounds after it has re-keyed the automation
//! surface to `"scriptList"`.

use super::read_source as read;

const WINDOW_VISIBILITY_PATH: &str = "src/main_sections/window_visibility.rs";
const LIFECYCLE_RESET_PATH: &str = "src/app_impl/lifecycle_reset.rs";

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

fn reset_helper_body(content: &str) -> &str {
    let fn_start = content
        .find("fn reset_hidden_main_window_to_script_list(")
        .expect("Expected reset_hidden_main_window_to_script_list function in lifecycle_reset.rs");
    let fn_end = content[fn_start + 1..]
        .find("\n    pub(crate) fn ")
        .map(|p| fn_start + 1 + p)
        .unwrap_or(content.len());
    &content[fn_start..fn_end]
}

fn defer_reset_helper_body(content: &str) -> &str {
    let fn_start = content
        .find("pub(crate) fn defer_reset_to_script_list_after_main_window_hidden(")
        .expect("Expected defer_reset_to_script_list_after_main_window_hidden function in lifecycle_reset.rs");
    let fn_end = content[fn_start + 1..]
        .find("\n    pub(crate) fn ")
        .map(|p| fn_start + 1 + p)
        .unwrap_or(content.len());
    &content[fn_start..fn_end]
}

#[test]
fn hide_main_window_helper_resizes_to_mini_default_when_in_mini_mode() {
    let visibility = read(WINDOW_VISIBILITY_PATH);
    let hide_body = hide_helper_body(&visibility);
    let lifecycle = read(LIFECYCLE_RESET_PATH);
    let reset_body = reset_helper_body(&lifecycle);
    let defer_body = defer_reset_helper_body(&lifecycle);

    assert!(
        hide_body.contains("reset_mini_bounds_after_hidden_reset")
            && hide_body.contains("defer_reset_to_script_list_after_main_window_hidden("),
        "hide_main_window_helper must pass mini-mode reset intent into the deferred hidden reset"
    );
    assert!(
        defer_body.contains("resize_to_mini_main_window_sync("),
        "deferred hidden reset must restore default mini ScriptList bounds"
    );
    assert!(
        reset_body.contains("MainWindowMode::Mini"),
        "hidden reset resize must be gated on MainWindowMode::Mini so Full-mode sizing is not clobbered on hide"
    );
}

#[test]
fn hide_resize_runs_after_surface_rekey_not_before() {
    let lifecycle = read(LIFECYCLE_RESET_PATH);
    let reset_body = reset_helper_body(&lifecycle);
    let defer_body = defer_reset_helper_body(&lifecycle);

    let rekey_pos = reset_body
        .find("self.rekey_main_automation_surface_from_current_view();")
        .expect("hidden reset must re-key automation surface from current ScriptList view");
    let visibility_pos = reset_body
        .find("set_automation_visibility(\"main\", false)")
        .expect("hidden reset must preserve hidden automation visibility after re-key");
    let resize_pos = defer_body
        .find("resize_to_mini_main_window_sync(")
        .expect("deferred hidden reset must call resize_to_mini_main_window_sync");

    assert!(
        rekey_pos < visibility_pos
            && defer_body.find("reset_hidden_main_window_to_script_list(cx, reason)")
                < Some(resize_pos),
        "Resize must run after the hidden reset re-keys the automation surface to ScriptList"
    );
}

#[test]
fn hide_helper_preserves_hidden_automation_visibility_after_rekey() {
    let lifecycle = read(LIFECYCLE_RESET_PATH);
    let reset_body = reset_helper_body(&lifecycle);

    assert!(
        reset_body.contains("self.rekey_main_automation_surface_from_current_view();")
            && reset_body.contains("crate::windows::set_automation_visibility(\"main\", false);"),
        "hidden reset must re-key main to ScriptList while keeping the automation window hidden"
    );
}

#[test]
fn hide_helper_does_not_resize_before_saving_position() {
    let visibility = read(WINDOW_VISIBILITY_PATH);
    let hide_body = hide_helper_body(&visibility);
    let lifecycle = read(LIFECYCLE_RESET_PATH);
    let defer_body = defer_reset_helper_body(&lifecycle);

    let save_pos = hide_body
        .find("save_main_position_for_display(")
        .expect("hide helper must save position for next show");
    let defer_pos = hide_body
        .find("defer_reset_to_script_list_after_main_window_hidden(")
        .expect("hide helper must defer hidden ScriptList reset");
    let resize_pos = defer_body
        .find("resize_to_mini_main_window_sync(")
        .expect("deferred hidden reset must call resize_to_mini_main_window_sync");

    assert!(
        save_pos < defer_pos && resize_pos > 0,
        "Position save must capture the pre-resize bounds before the hidden reset can resize to 480x440"
    );
}
