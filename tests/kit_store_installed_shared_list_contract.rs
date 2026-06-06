const KIT_STORE_SOURCE: &str = include_str!("../src/render_builtins/kit_store.rs");
const STARTUP_SOURCE: &str = include_str!("../src/app_impl/startup.rs");

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

#[test]
fn installed_kits_uses_shared_search_input_and_list_item_rows() {
    let body = function_body(KIT_STORE_SOURCE, "fn render_installed_kits(", "}\n}");

    for required in [
        "render_builtin_main_input_header(",
        "render_builtin_main_input_count_label(",
        "render_main_view_chrome(",
        "ListItem::new(",
        "ListItemColors::from_theme(&self.theme)",
        ".selected(is_selected)",
        ".hovered(is_hovered)",
        ".main_menu_theme(main_menu_theme)",
        ".with_accent_bar(true)",
        ".semantic_id(Self::kit_store_installed_row_semantic_id",
        "builtin_uniform_list_scrollbar(&self.list_scroll_handle",
        "main_window_footer_slot(crate::components::render_simple_hint_strip",
        "crate::ui_foundation::should_submit_selected_row_click",
    ] {
        assert!(
            body.contains(required),
            "Installed Kits must use shared search/list contract: {required}"
        );
    }

    for forbidden in [
        "h(px(KIT_STORE_ROW_HEIGHT))",
        "selected_row_bg",
        "hover_row_bg",
        "kit-store-update-btn",
        "kit-store-remove-btn",
        ".bg(accent_badge_bg)",
        ".bg(badge_bg)",
    ] {
        assert!(
            !body.contains(forbidden),
            "Installed Kits must not keep bespoke row chrome: {forbidden}"
        );
    }
}

#[test]
fn installed_kits_shared_input_drives_filter_state() {
    let change_body = function_body(
        STARTUP_SOURCE,
        "InputEvent::Change => {",
        "InputEvent::PressEnter",
    );

    for required in [
        "AppView::InstalledKitsView",
        "filter",
        "*filter = current_value",
        "*selected_index = 0",
        ".scroll_to_item(0, gpui::ScrollStrategy::Nearest)",
        "cx.notify()",
    ] {
        assert!(
            change_body.contains(required),
            "shared GPUI input changes must drive InstalledKitsView filter state: {required}"
        );
    }
}

#[test]
fn installed_kits_key_handler_does_not_duplicate_text_input() {
    let body = function_body(KIT_STORE_SOURCE, "fn render_installed_kits(", "}\n}");

    for required in [
        "is_key_up(key)",
        "is_key_down(key)",
        "is_key_enter(key)",
        "is_key_escape(key)",
        "\"delete\"",
        "kit_store_update_selected_kit",
        "kit_store_remove_selected_kit",
    ] {
        assert!(
            body.contains(required),
            "Installed Kits key handler must preserve navigation/action behavior: {required}"
        );
    }

    for forbidden in ["filter.push(", "filter.pop()", "\"backspace\""] {
        assert!(
            !body.contains(forbidden),
            "Installed Kits shared input must own text editing/backspace: {forbidden}"
        );
    }
}

#[test]
fn installed_kits_navigation_uses_filtered_rows() {
    let body = function_body(KIT_STORE_SOURCE, "fn render_installed_kits(", "}\n}");

    for required in [
        "Self::kit_store_installed_visible_rows(kits, filter).len()",
        "Self::kit_store_installed_visible_rows(&kits, &filter_owned)",
        "Self::kit_store_installed_selected_visible_kit(\n                                kits,\n                                filter,",
    ] {
        assert!(
            body.contains(required),
            "Installed Kits navigation/action paths must use filtered visible rows: {required}"
        );
    }

    assert!(
        !body.contains("*selected_index < kits.len().saturating_sub(1)"),
        "Installed Kits arrow navigation must not use raw kit count"
    );
}
