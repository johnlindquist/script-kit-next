//! Source-level contract for dictation overlay focus and hidden-main behavior.
//!
//! The dictation overlay must be able to appear while Script Kit's main panel
//! remains hidden, without activating the app or briefly flashing the launcher.

const DICTATION_WINDOW: &str = include_str!("../src/dictation/window.rs");

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

// @lat: [[acp-chat#ACP Chat#Detached window behavior#Dictation delivery to the composer]]
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
