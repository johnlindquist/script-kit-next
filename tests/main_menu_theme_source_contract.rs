use std::{collections::HashSet, fs};

use gpui::FontWeight;
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
fn all_header_variations_keep_logo_free_numbered_header_layout() {
    for theme in MainMenuThemeVariant::all() {
        let tokens = theme.def().header_info_bar;
        assert_eq!(tokens.logo_placement, MainMenuLogoPlacement::Hidden);
        assert_eq!(
            tokens.input_text_alignment,
            MainMenuInputTextAlignment::RowTextColumn
        );
        assert!(!tokens.hide_initial_section_header);
        assert!(tokens.show_cwd);
        assert!(tokens.show_agent_model);
    }
}

/// The header context zone must NOT render the old centered variation badge
/// (the theme-exploration reference number); the exploration cycle has been
/// retired in favor of the settled design.
#[test]
fn header_context_does_not_render_variation_badge() {
    let shared = read_source("src/components/main_view_chrome.rs");

    assert!(!shared.contains("MAIN_VIEW_CONTEXT_VARIATION_BADGE_ID"));
    assert!(!shared.contains("variation_badge"));
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
        let def = variant.def();
        assert_eq!(def.header_info_bar.font_family, "JetBrains Mono");
        assert_eq!(def.header_info_bar.font_size, 10.5);
        assert_eq!(def.header_info_bar.opacity, 0.34);
        assert!(def.header_info_bar.show_cwd);
        assert!(def.header_info_bar.show_agent_model);
        assert!(def.shell.header_padding_y <= 4.0);
        assert!(def.shell.header_gap <= 4.0);
        assert_eq!(def.shell.divider_height, 0.0);
        // Commit ee4a244f3 ("Prepare UI polish and release CI gates") gave the
        // first section header 4px of extra breathing room beyond the derived
        // header-padding + 16px section line height + section-padding sum.
        assert_eq!(
            def.list.first_section_header_height,
            def.shell.header_padding_y + 16.0 + def.list.section_padding_bottom + 4.0
        );
    }
}

#[test]
fn main_menu_theme_owns_header_context_and_section_geometry_defaults() {
    let def = MainMenuThemeVariant::InfoBarBase.def();

    assert_eq!(def.header_info_bar.context_edge_outset_x, 8.0);
    assert_eq!(def.list.section_padding_x, 14.0);
    assert_eq!(def.list.section_padding_top, 12.0);
    assert_eq!(def.list.section_padding_bottom, 4.0);
    assert_eq!(def.list.section_gap, 6.0);
    assert_eq!(def.list.section_icon_size, 10.0);
    assert_eq!(def.typography.section_weight, FontWeight::SEMIBOLD);
}

#[test]
fn base_header_borrows_low_contrast_vertical_spacing_without_shell_geometry_drift() {
    let base = MainMenuThemeVariant::InfoBarBase.def();
    let low = MainMenuThemeVariant::InfoBarLowContrastKeys.def();

    assert_eq!(
        base.header_info_bar.height_px,
        low.header_info_bar.height_px
    );
    assert!(base.header_info_bar.show_keys);
    assert_eq!(base.header_info_bar.gap_px, 7.0);
    // Commit ee4a244f3 ("Prepare UI polish and release CI gates") gave the
    // base info-bar pills 6px of horizontal padding.
    assert_eq!(base.header_info_bar.pill_padding_x, 6.0);
    assert_eq!(base.header_info_bar.pill_padding_y, 0.0);
    assert_eq!(base.header_info_bar.pill_border_alpha, 0x00);
    assert_eq!(base.header_info_bar.pill_bg_alpha, 0x00);
    assert_eq!(
        MainMenuThemeVariant::InfoBarBase.geometry_signature(),
        MainMenuThemeVariant::InfoBarLowContrastKeys.geometry_signature()
    );
}

#[test]
fn first_section_separator_padding_matches_header_bottom_padding() {
    let list_item = read_source("src/list_item/mod.rs");

    assert!(list_item.contains("pub first_section_padding_top: f32"));
    assert!(list_item.contains("first_section_padding_top: def.shell.header_padding_y"));
    assert!(list_item.contains("metrics.first_section_padding_top"));
    assert!(list_item.contains("if is_first"));
    assert!(list_item.contains("header.justify_start()"));
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

    for variant in MainMenuThemeVariant::all() {
        let name = format!("{variant:?}");
        assert!(
            name.starts_with("InfoBar"),
            "every MainMenuThemeVariant slot must be a header info-bar slot; {name} is not"
        );
        assert!(
            variant_enum.contains(&name),
            "MainMenuThemeVariant enum body must declare the {name} slot"
        );
    }
}

#[test]
fn shared_main_view_columns_are_cross_theme_source_of_truth() {
    let shared = read_source("src/components/main_view_chrome.rs");

    assert!(shared.contains("pub(crate) fn main_view_content_columns"));
    assert!(shared.contains("pub(crate) fn main_view_text_column_x"));
    assert!(!shared.contains("pub(crate) fn main_view_should_show_state_icon"));
    assert!(shared.contains(
        "main_view_row_leading_x(def) + def.icon.container_size + def.row.icon_text_gap"
    ));
    assert!(!shared.contains("main_view_state_icon_uses_script_kit_logo"));
    assert!(shared.contains("pub(crate) fn main_view_input_text_inset_left"));
    assert!(shared.contains("def.search.text_inset_x"));
    assert!(!shared.contains("has_leading"));
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
