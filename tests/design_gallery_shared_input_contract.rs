//! Source-level contract for Design Gallery shared main-search migration.
//!
//! Design Gallery keeps its gallery data helpers, but the filter UI must use
//! the shared GPUI search input and shared minimal list shell. Text edits flow
//! through `InputEvent::Change`; key handlers own navigation and dismissal.

const DESIGN_GALLERY: &str = include_str!("../src/render_builtins/design_gallery.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const STARTUP_NEW_ARROW: &str = include_str!("../src/app_impl/startup_new_arrow.rs");
const SIMULATE_KEY_DISPATCH: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

#[test]
fn design_gallery_uses_shared_search_input_shell() {
    let start = DESIGN_GALLERY
        .find("fn render_design_gallery(")
        .expect("missing render_design_gallery");
    let render_body = &DESIGN_GALLERY[start..];

    for required in [
        "self.render_search_input()",
        "render_minimal_list_prompt_shell_with_footer(",
        ".key_context(\"design_gallery\")",
        ".track_focus(&self.focus_handle)",
        ".on_key_down(handle_key)",
        "main_window_footer_slot(",
        "render_simple_hint_strip(",
        "Self::design_gallery_count_label(filtered_len)",
    ] {
        assert!(
            render_body.contains(required),
            "Design Gallery render must keep shared input/shell pattern: {required}"
        );
    }

    for forbidden in [
        "input_display",
        "input_is_empty",
        "design_gallery_input_display",
        "Search input with blinking cursor",
        "CURSOR_WIDTH",
        "CURSOR_HEIGHT_LG",
        "CURSOR_MARGIN_Y",
        "CURSOR_GAP_X",
        ".ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))",
        ".child(div().text_xl().child(\"🎨\"))",
    ] {
        assert!(
            !render_body.contains(forbidden),
            "Design Gallery render must not keep bespoke fake input chrome: {forbidden}"
        );
    }
}

#[test]
fn design_gallery_shared_input_drives_filter_state() {
    let input_change_body = source_between(
        STARTUP,
        "InputEvent::Change =>",
        "\n                InputEvent::PressEnter",
    );

    for required in [
        "AppView::DesignGalleryView",
        "*filter != current_value",
        "*filter = current_value",
        "*selected_index = 0",
        "this.design_gallery_scroll_handle",
        ".scroll_to_item(0, gpui::ScrollStrategy::Top)",
        "cx.notify()",
    ] {
        assert!(
            input_change_body.contains(required),
            "InputEvent::Change must route Design Gallery shared input: {required}"
        );
    }
}

#[test]
fn design_gallery_key_handler_does_not_duplicate_text_input() {
    let key_handler_body = source_between(
        DESIGN_GALLERY,
        "let handle_key = cx.listener(",
        "\n        // Pre-compute colors",
    );

    for required in [
        "is_key_up(key)",
        "is_key_down(key)",
        "is_key_escape(key)",
        "key.eq_ignore_ascii_case(\"w\")",
    ] {
        assert!(
            key_handler_body.contains(required),
            "Design Gallery key handler must retain navigation/dismissal handling: {required}"
        );
    }

    for forbidden in [
        "filter.pop()",
        "filter.push(",
        "event.keystroke.key_char",
        "\"backspace\" =>",
        "\"delete\" =>",
    ] {
        assert!(
            !key_handler_body.contains(forbidden),
            "Design Gallery key handler must not duplicate shared text input: {forbidden}"
        );
    }
}

#[test]
fn design_gallery_global_arrow_navigation_uses_filtered_rows() {
    for (label, source) in [
        ("startup.rs", STARTUP),
        ("startup_new_arrow.rs", STARTUP_NEW_ARROW),
    ] {
        let arrow_body = source_between(
            source,
            "AppView::DesignGalleryView {\n                                    selected_index,",
            "\n                                AppView::WindowSwitcherView",
        );

        for required in [
            "Self::design_gallery_visible_rows(filter).len()",
            "this.design_gallery_scroll_handle",
            "gpui::ScrollStrategy::Nearest",
            "this.input_mode = InputMode::Keyboard",
            "this.hovered_index = None",
            "cx.notify()",
        ] {
            assert!(
                arrow_body.contains(required),
                "{label} Design Gallery arrow branch must use filtered shared rows: {required}"
            );
        }

        for forbidden in [
            "design_gallery_total_items()",
            "build_gallery_items().len()",
        ] {
            assert!(
                !arrow_body.contains(forbidden),
                "{label} Design Gallery arrow branch must not use raw gallery bounds: {forbidden}"
            );
        }
    }
}

#[test]
fn design_gallery_simulate_key_navigation_uses_filtered_rows() {
    let simulate_body = source_between(
        SIMULATE_KEY_DISPATCH,
        "AppView::DesignGalleryView { .. } =>",
        "\n                AppView::PathPrompt",
    );

    for required in [
        "SimulateKey: Dispatching",
        "DesignGalleryView",
        "Self::design_gallery_visible_rows(&filter).len()",
        "view.design_gallery_scroll_handle",
        ".scroll_to_item(new_index, ScrollStrategy::Nearest)",
        "view.input_mode = InputMode::Keyboard",
        "view.hovered_index = None",
        "ctx.notify()",
    ] {
        assert!(
            simulate_body.contains(required),
            "simulateKey Design Gallery navigation must use filtered rows: {required}"
        );
    }

    for forbidden in [
        "design_gallery_total_items()",
        "build_gallery_items().len()",
    ] {
        assert!(
            !simulate_body.contains(forbidden),
            "simulateKey Design Gallery navigation must not use raw gallery bounds: {forbidden}"
        );
    }
}
