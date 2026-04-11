use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, StoryVariant};
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainMenuWeightSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub description: &'static str,
    pub search_weight: FontWeight,
    pub section_weight: FontWeight,
    pub title_weight: FontWeight,
    pub selected_title_weight: FontWeight,
    pub source_weight: FontWeight,
    pub chip_weight: FontWeight,
    pub kind_weight: FontWeight,
    pub search_opacity: f32,
    pub metadata_opacity: f32,
    pub kind_opacity: f32,
    pub selected_fill_opacity: f32,
}

#[derive(Clone, Copy)]
struct MenuPreviewRow {
    title: &'static str,
    source: &'static str,
    chip: Option<&'static str>,
    kind: &'static str,
    icon_label: &'static str,
    icon_bg: u32,
    selected: bool,
}

const ROWS: &[MenuPreviewRow] = &[
    MenuPreviewRow {
        title: "Clipboard History",
        source: "Raycast",
        chip: Some("ch"),
        kind: "Command",
        icon_label: "▣",
        icon_bg: 0xE15C4D,
        selected: true,
    },
    MenuPreviewRow {
        title: "Activity Monitor",
        source: "",
        chip: None,
        kind: "Application",
        icon_label: "⌗",
        icon_bg: 0x22342E,
        selected: false,
    },
    MenuPreviewRow {
        title: "Empty Trash",
        source: "System",
        chip: None,
        kind: "Command",
        icon_label: "⌫",
        icon_bg: 0x52565E,
        selected: false,
    },
    MenuPreviewRow {
        title: "AI Chat",
        source: "Raycast AI",
        chip: None,
        kind: "Command",
        icon_label: "✦",
        icon_bg: 0x2E1D32,
        selected: false,
    },
    MenuPreviewRow {
        title: "Raycast Notes",
        source: "Raycast Notes",
        chip: None,
        kind: "Command",
        icon_label: "T",
        icon_bg: 0xE26461,
        selected: false,
    },
    MenuPreviewRow {
        title: "My Passwords",
        source: "1Password",
        chip: None,
        kind: "Command",
        icon_label: "◔",
        icon_bg: 0xF2F2F2,
        selected: false,
    },
    MenuPreviewRow {
        title: "Generate Password",
        source: "1Password",
        chip: None,
        kind: "Command",
        icon_label: "◔",
        icon_bg: 0xF2F2F2,
        selected: false,
    },
    MenuPreviewRow {
        title: "My Vaults",
        source: "1Password",
        chip: None,
        kind: "Command",
        icon_label: "◔",
        icon_bg: 0xF2F2F2,
        selected: false,
    },
];

pub const SPECS: [MainMenuWeightSpec; 15] = [
    MainMenuWeightSpec { id: "raycast-baseline", name: "Raycast Baseline", family: "Balanced", description: "Medium search, quiet metadata, and a selected row that leans on fill before weight.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::MEDIUM, search_opacity: 0.56, metadata_opacity: 0.62, kind_opacity: 0.70, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "lighter-search", name: "Lighter Search", family: "Balanced", description: "Pulls the search line back so the first selected row does more of the visual work.", search_weight: FontWeight::NORMAL, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::MEDIUM, search_opacity: 0.48, metadata_opacity: 0.60, kind_opacity: 0.66, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "softer-sections", name: "Softer Sections", family: "Balanced", description: "Drops the section headers to regular weight so they whisper even more.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::NORMAL, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::NORMAL, kind_weight: FontWeight::MEDIUM, search_opacity: 0.54, metadata_opacity: 0.60, kind_opacity: 0.66, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "selected-contrast", name: "Selected Contrast", family: "Balanced", description: "Makes the selected label semibold while leaving the rest close to baseline.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::SEMIBOLD, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::MEDIUM, search_opacity: 0.56, metadata_opacity: 0.62, kind_opacity: 0.70, selected_fill_opacity: 0.10 },
    MainMenuWeightSpec { id: "all-normal", name: "All Normal", family: "Balanced", description: "Uses opacity and spacing almost exclusively, with only the section headers staying medium.", search_weight: FontWeight::NORMAL, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::NORMAL, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::NORMAL, kind_weight: FontWeight::NORMAL, search_opacity: 0.50, metadata_opacity: 0.58, kind_opacity: 0.62, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "vendor-forward", name: "Vendor Forward", family: "Metadata", description: "Promotes vendor/source labels so the list feels more app-directory-like.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::NORMAL, search_opacity: 0.56, metadata_opacity: 0.72, kind_opacity: 0.58, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "kind-quiet", name: "Kind Quiet", family: "Metadata", description: "Recedes the right-edge kind labels so they stop competing with the row title.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::NORMAL, search_opacity: 0.56, metadata_opacity: 0.62, kind_opacity: 0.46, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "chip-forward", name: "Chip Forward", family: "Metadata", description: "Lets the alias chip carry more of the accessory emphasis than the vendor text.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::SEMIBOLD, kind_weight: FontWeight::MEDIUM, search_opacity: 0.56, metadata_opacity: 0.60, kind_opacity: 0.68, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "whisper-metadata", name: "Whisper Metadata", family: "Metadata", description: "Keeps all secondary labels regular and dim so only the names really read.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::NORMAL, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::NORMAL, kind_weight: FontWeight::NORMAL, search_opacity: 0.54, metadata_opacity: 0.50, kind_opacity: 0.50, selected_fill_opacity: 0.11 },
    MainMenuWeightSpec { id: "directory-kind", name: "Directory Kind", family: "Metadata", description: "Uses stronger right-edge kind labels for a more catalog-like reading order.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::NORMAL, kind_weight: FontWeight::SEMIBOLD, search_opacity: 0.56, metadata_opacity: 0.58, kind_opacity: 0.74, selected_fill_opacity: 0.12 },
    MainMenuWeightSpec { id: "title-forward", name: "Title Forward", family: "Primary", description: "Pushes all row titles to medium while keeping metadata light, like a firmer launcher list.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::MEDIUM, selected_title_weight: FontWeight::SEMIBOLD, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::NORMAL, search_opacity: 0.56, metadata_opacity: 0.58, kind_opacity: 0.58, selected_fill_opacity: 0.10 },
    MainMenuWeightSpec { id: "selected-medium", name: "Selected Medium", family: "Primary", description: "Keeps even the selected title at medium so the highlight relies more on the filled row.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::NORMAL, search_opacity: 0.56, metadata_opacity: 0.56, kind_opacity: 0.56, selected_fill_opacity: 0.16 },
    MainMenuWeightSpec { id: "crisp-raycast", name: "Crisp Raycast", family: "Primary", description: "Leans closest to the reference: medium section labels, medium selected title, and restrained accessories.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::MEDIUM, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::MEDIUM, search_opacity: 0.58, metadata_opacity: 0.60, kind_opacity: 0.68, selected_fill_opacity: 0.14 },
    MainMenuWeightSpec { id: "semibold-hero", name: "Semibold Hero", family: "Primary", description: "Tests whether a semibold selected row feels too assertive for the otherwise quiet shell.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::SEMIBOLD, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::NORMAL, search_opacity: 0.56, metadata_opacity: 0.56, kind_opacity: 0.54, selected_fill_opacity: 0.14 },
    MainMenuWeightSpec { id: "metadata-split", name: "Metadata Split", family: "Primary", description: "Splits accessory weight: vendor regular, chip medium, kind medium for a more Raycast-like hierarchy.", search_weight: FontWeight::MEDIUM, section_weight: FontWeight::MEDIUM, title_weight: FontWeight::NORMAL, selected_title_weight: FontWeight::MEDIUM, source_weight: FontWeight::NORMAL, chip_weight: FontWeight::MEDIUM, kind_weight: FontWeight::MEDIUM, search_opacity: 0.56, metadata_opacity: 0.58, kind_opacity: 0.68, selected_fill_opacity: 0.13 },
];

pub fn main_menu_raycast_weight_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id, spec.name)
                .description(spec.description)
                .with_prop("surface", "mainMenu")
                .with_prop("family", spec.family)
                .with_prop("variantId", spec.id)
        })
        .collect()
}

pub fn render_main_menu_raycast_weight_story_preview(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), false)
}

pub fn render_main_menu_raycast_weight_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), true)
}

pub fn render_main_menu_raycast_weight_gallery() -> AnyElement {
    let theme = get_cached_theme();
    let mut root = story_container().gap_6().child(
        div().flex().flex_col().gap_1()
            .child(div().text_sm().text_color(theme.colors.text.tertiary.to_rgb()).child("Main Menu"))
            .child(div().text_xs().text_color(theme.colors.text.muted.to_rgb()).child(
                "Fifteen weight studies that keep the current launcher shell but move the typography hierarchy closer to Raycast.",
            )),
    );
    for family in ["Balanced", "Metadata", "Primary"] {
        let mut section = story_section(family).gap(px(12.0));
        for spec in SPECS.iter().copied().filter(|spec| spec.family == family) {
            section = section.child(render_gallery_item(spec));
        }
        root = root.child(section);
    }
    root.into_any_element()
}

fn resolve_spec(stable_id: &str) -> Option<MainMenuWeightSpec> {
    SPECS.iter().copied().find(|spec| spec.id == stable_id)
}

fn render_gallery_item(spec: MainMenuWeightSpec) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .bg(theme.colors.background.title_bar.with_opacity(0.22))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .rounded(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.colors.text.primary.to_rgb())
                        .child(spec.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.text.muted.to_rgb())
                        .child(spec.description),
                ),
        )
        .child(render_spec_stage(spec, false))
        .into_any_element()
}

fn render_spec_stage(spec: MainMenuWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 360.0 } else { 1188.0 };
    let height = if compact { 232.0 } else { 744.0 };
    div()
        .w(px(width))
        .h(px(height))
        .bg(theme.colors.background.main.with_opacity(0.28))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.14))
        .rounded(px(16.0))
        .overflow_hidden()
        .child(render_main_menu_shell(spec, compact))
        .into_any_element()
}

fn render_main_menu_shell(spec: MainMenuWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(theme.colors.background.main.to_rgb())
        .rounded(px(18.0))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.28))
        .child(render_header(spec, compact))
        .child(
            div()
                .mx(px(22.0))
                .h(px(1.0))
                .bg(theme.colors.ui.border.with_opacity(0.24)),
        )
        .child(render_rows(spec, compact))
        .child(render_footer(compact))
        .into_any_element()
}

fn render_header(spec: MainMenuWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let search_size = if compact { 15.0 } else { 23.0 };
    let ask_size = if compact { 12.0 } else { 16.0 };
    div()
        .w_full()
        .h(px(if compact { 54.0 } else { 84.0 }))
        .px(px(if compact { 14.0 } else { 22.0 }))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .w(px(2.0))
                        .h(px(if compact { 22.0 } else { 34.0 }))
                        .bg(get_cached_theme().colors.text.primary.with_opacity(0.70)),
                )
                .child(
                    div()
                        .text_size(px(search_size))
                        .font_weight(spec.search_weight)
                        .text_color(theme.colors.text.primary.with_opacity(spec.search_opacity))
                        .child("Search for apps and commands..."),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .text_size(px(ask_size))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text.primary.with_opacity(0.64))
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(if compact { 8.0 } else { 12.0 }))
                        .py(px(if compact { 4.0 } else { 6.0 }))
                        .rounded(px(9.0))
                        .border_1()
                        .border_color(theme.colors.ui.border.with_opacity(0.32))
                        .text_size(px(if compact { 11.0 } else { 14.0 }))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text.primary.with_opacity(0.60))
                        .child("Tab"),
                ),
        )
        .into_any_element()
}

fn render_rows(spec: MainMenuWeightSpec, compact: bool) -> AnyElement {
    let mut column = div()
        .w_full()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .overflow_hidden()
        .px(px(if compact { 12.0 } else { 22.0 }))
        .pt(px(if compact { 10.0 } else { 18.0 }))
        .gap(px(if compact { 4.0 } else { 6.0 }));
    column = column.child(render_section_header("Suggestions", spec, compact));
    for row in ROWS.iter().take(5).copied() {
        column = column.child(render_row(row, spec, compact));
    }
    column = column.child(render_section_header("Commands", spec, compact));
    for row in ROWS
        .iter()
        .skip(5)
        .take(if compact { 2 } else { 3 })
        .copied()
    {
        column = column.child(render_row(row, spec, compact));
    }
    column.into_any_element()
}

fn render_section_header(
    label: &'static str,
    spec: MainMenuWeightSpec,
    compact: bool,
) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .pt(px(if compact { 4.0 } else { 8.0 }))
        .pb(px(if compact { 2.0 } else { 4.0 }))
        .text_size(px(if compact { 11.0 } else { 14.0 }))
        .font_weight(spec.section_weight)
        .text_color(theme.colors.text.primary.with_opacity(0.62))
        .child(label)
        .into_any_element()
}

fn render_row(row: MenuPreviewRow, spec: MainMenuWeightSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let row_height = if compact { 34.0 } else { 62.0 };
    let icon_size = if compact { 20.0 } else { 28.0 };
    let title_size = if compact { 14.0 } else { 21.0 };
    let meta_size = if compact { 12.0 } else { 18.0 };
    let kind_size = if compact { 12.0 } else { 18.0 };
    let chip_size = if compact { 10.0 } else { 14.0 };
    div()
        .w_full()
        .h(px(row_height))
        .px(px(if compact { 10.0 } else { 14.0 }))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .rounded(px(if compact { 10.0 } else { 14.0 }))
        .when(row.selected, |d| {
            d.bg(theme
                .colors
                .text
                .primary
                .with_opacity(spec.selected_fill_opacity))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(if compact { 10.0 } else { 18.0 }))
                .child(
                    div()
                        .size(px(icon_size))
                        .rounded(px(if compact { 6.0 } else { 8.0 }))
                        .bg(rgb(row.icon_bg))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(if compact { 10.0 } else { 14.0 }))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if row.icon_bg == 0xF2F2F2 {
                            rgb(0x3478F6)
                        } else {
                            rgb(0xFFFFFF)
                        })
                        .child(row.icon_label),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(if compact { 8.0 } else { 14.0 }))
                        .child(
                            div()
                                .text_size(px(title_size))
                                .font_weight(if row.selected {
                                    spec.selected_title_weight
                                } else {
                                    spec.title_weight
                                })
                                .text_color(
                                    theme.colors.text.primary.with_opacity(if row.selected {
                                        0.96
                                    } else {
                                        0.90
                                    }),
                                )
                                .child(row.title),
                        )
                        .when(!row.source.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_size(px(meta_size))
                                    .font_weight(spec.source_weight)
                                    .text_color(
                                        theme
                                            .colors
                                            .text
                                            .primary
                                            .with_opacity(spec.metadata_opacity),
                                    )
                                    .child(row.source),
                            )
                        })
                        .when_some(row.chip, |d, chip| {
                            d.child(
                                div()
                                    .px(px(if compact { 6.0 } else { 10.0 }))
                                    .py(px(if compact { 3.0 } else { 5.0 }))
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(theme.colors.ui.border.with_opacity(0.34))
                                    .font_family(FONT_MONO)
                                    .text_size(px(chip_size))
                                    .font_weight(spec.chip_weight)
                                    .text_color(theme.colors.text.primary.with_opacity(0.58))
                                    .child(chip),
                            )
                        }),
                ),
        )
        .child(
            div()
                .text_size(px(kind_size))
                .font_weight(spec.kind_weight)
                .text_color(theme.colors.text.primary.with_opacity(spec.kind_opacity))
                .child(row.kind),
        )
        .into_any_element()
}

fn render_footer(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let text_size = if compact { 12.0 } else { 16.0 };
    div()
        .w_full()
        .h(px(if compact { 34.0 } else { 62.0 }))
        .px(px(if compact { 12.0 } else { 22.0 }))
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.24))
        .bg(theme.colors.background.title_bar.with_opacity(0.28))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(if compact { 14.0 } else { 24.0 }))
        .child(render_footer_item("Open Command", "↩", text_size))
        .child(render_footer_item("Actions", "⌘K", text_size))
        .into_any_element()
}

fn render_footer_item(label: &'static str, shortcut: &'static str, text_size: f32) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(text_size))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.text.primary.with_opacity(0.88))
                .child(label),
        )
        .child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(8.0))
                .bg(theme.colors.text.primary.with_opacity(0.08))
                .font_family(FONT_MONO)
                .text_size(px(text_size - 1.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.text.primary.with_opacity(0.58))
                .child(shortcut),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{main_menu_raycast_weight_story_variants, SPECS};

    #[test]
    fn main_menu_raycast_story_exposes_fifteen_variants() {
        assert_eq!(main_menu_raycast_weight_story_variants().len(), 15);
        assert_eq!(SPECS.len(), 15);
    }
}
