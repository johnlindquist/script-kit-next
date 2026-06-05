//! Source-level contract for App Launcher shared main-search migration.
//!
//! App Launcher should use the same GPUI search input and shared row chrome as
//! the main filterable builtins. Its key handler owns selection/launch only;
//! text edits flow through `InputEvent::Change`.

const APP_LAUNCHER: &str = include_str!("../src/render_builtins/app_launcher.rs");
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
fn app_launcher_uses_shared_search_input_and_rows() {
    let render_body = source_between(APP_LAUNCHER, "fn render_app_launcher(", "\n#[cfg(test)]");

    for required in [
        "self.render_search_input()",
        "ListItem::new(",
        "ListItemColors::from_theme(&self.theme)",
        ".selected(is_selected)",
        ".hovered(is_hovered)",
        ".main_menu_theme(",
        ".with_accent_bar(true)",
        "builtin_uniform_list_scrollbar(&self.list_scroll_handle",
        "render_minimal_list_prompt_shell_with_footer(",
        ".key_context(\"app_launcher\")",
        ".track_focus(&self.focus_handle)",
        ".on_key_down(handle_key)",
        "PromptChromeAudit::minimal_list(",
    ] {
        assert!(
            render_body.contains(required),
            "App Launcher render must keep shared main-list pattern: {required}"
        );
    }

    for forbidden in [
        "input_display",
        "input_is_empty",
        "Search input with blinking cursor",
        "CURSOR_WIDTH",
        "CURSOR_HEIGHT_LG",
        "CURSOR_MARGIN_Y",
        "CURSOR_GAP_X",
        ".ml(px(-(CURSOR_WIDTH + CURSOR_GAP_X)))",
        "text_muted = self.theme.colors.text.muted",
    ] {
        assert!(
            !render_body.contains(forbidden),
            "App Launcher render must not keep bespoke input chrome: {forbidden}"
        );
    }
}

#[test]
fn app_launcher_shared_input_drives_filter_state() {
    let input_change_body = source_between(
        STARTUP,
        "InputEvent::Change =>",
        "\n                InputEvent::PressEnter",
    );

    for required in [
        "AppView::AppLauncherView",
        "*filter != current_value",
        "*filter = current_value",
        "*selected_index = 0",
        "this.list_scroll_handle",
        ".scroll_to_item(0, gpui::ScrollStrategy::Nearest)",
        "cx.notify()",
    ] {
        assert!(
            input_change_body.contains(required),
            "InputEvent::Change must route App Launcher shared input: {required}"
        );
    }
}

#[test]
fn app_launcher_key_handler_does_not_duplicate_text_input() {
    let key_handler_body = source_between(
        APP_LAUNCHER,
        "let handle_key = cx.listener(",
        "\n        let color_resolver",
    );

    for required in [
        "is_key_up(key)",
        "is_key_down(key)",
        "is_key_enter(key)",
        "is_key_escape(key)",
    ] {
        assert!(
            key_handler_body.contains(required),
            "App Launcher key handler must retain navigation/launch/escape handling: {required}"
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
            "App Launcher key handler must not duplicate shared text input: {forbidden}"
        );
    }
}

#[test]
fn app_launcher_global_arrow_navigation_uses_filtered_rows() {
    for (label, source) in [
        ("startup.rs", STARTUP),
        ("startup_new_arrow.rs", STARTUP_NEW_ARROW),
    ] {
        let arrow_body = source_between(
            source,
            "AppView::AppLauncherView {\n                                    selected_index,",
            "\n                                AppView::WindowSwitcherView",
        );

        assert!(
            arrow_body.contains("let filtered_len =")
                && arrow_body.contains("Self::app_launcher_filtered_entries(&this.apps, filter)")
                && arrow_body.contains(".len();"),
            "{label} App Launcher global arrow branch must bound navigation by visible filtered apps"
        );
        assert!(
            !arrow_body.contains("let filtered_len = this.apps.len();"),
            "{label} App Launcher global arrow branch must not use the raw app dataset length"
        );
    }
}

#[test]
fn app_launcher_simulate_key_navigation_uses_filtered_rows() {
    let simulate_body = source_between(
        SIMULATE_KEY_DISPATCH,
        "AppView::AppLauncherView { .. } =>",
        "\n                AppView::PathPrompt",
    );

    for required in [
        "SimulateKey: Dispatching",
        "AppLauncherView",
        "Self::app_launcher_filtered_entries(&view.apps, &filter).len()",
        "view.list_scroll_handle",
        ".scroll_to_item(new_index, ScrollStrategy::Nearest)",
        "view.input_mode = InputMode::Keyboard",
        "view.hovered_index = None",
        "ctx.notify()",
    ] {
        assert!(
            simulate_body.contains(required),
            "simulateKey App Launcher navigation must use shared filtered rows: {required}"
        );
    }

    assert!(
        !simulate_body.contains("view.apps.len()"),
        "simulateKey App Launcher navigation must not use the raw app dataset length"
    );
}
