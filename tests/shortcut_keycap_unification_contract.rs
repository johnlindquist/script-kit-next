const FOOTER_CHROME: &str = include_str!("../src/components/footer_chrome.rs");
const LIST_ITEM: &str = include_str!("../src/list_item/mod.rs");
const ACTIONS_DIALOG: &str = include_str!("../src/actions/dialog.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("section start must exist");
    let tail = &source[start_idx..];
    let end_idx = tail
        .find(end)
        .map(|idx| start_idx + idx)
        .unwrap_or(source.len());
    &source[start_idx..end_idx]
}

#[test]
fn footer_chrome_exposes_row_keycap_renderer_and_layout_model() {
    assert!(FOOTER_CHROME.contains("render_footer_row_shortcut_keycaps_from_tokens"));
    assert!(FOOTER_CHROME.contains("render_footer_shortcut_keycaps_from_tokens"));
    assert!(FOOTER_CHROME.contains("footer_shortcut_keycap_layout_model"));
    assert!(FOOTER_CHROME.contains("FOOTER_SHORTCUT_LAYOUT_MEASUREMENT_SOURCE"));
    assert!(FOOTER_CHROME.contains("FOOTER_KEYCAP_HEIGHT_PX"));
    assert!(FOOTER_CHROME.contains("footer_key_glyph_nudge_y(token)"));
    assert!(FOOTER_CHROME.contains("fn render_footer_keycap("));
    assert!(!FOOTER_CHROME.contains("pub(crate) fn render_footer_keycap("));
}

#[test]
fn main_list_row_shortcuts_use_footer_keycaps_not_hint_strip_inline_renderer() {
    let shortcut_section =
        section_between(LIST_ITEM, "// Shortcut", "// Determine background color");

    assert!(
        shortcut_section.contains(
            "crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens"
        ),
        "main list row shortcuts must render through footer_chrome row keycaps"
    );
    assert!(
        shortcut_section.contains("should_show_search_shortcut"),
        "main list must preserve selected-only shortcut visibility"
    );
    assert!(
        !shortcut_section.contains("crate::components::hint_strip::render_inline_shortcut_keys")
            && !shortcut_section.contains("whisper_inline_shortcut_colors"),
        "main list row shortcuts must not use hint_strip inline shortcut styling"
    );
}

#[test]
fn actions_dialog_row_shortcuts_use_footer_keycaps_not_hint_strip_inline_renderer() {
    let row_shortcut_section = section_between(
        ACTIONS_DIALOG,
        "// Action menus intentionally keep shortcuts visible on all rows.",
        "let action_row = div()",
    );

    assert!(
        row_shortcut_section.contains(
            "crate::components::footer_chrome::render_footer_row_shortcut_keycaps_from_tokens"
        ),
        "Actions dialog row shortcuts must render through footer_chrome row keycaps"
    );
    assert!(
        row_shortcut_section.contains("RowShortcutVisibilityPolicy::AllRows"),
        "Actions dialog must preserve all-row shortcut discoverability"
    );
    assert!(
        !row_shortcut_section
            .contains("crate::components::hint_strip::render_inline_shortcut_keys")
            && !row_shortcut_section.contains("whisper_inline_shortcut_colors"),
        "Actions dialog row shortcuts must not use hint_strip inline shortcut styling"
    );
}

#[test]
fn actions_dialog_devtools_shortcut_layout_uses_footer_keycap_model() {
    let geometry_section = section_between(
        ACTIONS_DIALOG,
        "fn devtools_row_geometry(&self, cx: &gpui::App)",
        "fn devtools_text_fingerprint",
    );

    assert!(
        geometry_section.contains(
            "crate::components::footer_chrome::footer_shortcut_keycap_layout_model_measured"
        ),
        "Actions dialog DevTools shortcutLayout must use measured footer keycap geometry"
    );
    assert!(
        geometry_section.contains("FOOTER_SHORTCUT_LAYOUT_MEASUREMENT_SOURCE"),
        "Actions dialog DevTools receipt must name the footer keycap measurement source"
    );
    assert!(
        !geometry_section.contains("crate::components::hint_strip::inline_shortcut_layout_model")
            && !geometry_section.contains("runtime.hintStrip.inlineShortcutLayoutModel"),
        "Actions dialog DevTools shortcutLayout must not report hint_strip inline geometry"
    );
}
