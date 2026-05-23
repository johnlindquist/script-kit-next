//! Source-level contract for dictation overlay focus and hidden-main behavior.
//!
//! The dictation overlay must be able to appear while Script Kit's main panel
//! remains hidden, without activating the app or briefly flashing the launcher.

const DICTATION_WINDOW: &str = include_str!("../src/dictation/window.rs");
const FOOTER_CHROME: &str = include_str!("../src/components/footer_chrome.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("section start must exist");
    let tail = &source[start_idx..];
    let end_idx = tail
        .find(end)
        .map(|idx| start_idx + idx)
        .unwrap_or(source.len());
    &source[start_idx..end_idx]
}

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

// doc-anchor-removed: [[acp-chat#ACP Chat#Detached window behavior#Dictation delivery to the composer]]
#[test]
fn dictation_overlay_opens_without_revealing_hidden_main_panel() {
    let section = section_between(
        DICTATION_WINDOW,
        "pub fn open_dictation_overlay(",
        "pub fn update_dictation_overlay(",
    );
    let compact_section = compact(section);

    assert!(
        compact_section.contains(&compact(
            "let main_was_visible = crate::is_main_window_visible();"
        )),
        "overlay must snapshot main-window visibility before native window operations"
    );
    let visibility_snapshot_pos = section
        .find("let main_was_visible = crate::is_main_window_visible();")
        .expect("overlay must snapshot main visibility before native window creation");
    let open_window_pos = section
        .find(".open_window(window_options")
        .expect("overlay must create the native window through GPUI open_window");
    assert!(
        visibility_snapshot_pos < open_window_pos,
        "main visibility must be captured before opening the overlay window"
    );
    assert!(
        compact_section.contains(&compact("focus: false,")),
        "overlay must not activate the app when it opens"
    );
    assert!(
        compact_section.contains(&compact("show: false,")),
        "overlay must be created hidden so GPUI does not surface sibling launcher panels"
    );
    assert!(
        compact_section.contains(&compact("kind: gpui::WindowKind::PopUp,")),
        "overlay must stay on GPUI's nonactivating popup window path"
    );

    let order_out_pos = section
        .find("msg_send![main_window, orderOut: cocoa::base::nil]")
        .expect("overlay must order out the hidden main panel before surfacing");
    let order_front_pos = section
        .find("msg_send![ns_window, orderFrontRegardless]")
        .expect("overlay must order itself front without activation");
    assert!(
        order_out_pos < order_front_pos,
        "hidden-main orderOut must happen before overlay orderFrontRegardless"
    );
}

// doc-anchor-removed: [[dictation-overlay-shortcuts#Dictation Overlay Shortcuts#Visible shortcut rail]]
#[test]
fn dictation_overlay_renders_visible_shortcut_rail() {
    assert!(
        DICTATION_WINDOW.contains("pub(crate) const OVERLAY_WIDTH_PX: f32 = 520.0;")
            && DICTATION_WINDOW.contains("pub(crate) const OVERLAY_HEIGHT_PX: f32 = 72.0;"),
        "dictation overlay must reserve enough room for visible controls and action chips"
    );
    assert!(
        DICTATION_WINDOW.contains("const ACTION_STOP_LABEL: &str = \"Stop\";")
            && DICTATION_WINDOW.contains("const ACTION_MIC_LABEL: &str = \"Select Mic\";")
            && DICTATION_WINDOW.contains("const ACTION_CANCEL_LABEL: &str = \"Cancel\";")
            && DICTATION_WINDOW.contains("const ACTION_CONTINUE_LABEL: &str = \"Continue\";")
            && DICTATION_WINDOW.contains("const ACTION_CLOSE_LABEL: &str = \"Close\";")
            && DICTATION_WINDOW.contains("const ESC_KEYCAP: &str = \"esc\";")
            && DICTATION_WINDOW.contains("const ENTER_KEYCAP: &str = \"\\u{21b5}\";")
            && DICTATION_WINDOW.contains("const MIC_ICON_PATH: &str = concat!("),
        "recording, confirming, and terminal phases must use compact action labels plus keycaps"
    );
    assert!(
        DICTATION_WINDOW.contains("fn render_action_chip")
            && DICTATION_WINDOW.contains("fn render_clickable_action_chip")
            && DICTATION_WINDOW.contains(".id(\"dictation-action-rail\")")
            && DICTATION_WINDOW.contains("self.render_recording_actions(cx)")
            && DICTATION_WINDOW.contains("render_static_action_rail(["),
        "runtime and preview renders must both include the visible compact action rail"
    );
    assert!(
        DICTATION_WINDOW
            .contains("crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT")
            && DICTATION_WINDOW.contains("crate::components::footer_chrome::footer_rail_chrome")
            && DICTATION_WINDOW.contains(".bg(rgba(rail_chrome.surface_rgba))")
            && DICTATION_WINDOW.contains("rgba(rail_chrome.divider_rgba)")
            && DICTATION_WINDOW.contains(".px(px(rail_chrome.side_inset_px))"),
        "dictation action rail must consume shared native-footer surface, height, divider, and inset tokens"
    );
    assert!(
        DICTATION_WINDOW.contains("crate::components::footer_chrome::render_footer_hint_content")
            && DICTATION_WINDOW.contains("crate::components::footer_chrome::FooterHintKeyMode")
            && DICTATION_WINDOW.contains("fn render_mic_action_chip_content(")
            && DICTATION_WINDOW.contains(".external_path(MIC_ICON_PATH)")
            && DICTATION_WINDOW.contains("fn footer_action_button_height()")
            && DICTATION_WINDOW.contains(".h(px(footer_action_button_height()))")
            && DICTATION_WINDOW.contains(".group(\"footer-action-button\")")
            && !DICTATION_WINDOW.contains("render_inline_shortcut_keys("),
        "dictation action chips must render through the shared footer chrome owner with inset button height"
    );
    assert!(
        DICTATION_WINDOW.contains(
            "pub(crate) const DICTATION_OVERLAY_FOOTER_SURFACE: &str = \"dictation_overlay\";"
        ),
        "live dictation overlay should keep its own local footer/debug surface identity"
    );
    assert!(
        !DICTATION_WINDOW.contains("footer_action_channel")
            && !DICTATION_WINDOW.contains("MainWindowFooterConfig")
            && !DICTATION_WINDOW.contains("active_main_window_footer_surface")
            && !DICTATION_WINDOW.contains("FooterAction::"),
        "dictation overlay must not import main-window native footer ownership or action routing"
    );
    assert!(
        FOOTER_CHROME.contains("pub(crate) const FOOTER_HINT_FONT_SIZE_PX: f32 = 12.5;")
            && FOOTER_CHROME
                .contains("pub(crate) const FOOTER_HINT_FONT_WEIGHT_APPKIT: f64 = 0.18;")
            && FOOTER_CHROME.contains(
                "pub(crate) const FOOTER_HINT_FONT_WEIGHT_GPUI: FontWeight = FontWeight::SEMIBOLD;"
            )
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_KEYCAP_HEIGHT_PX: f32 = 20.0;")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_KEY_GLYPH_NUDGE_Y_PX: f32 = 1.0;")
            && FOOTER_CHROME
                .contains("pub(crate) const FOOTER_RETURN_GLYPH_NUDGE_Y_PX: f32 = 1.0;")
            && FOOTER_CHROME
                .contains("pub(crate) const FOOTER_BUTTON_VERTICAL_INSET_PX: f32 = 2.0;")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_LABELCAP_BORDER_ALPHA: f32 = 0.0;")
            && FOOTER_CHROME.contains("let alpha = footer_keycap_border_alpha(theme, selected);")
            && FOOTER_CHROME.contains("pub(crate) fn footer_button_height(footer_height: f32)")
            && FOOTER_CHROME.contains("pub(crate) fn footer_rail_chrome(theme: &Theme)")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_ITEM_GAP_PX")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_CONTENT_GAP_PX")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_CONTENT_PADDING_X_PX")
            && FOOTER_CHROME.contains("pub(crate) const FOOTER_ACTION_BUTTON_RADIUS_PX")
            && FOOTER_CHROME
                .contains("FOOTER_KEY_GLYPH_NUDGE_Y_PX + FOOTER_RETURN_GLYPH_NUDGE_Y_PX"),
        "shared footer chrome tokens must pin native font, keycap, rail, and button chrome contracts"
    );
    assert!(
        FOOTER_CHROME.contains("\"esc\" | \"escape\" => \"⎋\".to_string()")
            && FOOTER_CHROME.contains("fn render_footer_labelcap(")
            && FOOTER_CHROME.contains("footer_labelcap_border_color(theme)")
            && !FOOTER_CHROME.contains(".bg(footer_keycap_bg_color(theme))")
            && !FOOTER_CHROME.contains("FOOTER_KEYCAP_BG_ALPHA")
            && FOOTER_CHROME.contains(".flex_none()")
            && FOOTER_CHROME.contains(".min_h(px(FOOTER_KEYCAP_HEIGHT_PX))")
            && FOOTER_CHROME.contains(".line_height(px(FOOTER_KEYCAP_HEIGHT_PX))")
            && FOOTER_CHROME.contains(".group_hover(\"footer-action-button\""),
        "shared footer keycaps must use the escape glyph, fixed native-footer sizing, labelcap balance, hover foreground, and no steady-state fill"
    );
    assert!(
        DICTATION_WINDOW.contains("\"dictation-stop-button\"")
            && DICTATION_WINDOW.contains("\"dictation-mic-button\"")
            && DICTATION_WINDOW.contains("\"dictation-cancel-button\"")
            && DICTATION_WINDOW.contains("this.submit_overlay_session(window, cx)")
            && DICTATION_WINDOW.contains("this.open_microphone_picker(window, cx)")
            && DICTATION_WINDOW.contains("this.abort_overlay_session(window, cx)"),
        "recording Stop, Mic, and Cancel controls must be clickable from the overlay"
    );
    assert!(
        DICTATION_WINDOW.contains("fn open_microphone_picker")
            && DICTATION_WINDOW.contains("list_input_device_menu_items(selected_device_id)")
            && DICTATION_WINDOW.contains("build_dictation_microphone_popup_snapshot")
            && DICTATION_WINDOW.contains("sync_dictation_microphone_popup_window"),
        "overlay Select Mic control must use the shared microphone menu items and attached popup window"
    );
    assert!(
        DICTATION_WINDOW.contains("fn dictation_stop_keycap()")
            && DICTATION_WINDOW.contains("fn dictation_hotkey_keycap(")
            && DICTATION_WINDOW.contains(".get_dictation_hotkey()")
            && DICTATION_WINDOW.contains(".replace(\"Semicolon\", \";\")")
            && !DICTATION_WINDOW.contains("unwrap_or_else(|| \"again\".to_string())"),
        "recording Stop must show the configured dictation hotkey instead of the stale Again label"
    );

    let runtime_render = section_between(
        DICTATION_WINDOW,
        "impl Render for DictationOverlay",
        "/// Format the finished overlay state label.",
    );
    let preview_render = section_between(
        DICTATION_WINDOW,
        "pub(crate) fn render_dictation_overlay_state_preview",
        "fn render_static_target_badge_slot",
    );
    assert!(
        runtime_render.contains("DictationSessionPhase::Delivering")
            && runtime_render.contains("self.render_close_action(cx)"),
        "runtime Delivering state must render the compact Close + esc action"
    );
    assert!(
        preview_render.contains("DictationSessionPhase::Delivering")
            && preview_render.contains("ACTION_CLOSE_LABEL")
            && preview_render.contains("ESC_KEYCAP"),
        "preview Delivering state must render the same compact Close + esc action"
    );
}
