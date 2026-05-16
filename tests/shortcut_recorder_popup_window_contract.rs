use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).expect("failed to read source file")
}

#[test]
fn shortcut_recorder_opens_as_native_blurred_popup() {
    let source = repo_file("src/app_impl/shortcut_recorder.rs");
    let start = source
        .find("fn open_shortcut_recorder_window(")
        .expect("open_shortcut_recorder_window not found");
    let body = &source[start..];

    assert!(
        body.contains("WindowBackgroundAppearance::Blurred"),
        "shortcut recorder popup must request a blurred GPUI window"
    );
    assert!(
        body.contains("kind: WindowKind::PopUp"),
        "shortcut recorder popup must use the same native popup window kind as actions"
    );
    assert!(
        body.contains("crate::platform::configure_actions_popup_window"),
        "shortcut recorder popup must use the shared AppKit popup vibrancy path"
    );
    assert!(
        body.contains("attach_shortcut_recorder_to_parent_window"),
        "shortcut recorder popup should stay parent-attached for ordering"
    );
    assert!(
        source.contains("orderFrontRegardless"),
        "shortcut recorder popup should resurface without overriding GPUI popup level"
    );
    assert!(
        !source.contains("setLevel:"),
        "shortcut recorder popup must not override WindowKind::PopUp level"
    );
    assert!(
        body.contains("register_attached_popup"),
        "shortcut recorder popup must register as an attached popup for automation"
    );

    let secondary_source = repo_file("src/platform/secondary_window_config.rs");
    assert!(
        secondary_source.contains("setBecomesKeyOnlyIfNeeded: true"),
        "shared popup config should preserve no-eager-key behavior for child popups"
    );
}

#[test]
fn shortcut_recorder_detached_render_removes_parent_backdrop() {
    let source = repo_file("src/components/shortcut_recorder/render.rs");
    assert!(
        source.contains("if self.detached_window"),
        "shortcut recorder render must branch for detached popup rendering"
    );
    assert!(
        source.contains(".id(\"shortcut-backdrop\")"),
        "inline mode should keep its backdrop path"
    );

    let detached_start = source
        .find("if self.detached_window")
        .expect("detached branch not found");
    let detached_branch = &source[detached_start
        ..source[detached_start..]
            .find("} else {")
            .map(|offset| detached_start + offset)
            .expect("detached branch else not found")];
    assert!(
        !detached_branch.contains("shortcut-backdrop"),
        "detached popup rendering must not dim the parent launcher with the old backdrop"
    );
    assert!(
        detached_branch.contains("detached_surface_cancel")
            && detached_branch.contains(".on_mouse_down(gpui::MouseButton::Left"),
        "detached popup margin should be an invisible click-to-cancel target"
    );
    assert!(
        source.contains(".id(\"shortcut-modal-content\")")
            && source.contains(".on_mouse_down(gpui::MouseButton::Left, |_, _, _|"),
        "modal content should still stop mouse-down propagation so inside clicks do not cancel"
    );
}

#[test]
fn shortcut_recorder_uses_compact_modal_copy_and_bounds() {
    let popup_source = repo_file("src/app_impl/shortcut_recorder.rs");
    assert!(
        popup_source.contains("const SHORTCUT_RECORDER_POPUP_WIDTH: f32 = 360.0;"),
        "shortcut recorder popup should stay narrower than the launcher"
    );
    assert!(
        popup_source.contains("const SHORTCUT_RECORDER_POPUP_HEIGHT: f32 = 196.0;"),
        "shortcut recorder popup should stay compact and modal-sized"
    );

    let render_source = repo_file("src/components/shortcut_recorder/render.rs");
    assert!(
        render_source.contains("AppChromeColors::from_theme(&self.theme)")
            && render_source.contains("chrome.popup_surface_rgba")
            && render_source.contains("chrome.border_rgba"),
        "shortcut recorder popup chrome should use shared AppChromeColors tokens"
    );
    assert!(
        render_source.contains("mods.platform && key.eq_ignore_ascii_case(\"w\")"),
        "shortcut recorder must treat Cmd+W as explicit cancel, not a captured shortcut"
    );
    assert!(
        render_source
            .contains("if (mods.platform && key.eq_ignore_ascii_case(\"w\")) || is_key_escape(key) {\n                this.cancel();\n                cx.notify();"),
        "shortcut recorder Esc should cancel and notify like Cmd+W instead of clearing a captured shortcut"
    );
    assert!(
        render_source.contains("cx.stop_propagation()"),
        "shortcut recorder key handler should consume captured modal keys"
    );
    assert!(
        render_source.contains("RECORDER_MODAL_WIDTH"),
        "shortcut recorder should use its own compact modal width"
    );
    assert!(
        !render_source.contains("Shortcut for"),
        "shortcut recorder title copy should not include verbose framing text"
    );
    assert!(
        !render_source.contains("Esc cancels. Enter saves"),
        "shortcut recorder should avoid long instructional copy"
    );

    let helper_source = repo_file("src/components/shortcut_recorder/render_helpers.rs");
    assert!(
        helper_source.contains("\"Press keys\""),
        "empty shortcut recorder prompt should stay short"
    );
    assert!(
        !helper_source.contains("Press any key combination"),
        "empty shortcut recorder prompt should not use verbose instructional copy"
    );

    let component_source = repo_file("src/components/shortcut_recorder/component.rs");
    let handle_escape_start = component_source
        .find("pub fn handle_escape")
        .expect("ShortcutRecorder::handle_escape not found");
    let handle_escape_body = &component_source[handle_escape_start..];
    assert!(
        handle_escape_body.contains("self.cancel();")
            && handle_escape_body.contains("cx.notify();")
            && !handle_escape_body.contains("self.clear(cx)"),
        "ShortcutRecorder::handle_escape should remain a cancel path, not a clear path"
    );
}

#[test]
fn shortcut_recorder_storybook_uses_live_popup_chrome_tokens() {
    let source = repo_file("src/storybook/shortcut_recorder_states.rs");
    assert!(
        source.contains("AppChromeColors::from_theme(&theme)")
            && source.contains("chrome.popup_surface_rgba")
            && source.contains("chrome.border_rgba")
            && source.contains("chrome.text_primary_hex")
            && source.contains("chrome.accent_hex"),
        "shortcut recorder Storybook states should use the same popup chrome tokens as the live recorder"
    );
    assert!(
        !source.contains("get_vibrancy_background(&theme)")
            && !source.contains("border_color(rgba((colors.text_primary << 8) | 0x22))"),
        "shortcut recorder Storybook should not use a separate vibrancy/background and hand-packed border path"
    );
}

#[test]
fn shortcut_recorder_blocks_main_window_blur_dismissal() {
    let source = repo_file("src/main_sections/render_impl.rs");
    let focus_lost_start = source
        .find("if self.was_window_focused && !is_window_focused")
        .expect("main window focus-lost block not found");
    let focus_lost_block = &source[focus_lost_start..];

    assert!(
        focus_lost_block.contains("&& self.shortcut_recorder_state.is_none()"),
        "main window auto-dismiss must be disabled while the shortcut recorder popup is open"
    );
    assert!(
        focus_lost_block.contains("self.shortcut_recorder_state.is_some()"),
        "focus-lost logging should document the shortcut recorder coexistence path"
    );
    assert!(
        focus_lost_block.contains("shortcut recorder is open"),
        "shortcut recorder focus-lost branch should produce a diagnostic log"
    );
}
