use std::{collections::HashSet, fs};

use script_kit_gpui::designs::MainMenuThemeVariant;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn all_main_menu_themes_have_unique_geometry_signatures() {
    let signatures = MainMenuThemeVariant::all()
        .iter()
        .map(|theme| theme.geometry_signature())
        .collect::<HashSet<_>>();

    assert_eq!(
        signatures.len(),
        MainMenuThemeVariant::COUNT,
        "main-menu themes must differ by geometry/typography, not just alpha"
    );
}

#[test]
fn theme_geometry_varies_across_several_token_families() {
    let defs = MainMenuThemeVariant::all()
        .iter()
        .map(|theme| theme.def())
        .collect::<Vec<_>>();

    let row_heights = defs
        .iter()
        .map(|def| def.list.item_height as u32)
        .collect::<HashSet<_>>();
    let search_heights = defs
        .iter()
        .map(|def| def.search.height as u32)
        .collect::<HashSet<_>>();
    let shell_insets = defs
        .iter()
        .map(|def| def.shell.content_inset_x as u32)
        .collect::<HashSet<_>>();
    let icon_sizes = defs
        .iter()
        .map(|def| def.icon.container_size as u32)
        .collect::<HashSet<_>>();
    let footer_gaps = defs
        .iter()
        .map(|def| def.footer.metrics.item_gap_px as u32)
        .collect::<HashSet<_>>();

    assert!(row_heights.len() >= 8);
    assert!(search_heights.len() >= 6);
    assert!(shell_insets.len() >= 8);
    assert!(icon_sizes.len() >= 6);
    assert!(footer_gaps.len() >= 5);
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
fn shared_main_view_columns_are_cross_theme_source_of_truth() {
    let shared = read_source("src/components/main_view_chrome.rs");

    assert!(shared.contains("pub(crate) fn main_view_content_columns"));
    assert!(shared.contains("pub(crate) fn main_view_text_column_x"));
    assert!(shared.contains(
        "main_view_row_leading_x(def) + def.icon.container_size + def.row.icon_text_gap"
    ));
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
