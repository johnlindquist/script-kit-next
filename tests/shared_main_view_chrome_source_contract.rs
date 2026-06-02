use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn script_list_and_acp_use_shared_main_view_input_shell() {
    let shared = read_source("src/components/main_view_chrome.rs");
    let script_list = read_source("src/render_script_list/mod.rs");
    let acp = read_source("src/ai/acp/view.rs");

    assert!(shared.contains("pub(crate) fn render_main_view_input_shell"));
    assert!(shared.contains("pub(crate) fn render_main_view_header"));
    assert!(shared.contains("pub(crate) fn render_main_view_header_divider"));
    assert!(shared.contains("pub(crate) fn render_main_view_main_slot"));
    assert!(shared.contains("pub(crate) fn render_main_view_shell"));
    assert!(shared.contains("pub(crate) fn main_view_input_text_inset_left"));
    assert!(shared.contains("pub(crate) fn render_main_view_state_icon"));
    assert!(shared.contains("pub(crate) fn render_main_view_context_zone"));
    assert!(shared.contains("pub(crate) fn main_view_state_icon_left"));
    assert!(shared.contains("pub(crate) fn render_main_view_chrome"));
    assert!(shared.contains("pub(crate) struct MainViewInputChrome"));
    assert!(shared.contains("pub(crate) struct MainViewHeaderChrome"));
    assert!(shared.contains("pub(crate) struct MainViewDividerChrome"));
    assert!(shared.contains("pub(crate) struct MainViewChrome"));
    assert!(shared.contains("pub(crate) struct MainViewColumnMetrics"));
    assert!(shared.contains("pub(crate) fn main_view_content_columns"));
    assert!(shared.contains("pub(crate) fn main_view_text_column_x"));
    assert!(shared.contains("main_view_state_icon_slot_size(def)"));
    assert!(
        !shared.contains("pre_main") && !shared.contains("post_main"),
        "shared main-view chrome should expose exactly one main slot so ScriptList and Agent Chat only swap the main section"
    );
    assert!(shared.contains("MAIN_VIEW_SHELL_ID"));
    assert!(shared.contains("MAIN_VIEW_HEADER_ID"));
    assert!(shared.contains("MAIN_VIEW_CONTEXT_ZONE_ID"));
    assert!(shared.contains("MAIN_VIEW_CONTEXT_LOGO_ID"));
    assert!(shared.contains("MAIN_VIEW_CONTEXT_CWD_BUTTON_ID"));
    assert!(shared.contains("MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID"));
    assert!(shared.contains("MAIN_VIEW_CONTEXT_VARIATION_BADGE_ID"));
    assert!(shared.contains("MAIN_VIEW_INPUT_SHELL_ID"));
    assert!(shared.contains("MAIN_VIEW_INPUT_STATE_ICON_ID"));
    assert!(shared.contains("MAIN_VIEW_HEADER_DIVIDER_ID"));
    assert!(shared.contains("MAIN_VIEW_MAIN_ID"));
    assert!(shared.contains(".id(MAIN_VIEW_HEADER_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_CONTEXT_ZONE_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_CONTEXT_LOGO_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_CONTEXT_CWD_BUTTON_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_CONTEXT_MODEL_BUTTON_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_CONTEXT_VARIATION_BADGE_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_SHELL_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_INPUT_SHELL_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_INPUT_STATE_ICON_ID)"));
    assert!(shared.contains(".absolute()"));
    assert!(shared.contains("main_view_state_icon_left(def)"));
    assert!(shared.contains("\"/assets/logo.svg\""));
    assert!(shared.contains("main_view_state_icon_uses_script_kit_logo"));
    assert!(shared.contains("main_view_should_show_state_icon"));
    assert!(shared.contains("MainMenuLogoPlacement::InputLeading"));
    assert!(shared.contains("theme.colors.accent.selected"));
    assert!(
        shared.contains("\"search\" | \"find\" | \"magnifyingglass\""),
        "default search icon names should resolve to the accent-tinted Script Kit logo"
    );
    assert!(shared.contains(".id(MAIN_VIEW_HEADER_DIVIDER_ID)"));
    assert!(shared.contains(".id(MAIN_VIEW_MAIN_ID)"));
    assert!(shared.contains("def.search"));
    assert!(shared.contains("search.text_inset_x"));
    assert!(shared.contains("def.icon.container_size"));
    assert!(shared.contains("def.row.icon_text_gap"));
    assert!(shared.contains("def.header_info_bar"));
    assert!(shared.contains("render_footer_hint_button_like"));
    assert!(shared.contains("FooterHintButtonSpec"));
    assert!(shared.contains(".h(px(info.height_px))"));
    assert!(shared.contains("(def.variant.index() + 1).to_string()"));
    assert!(shared.contains(".w(px(32.0))"));
    assert!(shared.contains("def.footer.button.hover"));
    assert!(!shared.contains("theme.colors.accent.selected << 8"));
    assert!(
        !shared.contains(
            "font_weight(gpui::FontWeight::SEMIBOLD)\n                        .child(\"Tab\")"
        ),
        "header Tab text should use the footer key/button renderer instead of local styling"
    );

    assert!(script_list.contains("render_main_view_input_shell"));
    assert!(script_list.contains("render_main_view_context_zone"));
    assert!(script_list.contains("render_main_view_state_icon"));
    assert!(script_list.contains("main_view_should_show_state_icon"));
    assert!(script_list.contains("hide_initial_section_header"));
    assert!(script_list.contains("main_view_state_icon_name_for_script_list"));
    assert!(script_list.contains("render_main_view_shell()"));
    assert!(script_list.contains("render_main_view_chrome"));
    assert!(script_list.contains("MainViewInputChrome"));
    assert!(script_list.contains("MainViewHeaderChrome"));
    assert!(script_list.contains("context: Some("));
    assert!(script_list.contains("FooterAction::Cwd"));
    assert!(script_list.contains("FooterAction::AgentModel"));
    assert!(script_list.contains("trailing: Vec::new()"));
    assert!(
        !script_list.contains("render_launcher_ask_ai_hint"),
        "ScriptList input should stay query-only; Agent belongs in the footer action zone"
    );
    assert!(script_list.contains("MainViewDividerChrome"));
    assert!(script_list.contains("MainViewChrome"));
    assert!(
        script_list.contains("let header_padding_x = shell.header_padding_x;"),
        "ScriptList should source left/right main-view input padding from the active theme shell"
    );
    assert!(
        script_list.contains("margin_x: shell.divider_margin_x"),
        "ScriptList divider should align with the active theme shell inset"
    );
    assert!(acp.contains("render_main_view_input_shell"));
    assert!(acp.contains("render_main_view_state_icon"));
    assert!(acp.contains("render_main_view_shell()"));
    assert!(acp.contains("render_main_view_chrome"));
    assert!(acp.contains("MainViewInputChrome"));
    assert!(acp.contains("MainViewHeaderChrome"));
    assert!(acp.contains("context: None"));
    assert!(acp.contains("MainViewDividerChrome"));
    assert!(acp.contains("MainViewChrome"));
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
    assert!(shared.contains("key: \"Tab\".into()"));
    assert!(shared.contains("key: \"Shift+Tab\".into()"));
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
            "main_view_row_leading_x(def) + main_view_state_icon_slot_size(def) + def.row.icon_text_gap"
        ),
        "input text column should be row leading plus rendered state-icon slot plus icon/text gap"
    );
    assert!(
        shared.contains("def.icon.container_size.min(def.search.height).max(16.0)"),
        "state icon slot must match the rendered/clamped logo size so the input placeholder does not drift right"
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
        shared.contains("(main_view_row_leading_x(def) - def.shell.header_padding_x).max(0.0)"),
        "state icon should align with the row icon column without shifting the input text column"
    );
    assert!(
        shared.contains(".pl(px(text_inset_left))"),
        "input text should keep using the text-column inset even when a state icon is present"
    );
}

#[test]
fn main_view_state_icon_maps_script_list_sources() {
    let script_list = read_source("src/render_script_list/mod.rs");

    assert!(script_list.contains("fn main_view_state_icon_name_for_script_list"));
    for (source, icon) in [
        ("RootUnifiedSourceFilter::Files", "\"folder-open\""),
        ("RootUnifiedSourceFilter::ClipboardHistory", "\"clipboard\""),
        (
            "RootUnifiedSourceFilter::Conversations",
            "\"message-circle\"",
        ),
        ("RootUnifiedSourceFilter::Scripts", "\"code\""),
        ("RootUnifiedSourceFilter::Commands", "\"code\""),
    ] {
        assert!(
            script_list.contains(source) && script_list.contains(icon),
            "script-list input state icon mapping missing {source} -> {icon}"
        );
    }
    assert!(script_list.contains("trimmed.starts_with('~')"));
    assert!(script_list.contains("positive_includes().next()"));
    assert!(script_list.contains("\"search\""));
}

#[test]
fn script_list_no_longer_owns_local_main_view_chrome() {
    let script_list = read_source("src/render_script_list/mod.rs");

    assert!(
        !script_list.contains(".id(\"main-menu-search-shell\")"),
        "ScriptList must use the shared main-view input shell instead of local search chrome"
    );
    assert!(
        !script_list.contains(".id(\"main-menu-shell\")"),
        "ScriptList must use the shared main-view root shell instead of local feature shell chrome"
    );
    assert!(
        !script_list.contains(".id(\"main-menu-header\")"),
        "ScriptList must use the shared main-view header instead of local header chrome"
    );
    assert!(
        !script_list.contains(".id(\"main-menu-header-divider\")"),
        "ScriptList must use the shared main-view divider instead of local divider chrome"
    );
    assert!(
        !script_list.contains(".id(\"main-menu-content\")"),
        "ScriptList must use the shared main-view main slot instead of local content chrome"
    );
}

#[test]
fn acp_composer_shell_consumes_main_menu_header_geometry() {
    let acp = read_source("src/ai/acp/view.rs");
    let ui_variant = read_source("src/ai/acp/ui_variant.rs");

    assert!(acp.contains("crate::designs::current_main_menu_theme().def()"));
    assert!(acp.contains("fn render_composer_input_shell"));
    assert!(acp.contains("render_main_view_input_shell"));
    assert!(acp.contains("render_main_view_header"));
    assert!(acp.contains("render_main_view_header_divider"));
    assert!(acp.contains("render_main_view_main_slot"));
    assert!(acp.contains("render_main_view_chrome"));
    assert!(acp.contains("let profile_icon = Self::render_input_profile_icon"));
    assert!(acp.contains("trailing: vec![profile_icon]"));
    assert!(acp.contains("padding_x: menu_def.shell.header_padding_x"));
    assert!(acp.contains("margin_x: menu_def.shell.divider_margin_x"));
    assert!(acp.contains("padding_y: menu_def.shell.header_padding_y"));
    assert!(
        !acp.contains(".id(\"agent-chat-shell\")"),
        "standard Agent Chat must use the shared main-view root shell instead of local feature shell chrome"
    );
    assert!(
        acp.contains(".id(\"acp-conversation\")"),
        "standard Agent Chat should wrap all conversation chrome inside one swapped main child"
    );
    assert!(
        ui_variant.contains("Self::Standard => AcpChatUiConfig {\n                transcript: AcpTranscriptPresentation::Standard,\n                composer: AcpComposerPlacement::Default"),
        "standard Agent Chat must stay on the default composer path that returns shared main-view chrome"
    );
}

#[test]
fn layout_model_exposes_shared_main_view_chrome_names() {
    let layout = read_source("src/app_layout/build_layout_info.rs");

    for name in [
        "MainViewHeader",
        "MainViewContextZone",
        "MainViewInput",
        "MainViewInputStateIcon",
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
            "let input_width = (window_width - (shell_horizontal_padding * 2.0)).max(0.0);"
        ),
        "layout model should derive MainViewInput width from shared header padding, not stale button edges"
    );
    assert!(
        layout.contains("main_view_input_text_inset_left("),
        "layout model should report the shared input text inset used by the render layer"
    );
    assert!(
        layout.contains("main_view_state_icon_left(menu_def)"),
        "layout model should report the shared state icon x used by the render layer"
    );
}

#[test]
fn acp_layout_model_swaps_only_main_section_to_conversation() {
    let layout = read_source("src/app_layout/build_layout_info.rs");

    assert!(
        layout.contains(
            "| AppView::AcpChatView { .. } => crate::window_resize::ViewType::MainWindow"
        ),
        "AcpChat should use the same stable main-window sizing target as the main menu chrome"
    );
    assert!(
        layout.contains("if matches!(self.current_view, AppView::AcpChatView { .. })"),
        "AcpChat needs its own layout branch before the launcher ScriptList fallback"
    );
    assert!(
        layout.contains("LayoutComponentInfo::new(\"AcpConversation\", LayoutComponentType::List)"),
        "AcpChat layout receipts should name the conversation as the swapped main section"
    );
    for name in [
        "AcpEmptyGuidance",
        "AcpEmptyGuidanceTitle",
        "AcpEmptyGuidanceBody",
        "AcpEmptyGuidanceShortcutSlot",
        "AcpEmptyGuidanceLabelColumn",
    ] {
        assert!(
            layout.contains(name),
            "AcpChat layout receipts should expose {name} for empty-state typography/spacing proof"
        );
    }
    assert!(layout.contains("main_view_content_columns(menu_def)"));
    assert!(
        layout.contains("with_parent(\"MainViewMain\")"),
        "AcpConversation should remain inside the shared MainViewMain slot"
    );
    assert!(
        layout.contains("} else {")
            && layout.contains(
                "// Script list: full width for MainWindow, left panel for split-preview surfaces."
            ),
        "AcpChat branch should not fall through to stale ScriptList/PreviewPanel layout components"
    );
}

#[test]
fn script_list_three_zone_footer_keeps_context_out_of_action_zone() {
    let ui_window = read_source("src/app_impl/ui_window.rs");
    let standard_footer = ui_window
        .split("fn standard_main_window_footer_buttons")
        .nth(1)
        .and_then(|tail| tail.split("fn main_window_footer_buttons_blocked").next())
        .expect("standard_main_window_footer_buttons body should be present");
    assert!(standard_footer.contains("FooterAction::Run"));
    assert!(standard_footer.contains("FooterAction::Actions"));
    assert!(standard_footer.contains("FooterAction::Ai"));
    assert!(standard_footer.contains("matches!(self.current_view, AppView::ScriptList)"));

    let global_left_chips = ui_window
        .split("fn current_view_shows_global_left_chips")
        .nth(1)
        .and_then(|tail| tail.split("fn global_main_window_left_chip_buttons").next())
        .expect("current_view_shows_global_left_chips body should be present");
    assert!(
        global_left_chips.contains("AppView::AcpChatView { .. }")
            && !global_left_chips.contains("AppView::ScriptList"),
        "ScriptList cwd/model context should render in MainViewContextZone, not as footer action buttons"
    );
}

#[test]
fn acp_component_bounds_model_uses_main_view_chrome() {
    let bounds = read_source("src/app_layout/build_component_bounds.rs");

    assert!(bounds.contains("\"MainViewHeader\""));
    assert!(bounds.contains("\"MainViewContextZone\""));
    assert!(bounds.contains("AppView::AcpChatView { .. } =>"));
    assert!(bounds.contains("\"MainViewMain\""));
    assert!(bounds.contains("\"AcpConversation\""));
    assert!(bounds.contains("\"MainViewInput\""));
    assert!(bounds.contains("\"MainViewInputStateIcon\""));
    assert!(bounds.contains("\"MainViewFooter\""));
    assert!(bounds.contains("\"AcpEmptyGuidance\""));
    assert!(bounds.contains("\"AcpEmptyGuidanceTitle\""));
    assert!(bounds.contains("\"AcpEmptyGuidanceShortcutSlot\""));
    assert!(bounds.contains("\"AcpEmptyGuidanceLabelColumn\""));
    assert!(bounds.contains("main_view_content_columns(menu_def)"));
    assert!(
        bounds.contains("AppView::ScriptList | AppView::AcpChatView { .. }"),
        "debug component bounds should emit shared input details for both ScriptList and AcpChat"
    );
}

#[test]
fn standard_agent_chat_mock_fixture_bypasses_provider_warmup() {
    let acp_launch = read_source("src/app_impl/tab_ai_mode/acp_launch.rs");
    let runtime_stdin = read_source("src/main_entry/runtime_stdin.rs");
    let app_run_setup = read_source("src/main_entry/app_run_setup.rs");
    let runtime_tail = read_source("src/main_entry/runtime_stdin_match_tail.rs");

    assert!(acp_launch.contains("fn open_standard_agent_chat_mock_fixture"));
    assert!(acp_launch.contains("StandardAgentChatMockFixtureConnection"));
    assert!(acp_launch.contains("AcpChatUiVariant::Standard"));
    assert!(
        acp_launch.contains("self.enter_embedded_acp_chat_surface(view_entity, cx);"),
        "fixture should use the same embedded ACP surface transition as real Agent Chat"
    );

    for (path, source) in [
        ("src/main_entry/runtime_stdin.rs", runtime_stdin),
        ("src/main_entry/app_run_setup.rs", app_run_setup),
        ("src/main_entry/runtime_stdin_match_tail.rs", runtime_tail),
    ] {
        assert!(
            source.contains("view.open_standard_agent_chat_mock_fixture(ctx);"),
            "{path} should open the deterministic standard Agent Chat fixture for openAiWithMockData"
        );
        assert!(
            !source.contains("Ignoring deprecated mock-data AI alias and opening Agent Chat"),
            "{path} should not route mock-data proof through provider warm-up"
        );
    }
}
