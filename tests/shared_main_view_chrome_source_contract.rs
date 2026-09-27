use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source
        .find(start)
        .unwrap_or_else(|| panic!("start marker not found: {start}"));
    let rest = &source[start_idx..];
    let end_idx = rest
        .find(end)
        .unwrap_or_else(|| panic!("end marker not found after {start}: {end}"));
    &rest[..end_idx]
}

#[test]
fn header_info_bar_reuses_footer_key_button_components() {
    let shared = read_source("src/components/main_view_chrome.rs");
    let footer = read_source("src/components/footer_chrome.rs");

    assert!(footer.contains("pub(crate) struct FooterHintButtonSpec"));
    assert!(footer.contains("pub(crate) fn render_footer_hint_button_like"));
    assert!(shared.contains("render_footer_hint_button_like"));
    assert!(shared.contains("FooterHintButtonSpec"));
    assert!(shared.contains("label: cwd_label.clone().into()"));
    assert!(shared.contains("key: if tab_key_active { \"⇥\" } else { \"\" }.into()"));
    assert!(shared.contains("agent_model_enabled && labels.shift_tab_key_active"));
    assert!(shared.contains("\"⇧⇥\""));
    assert!(!shared.contains("key: \"Tab\".into()"));
    assert!(!shared.contains("key: \"Shift+Tab\".into()"));
    assert!(shared.contains("slot_width_px: None"));
    assert!(!shared.contains("slot_width_px: Some(280.0)"));
    assert!(!shared.contains("slot_width_px: Some(310.0)"));
    assert!(shared.contains("label_font_size_px: Some(info.font_size)"));
    assert!(shared.contains("keycap_font_size_px: Some(header_keycap_font_size)"));
    assert!(
        shared.contains(".opacity(info.key_opacity.clamp(0.0, 1.0))"),
        "header key affordance may vary opacity but must wrap the shared footer renderer"
    );
    assert!(
        !shared.contains(".text_color(key_color)") && !shared.contains(".child(\"Shift+Tab\"),"),
        "header keys must not carry bespoke color/weight/text rendering that can drift from footer key buttons"
    );
}

#[test]
fn shared_main_view_columns_own_text_column_math() {
    let shared = read_source("src/components/main_view_chrome.rs");

    assert!(
        shared.contains("def.row.outer_padding_x + def.row.inner_padding_x"),
        "row leading x should come from the same row padding used by list rows"
    );
    assert!(
        shared.contains(
            "main_view_row_leading_x(def) + def.icon.container_size + def.row.icon_text_gap"
        ),
        "text column math should stay available for rows without leaking icons into the input chrome"
    );
    assert!(
        shared.contains("def.search.text_inset_x"),
        "input text inset should come directly from shared search tokens"
    );
    assert!(shared.contains("text_column_x"));
    assert!(
        shared.contains("content_right_inset_x: def.shell.header_padding_x"),
        "main-view info/content surfaces should share the shell right inset"
    );
    assert!(
        shared.contains("top_inset_y: def.list.first_section_header_height"),
        "main-view info/content surfaces should start on the same vertical rhythm as first list content"
    );
    assert!(
        !shared.contains("main_view_state_icon_left"),
        "MainMenuInput must not carry a leading state icon positioner"
    );
    assert!(
        shared.contains(".pl(px(text_inset_left))"),
        "input text should use the shared tokenized inset"
    );
}

#[test]
fn layout_model_exposes_shared_main_view_chrome_names() {
    let layout = read_source("src/app_layout/build_layout_info.rs");

    for name in [
        "MainViewHeader",
        "MainViewContextZone",
        "MainViewInput",
        "MainViewMain",
        "MainViewFooter",
    ] {
        assert!(layout.contains(name), "layout model missing {name}");
    }

    assert!(layout.contains(".with_parent(\"MainViewHeader\")"));
    assert!(layout.contains(".with_parent(\"MainViewMain\")"));
    assert!(
        !layout.contains("LayoutComponentInfo::new(\"SearchInput\""),
        "layout model should report the shared MainViewInput chrome name"
    );
    for stale_name in ["LogoButton", "ActionsButton", "RunButton"] {
        assert!(
            !layout.contains(&format!("LayoutComponentInfo::new(\"{stale_name}\"")),
            "layout model should not report stale header button chrome after MainViewInput became the shared full-width header"
        );
    }
    assert!(
        layout.contains(
            "let input_width = (main_width - (shell_horizontal_padding * 2.0)).max(0.0);"
        ),
        "layout model should derive MainViewInput width from shared header padding, not stale button edges"
    );
    assert!(
        layout.contains("main_view_input_text_inset_left("),
        "layout model should report the shared input text inset used by the render layer"
    );
    assert!(
        !layout.contains("MainViewInputStateIcon"),
        "layout model should not report a left state icon inside MainMenuInput"
    );
}

#[test]
fn main_view_layout_models_use_shared_context_chrome() {
    let layout = read_source("src/app_layout/build_layout_info.rs");
    let bounds = read_source("src/app_layout/build_component_bounds.rs");

    assert!(layout.contains("main_view_header_input_policy()"));
    assert!(layout.contains("main_view_header_metrics(menu_def, input_height)"));
    assert!(layout.contains("self.current_view.native_footer_surface().is_some()"));
    assert!(layout.contains("\"MainViewHeader\""));
    assert!(layout.contains("\"MainViewContextZone\""));
    assert!(layout.contains("\"MainViewInput\""));
    assert!(layout.contains("\"MainViewMain\""));
    assert!(
        bounds.contains("self.build_layout_info(") && !bounds.contains("main_view_header_metrics("),
        "debug-grid bounds must project the layout model instead of owning main-view geometry"
    );

    let shared_footer = layout
        .find("LayoutComponentInfo::new(\"MainViewFooter\"")
        .expect("layout model must expose the shared native footer");
    let first_detailed_early_return = layout
        .find("if matches!(self.current_view, AppView::PermissionsWizardView")
        .expect("permissions detailed receipt must remain present");
    assert!(
        shared_footer < first_detailed_early_return,
        "MainViewFooter must be emitted before view-specific early returns"
    );

    let permissions = source_between(
        &layout,
        "if matches!(self.current_view, AppView::PermissionsWizardView",
        "if matches!(self.current_view, AppView::About",
    );
    assert!(permissions.contains("let title_y = content_top + frame.inset_y;"));
    assert!(permissions.contains(".with_parent(\"MainViewMain\")"));
}

#[test]
fn main_window_footer_keeps_header_context_out_of_action_zone() {
    let ui_window = read_source("src/app_impl/ui_window.rs");
    let standard_footer = ui_window
        .split("fn standard_main_window_footer_buttons")
        .nth(1)
        .and_then(|tail| tail.split("fn main_window_footer_buttons_blocked").next())
        .expect("standard_main_window_footer_buttons body should be present");
    assert!(standard_footer.contains("main_window_run_footer_button("));
    assert!(standard_footer.contains("FooterAction::Actions"));
    assert!(standard_footer.contains("FooterAction::Ai"));
    assert!(standard_footer.contains("matches!(self.current_view, AppView::ScriptList)"));
    assert!(
        !standard_footer.contains("FooterAction::Cwd")
            && !standard_footer.contains("FooterAction::AgentModel"),
        "cwd/model context should render in MainViewContextZone, not as footer action buttons"
    );

    let footer_config = ui_window
        .split("pub(crate) fn main_window_footer_config_with_cx")
        .nth(1)
        .and_then(|tail| {
            tail.split("pub(crate) fn main_window_uses_native_footer")
                .next()
        })
        .expect("main_window_footer_config_with_cx body should be present");
    assert!(
        !footer_config.contains("prepend_global_main_window_left_chips")
            && !footer_config.contains("FooterAction::Cwd")
            && !footer_config.contains("FooterAction::AgentModel"),
        "main-window footer config should not inject duplicated cwd/model footer chips"
    );
}
