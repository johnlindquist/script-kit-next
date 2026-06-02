use std::{collections::HashSet, fs};

use script_kit_gpui::designs::{
    MainMenuInputTextAlignment, MainMenuLogoPlacement, MainMenuThemeVariant,
};

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn all_header_info_bar_variants_preserve_base_geometry() {
    let signatures = MainMenuThemeVariant::all()
        .iter()
        .map(|theme| theme.geometry_signature())
        .collect::<HashSet<_>>();

    assert_eq!(
        signatures.len(),
        1,
        "header variants must keep the current base geometry and vary only the information bar"
    );
}

#[test]
fn header_info_bar_variants_have_unique_header_signatures() {
    let signatures = MainMenuThemeVariant::all()
        .iter()
        .map(|theme| theme.header_info_bar_signature())
        .collect::<HashSet<_>>();

    assert_eq!(
        signatures.len(),
        MainMenuThemeVariant::COUNT,
        "each variation slot should explore a distinct header information-bar idea"
    );
}

#[test]
fn all_header_variations_keep_the_screenshot_base_layout() {
    for theme in MainMenuThemeVariant::all() {
        let tokens = theme.def().header_info_bar;
        assert_eq!(tokens.logo_placement, MainMenuLogoPlacement::InputLeading);
        assert_eq!(
            tokens.input_text_alignment,
            MainMenuInputTextAlignment::RowTextColumn
        );
        assert!(!tokens.hide_initial_section_header);
        assert!(tokens.show_cwd);
        assert!(tokens.show_agent_model);
    }
}

#[test]
fn script_list_consumes_shell_search_and_theme_list_metrics() {
    let source = read_source("src/render_script_list/mod.rs");

    assert!(source.contains("let menu_def = menu_theme.def();"));
    assert!(source.contains("let shell = menu_def.shell;"));
    assert!(source.contains("let search = menu_def.search;"));
    assert!(source.contains("render_main_view_shell()"));
    assert!(source.contains("render_main_view_chrome"));
    assert!(source.contains("MainViewChrome"));
    assert!(source.contains("MainViewHeaderChrome"));
    assert!(source.contains("MainViewDividerChrome"));
    assert!(source.contains("render_main_view_input_shell"));
    assert!(source.contains("effective_list_item_height_for_theme"));
    assert!(source.contains("effective_section_header_height_for_theme"));
    assert!(source.contains("effective_average_item_height_for_scroll_for_theme"));
}

#[test]
fn selector_copy_is_header_oriented_not_theme_oriented() {
    for variant in MainMenuThemeVariant::all() {
        assert!(variant.placeholder().contains("Header"));
        assert!(!variant.placeholder().contains("Theme"));
        let def = variant.def();
        assert_eq!(def.header_info_bar.font_family, "JetBrains Mono");
        assert_eq!(def.header_info_bar.font_size, 10.5);
        assert_eq!(def.header_info_bar.opacity, 0.34);
        assert!(def.header_info_bar.show_cwd);
        assert!(def.header_info_bar.show_agent_model);
        assert!(def.shell.header_padding_y <= 4.0);
        assert!(def.shell.header_gap <= 4.0);
    }
}

#[test]
fn main_menu_variant_slots_are_header_info_bar_slots() {
    let source = read_source("src/designs/core/main_menu_theme.rs");
    let variant_enum = source
        .split("pub enum MainMenuThemeVariant")
        .nth(1)
        .and_then(|tail| {
            tail.split("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
                .next()
        })
        .expect("MainMenuThemeVariant enum body should be readable");

    for old_name in [
        "TahoeClear",
        "TahoeGraphite",
        "TahoeBlueGlass",
        "FrostedCommand",
        "LiquidPrism",
        "CarbonNeon",
        "OperatorMonoGlass",
    ] {
        assert!(
            !variant_enum.contains(old_name),
            "old visual theme slot {old_name} should not remain in MainMenuThemeVariant"
        );
    }

    assert_eq!(
        variant_enum.matches("InfoBar").count(),
        MainMenuThemeVariant::COUNT
    );
}

#[test]
fn shared_main_view_columns_are_cross_theme_source_of_truth() {
    let shared = read_source("src/components/main_view_chrome.rs");

    assert!(shared.contains("pub(crate) fn main_view_content_columns"));
    assert!(shared.contains("pub(crate) fn main_view_text_column_x"));
    assert!(shared.contains("pub(crate) fn main_view_should_show_state_icon"));
    assert!(shared.contains(
        "main_view_row_leading_x(def) + main_view_state_icon_slot_size(def) + def.row.icon_text_gap"
    ));
    assert!(shared.contains("MainMenuLogoPlacement::InputLeading"));
    assert!(shared.contains("def.icon.container_size.min(def.search.height).max(16.0)"));
    assert!(shared.contains("(text_column_x - def.shell.header_padding_x)"));
    assert!(shared.contains(".max(def.search.text_inset_x)"));
}

#[test]
fn list_item_uses_main_menu_theme_metric_override() {
    let source = read_source("src/list_item/mod.rs");

    assert!(source.contains("ListItemMetricsOverride::from_main_menu_theme"));
    assert!(source.contains("self.main_menu_theme.def().row_kind"));
    assert!(source.contains("metrics.row_inner_padding_x"));
    assert!(source.contains("metrics.row_radius"));
    assert!(source.contains("metrics.icon_tile_size"));
    assert!(source.contains("metrics.accessory_gap"));
}

#[test]
fn footer_and_agent_hint_share_main_menu_footer_metrics() {
    let footer = read_source("src/components/footer_chrome.rs");
    let agent = read_source("src/components/launcher_ask_ai_hint.rs");

    assert!(footer.contains("current_main_menu_footer_metrics"));
    assert!(footer.contains("metrics.item_gap_px"));
    assert!(footer.contains("metrics.button_radius"));
    assert!(footer.contains("metrics.content_gap"));
    assert!(agent.contains("current_main_menu_footer_metrics"));
    assert!(agent.contains("footer_metrics.content_gap"));
    assert!(agent.contains("footer_metrics.button_radius"));
    assert!(agent.contains("render_footer_hint_content"));
}
