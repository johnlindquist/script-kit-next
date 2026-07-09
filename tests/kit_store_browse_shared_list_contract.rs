const KIT_STORE_SOURCE: &str = include_str!("../src/render_builtins/kit_store.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn browse_kit_store_uses_shared_search_input_and_list_item_rows() {
    let body = function_body(
        KIT_STORE_SOURCE,
        "fn render_browse_kits(",
        "fn render_installed_kits(",
    );

    for required in [
        "render_builtin_main_input_header(",
        "render_builtin_main_input_count_label(",
        "render_main_view_chrome_footer_flush(",
        "ListItem::new(",
        "ListItemColors::from_theme(&self.theme)",
        ".selected(is_selected)",
        ".hovered(is_hovered)",
        ".main_menu_theme(main_menu_theme)",
        ".with_accent_bar(true)",
        ".semantic_id(",
        "crate::ui_foundation::should_submit_selected_row_click",
        "builtin_uniform_list_scrollbar(&self.list_scroll_handle",
        "kit_store_set_browse_query(",
    ] {
        assert!(
            body.contains(required),
            "Browse Kit Store must use shared list/search contract: {required}"
        );
    }

    for forbidden in [
        "CURSOR_WIDTH",
        "CURSOR_HEIGHT_LG",
        "kit-store-install-btn",
        "selected_row_bg",
        "hover_row_bg",
        ".bg(accent_badge_bg)",
        "h(px(KIT_STORE_ROW_HEIGHT))",
    ] {
        assert!(
            !body.contains(forbidden),
            "Browse Kit Store must not keep bespoke search/row chrome: {forbidden}"
        );
    }
}

#[test]
fn browse_kit_store_shared_input_drives_query_fetch() {
    let change_body = function_body(
        STARTUP_SOURCE,
        "InputEvent::Change => {",
        "InputEvent::PressEnter",
    );

    for required in [
        "AppView::BrowseKitsView",
        "kit_store_set_browse_query(current_value, cx)",
    ] {
        assert!(
            change_body.contains(required),
            "shared GPUI input changes must route BrowseKitsView query text through kit_store_set_browse_query: {required}"
        );
    }
}
