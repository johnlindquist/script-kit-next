use std::fs;

fn prompt_handler_source() -> String {
    fs::read_to_string("src/prompt_handler/mod.rs").expect("read prompt handler source")
}

fn hud_manager_source() -> String {
    fs::read_to_string("src/hud_manager/mod.rs").expect("read HUD manager source")
}

fn show_hud_arm(source: &str) -> &str {
    let start = source
        .find("PromptMessage::ShowHud { text, duration_ms } => {")
        .expect("ShowHud arm exists");
    let tail = &source[start..];
    let end = tail
        .find("PromptMessage::SetStatus")
        .expect("SetStatus arm follows ShowHud");
    &tail[..end]
}

#[test]
fn hud_message_consumes_script_hide_restore_intent() {
    let source = prompt_handler_source();
    let arm = show_hud_arm(&source);

    assert!(
        arm.contains("script_kit_gpui::script_requested_hide()"),
        "ShowHud must inspect the script-requested hide flag before rendering standalone HUD feedback"
    );
    assert!(
        arm.contains("script_kit_gpui::set_script_requested_hide(false)"),
        "ShowHud must clear pending main-window restore intent so HUD visibility stays independent"
    );
    assert!(
        arm.contains("self.show_hud(text, duration_ms, cx);"),
        "ShowHud should still delegate to the HUD manager"
    );
}

#[test]
fn hud_message_does_not_request_main_window_show() {
    let source = prompt_handler_source();
    let arm = show_hud_arm(&source);

    assert!(
        !arm.contains("request_show_main_window"),
        "ShowHud must not request the main menu; HUD visibility is independent from launcher visibility"
    );
    assert!(
        !arm.contains("set_main_window_visible(true)"),
        "ShowHud must not mark the main window visible"
    );
    assert!(
        !arm.contains("prepare_window_for_prompt"),
        "ShowHud is not prompt UI and must not use prompt window preparation"
    );
}

#[test]
fn hud_surface_tracks_main_window_background_opacity_and_material() {
    let source = hud_manager_source();

    assert!(
        source.contains("crate::ui_foundation::main_window_matched_background_rgba(&theme)")
            && source.contains(".bg(rgba(colors.background_rgba))"),
        "HUD pill background must use the shared main-window background helper"
    );
    assert!(
        source.contains("crate::platform::configure_hud_window_vibrancy(")
            && source.contains("theme.should_use_dark_vibrancy()"),
        "HUD native window must use the same cached theme material/appearance path as the main window"
    );
}
