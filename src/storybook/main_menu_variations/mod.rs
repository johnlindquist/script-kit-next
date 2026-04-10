//! Main menu variation system for storybook.
//!
//! This module follows the same adoption contract as `notes_window_variations`:
//! typed `VariationId` → `AdoptableSurface` → `resolve_surface_live` → live id.

use std::{collections::HashSet, sync::OnceLock};

use gpui::{IntoElement as _, ParentElement as _, Styled as _};
use gpui_component::scroll::ScrollableElement as _;

use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
use super::StoryVariant;
use crate::ui_foundation::HexColorExt;

/// Stable IDs for adoptable Main Menu visual states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainMenuVariationId {
    CurrentMainMenu,
    EmptyState,
    SelectedResult,
}

impl MainMenuVariationId {
    pub const ALL: [Self; 3] = [
        Self::CurrentMainMenu,
        Self::EmptyState,
        Self::SelectedResult,
    ];
}

impl VariationId for MainMenuVariationId {
    fn as_str(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "current-main-menu",
            Self::EmptyState => "empty-state",
            Self::SelectedResult => "selected-result",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "Current Main Menu",
            Self::EmptyState => "Empty State",
            Self::SelectedResult => "Selected Result",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "Real launcher with populated search results",
            Self::EmptyState => "Real launcher chrome with no matching results",
            Self::SelectedResult => "Real launcher with a keyboard-focused result row",
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current-main-menu" => Some(Self::CurrentMainMenu),
            "empty-state" => Some(Self::EmptyState),
            "selected-result" => Some(Self::SelectedResult),
            _ => None,
        }
    }
}

/// Typed live-spec describing how the launcher should render for a given Main Menu variant.
///
/// These fields are consumed at render time via read-only local overrides — they must
/// never cause state mutation inside `render_script_list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuLiveSpec {
    /// When `true`, the list renders as empty regardless of actual script inventory.
    pub force_empty_results: bool,
    /// When `true`, the first real item (not a section header) gets keyboard focus.
    pub prefer_first_result_selected: bool,
    /// When set, overrides the filter text displayed in the empty-state body.
    pub filter_text_override: Option<&'static str>,
}

/// A Main Menu variation paired with its live-spec for adoption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainMenuVariationSpec {
    pub id: MainMenuVariationId,
    pub live: MainMenuLiveSpec,
}

pub const SPECS: [MainMenuVariationSpec; 3] = [
    MainMenuVariationSpec {
        id: MainMenuVariationId::CurrentMainMenu,
        live: MainMenuLiveSpec {
            force_empty_results: false,
            prefer_first_result_selected: true,
            filter_text_override: None,
        },
    },
    MainMenuVariationSpec {
        id: MainMenuVariationId::EmptyState,
        live: MainMenuLiveSpec {
            force_empty_results: true,
            prefer_first_result_selected: false,
            filter_text_override: Some("storybook-empty"),
        },
    },
    MainMenuVariationSpec {
        id: MainMenuVariationId::SelectedResult,
        live: MainMenuLiveSpec {
            force_empty_results: false,
            prefer_first_result_selected: true,
            filter_text_override: None,
        },
    },
];

pub struct MainMenuSurface;

impl AdoptableSurface for MainMenuSurface {
    type Id = MainMenuVariationId;
    type Spec = MainMenuVariationSpec;
    type Live = MainMenuLiveSpec;

    const STORY_ID: &'static str = "main-menu";
    const DEFAULT_ID: Self::Id = MainMenuVariationId::CurrentMainMenu;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        spec.id
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        spec.live
    }
}

pub fn main_menu_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "mainMenu")
                .with_prop("representation", "liveSurface")
                .with_prop("variantId", spec.id.as_str())
                .with_prop(
                    "forceEmptyResults",
                    spec.live.force_empty_results.to_string(),
                )
                .with_prop(
                    "preferFirstResultSelected",
                    spec.live.prefer_first_result_selected.to_string(),
                )
        })
        .collect()
}

pub fn resolve_main_menu_variant(
    selected: Option<&str>,
) -> (MainMenuLiveSpec, SurfaceSelectionResolution) {
    resolve_surface_live::<MainMenuSurface>(selected)
}

pub fn adopted_main_menu_variant() -> MainMenuVariationId {
    let selected = super::load_selected_story_variant(MainMenuSurface::STORY_ID);
    let (_, resolution) = resolve_surface_live::<MainMenuSurface>(selected.as_deref());
    MainMenuVariationId::from_stable_id(&resolution.resolved_variant_id)
        .unwrap_or(MainMenuSurface::DEFAULT_ID)
}

/// Resolve the current on-disk storybook selection into a typed `MainMenuLiveSpec`.
pub fn adopted_main_menu_live_spec() -> MainMenuLiveSpec {
    adopted_surface_live::<MainMenuSurface>()
}

pub fn render_main_menu_story_preview(stable_id: &str) -> gpui::AnyElement {
    render_main_menu_surface(stable_id, false)
}

pub fn render_main_menu_compare_thumbnail(stable_id: &str) -> gpui::AnyElement {
    render_main_menu_surface(stable_id, true)
}

#[derive(Clone)]
enum MainMenuPreviewEntry {
    Section { label: &'static str, count: usize },
    Row(MainMenuPreviewRow),
}

#[derive(Clone)]
struct MainMenuPreviewRow {
    title: String,
    subtitle: String,
    leading_icon: String,
    trailing_hint: String,
    primary_action_label: String,
}

fn render_main_menu_surface(stable_id: &str, compact: bool) -> gpui::AnyElement {
    let (live_spec, _) = resolve_main_menu_variant(Some(stable_id));
    let shell = super::IntegratedSurfaceShellConfig {
        width: if compact { 320.0 } else { 480.0 },
        height: if compact { 240.0 } else { 440.0 },
        corner_radius: 12.0,
        body_padding: 0.0,
        footer_height: crate::window_resize::mini_layout::HINT_STRIP_HEIGHT,
    };

    super::IntegratedSurfaceShell::new(shell, render_main_menu_body(live_spec, compact))
        .footer(render_main_menu_footer(live_spec))
        .into_any_element()
}

fn render_main_menu_body(live_spec: MainMenuLiveSpec, compact: bool) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let border = theme.colors.ui.border.to_rgb();
    let content = if live_spec.force_empty_results {
        render_main_menu_empty_state(live_spec)
    } else {
        render_main_menu_rows(live_spec, compact)
    };

    gpui::div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .bg(theme.colors.background.main.to_rgb())
        .child(render_main_menu_header(live_spec, compact))
        .child(gpui::div().mx(gpui::px(16.0)).h(gpui::px(1.0)).bg(border))
        .child(
            gpui::div()
                .flex_1()
                .min_h(gpui::px(0.0))
                .w_full()
                .overflow_hidden()
                .child(content),
        )
        .into_any_element()
}

fn render_main_menu_header(live_spec: MainMenuLiveSpec, compact: bool) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let chrome = crate::theme::AppChromeColors::from_theme(&theme);
    let search_text = live_spec.filter_text_override.unwrap_or("");
    let show_placeholder = search_text.is_empty();
    let input_text = if show_placeholder {
        crate::panel::DEFAULT_PLACEHOLDER
    } else {
        search_text
    };
    let input_color = if show_placeholder {
        theme.colors.text.dimmed.to_rgb()
    } else {
        theme.colors.text.primary.to_rgb()
    };

    let mut header = gpui::div()
        .w_full()
        .px(gpui::px(
            crate::window_resize::mini_layout::HEADER_PADDING_X,
        ))
        .py(gpui::px(
            crate::window_resize::mini_layout::HEADER_PADDING_Y,
        ))
        .flex()
        .flex_row()
        .items_center()
        .gap(gpui::px(crate::panel::HEADER_GAP))
        .child(
            gpui::div()
                .flex_1()
                .min_w(gpui::px(0.0))
                .flex()
                .items_center()
                .child(
                    gpui::div()
                        .w_full()
                        .text_size(gpui::px(if compact { 18.0 } else { 20.0 }))
                        .text_color(input_color)
                        .child(input_text),
                ),
        );

    if !compact {
        header = header.child(crate::components::render_launcher_ask_ai_hint(chrome));
    }

    header.into_any_element()
}

fn render_main_menu_rows(live_spec: MainMenuLiveSpec, compact: bool) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let colors = crate::components::UnifiedListItemColors::from_theme(&theme);
    let entries = main_menu_preview_entries();
    let max_entries = if compact { 5 } else { usize::MAX };
    let mut real_row_index = 0usize;

    gpui::div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .overflow_y_scrollbar()
        .children(entries.iter().take(max_entries).map(|entry| {
            match entry {
                MainMenuPreviewEntry::Section { label, count } => {
                    crate::components::SectionHeader::new(*label)
                        .count(*count)
                        .colors(colors)
                        .into_any_element()
                }
                MainMenuPreviewEntry::Row(row) => {
                    let is_selected = live_spec.prefer_first_result_selected && real_row_index == 0;
                    real_row_index += 1;

                    crate::components::UnifiedListItem::new(
                        gpui::ElementId::Name(format!("main-menu-row-{real_row_index}").into()),
                        crate::components::TextContent::plain(row.title.clone()),
                    )
                    .subtitle(crate::components::TextContent::plain(row.subtitle.clone()))
                    .leading(crate::components::LeadingContent::Icon {
                        name: row.leading_icon.clone().into(),
                        color: None,
                    })
                    .trailing(crate::components::TrailingContent::Hint(
                        row.trailing_hint.clone().into(),
                    ))
                    .density(crate::components::Density::Comfortable)
                    .colors(colors)
                    .state(crate::components::ItemState {
                        is_selected,
                        is_hovered: false,
                        is_disabled: false,
                    })
                    .with_accent_bar(true)
                    .into_any_element()
                }
            }
        }))
        .into_any_element()
}

fn render_main_menu_empty_state(live_spec: MainMenuLiveSpec) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let empty_text = live_spec.filter_text_override.unwrap_or("storybook-empty");
    let icon = crate::designs::icon_variations::IconName::MagnifyingGlass;

    gpui::div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(gpui::px(10.0))
        .child(
            gpui::svg()
                .external_path(icon.external_path())
                .size(gpui::px(28.0))
                .text_color(theme.colors.text.dimmed.with_opacity(0.55).to_rgb()),
        )
        .child(
            gpui::div()
                .text_color(theme.colors.text.primary.to_rgb())
                .text_size(gpui::px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(format!("No results for \"{empty_text}\"")),
        )
        .child(
            gpui::div()
                .text_xs()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("Try a different search term or press ⌘↵ to ask AI"),
        )
        .into_any_element()
}

fn render_main_menu_footer(live_spec: MainMenuLiveSpec) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let chrome = crate::theme::AppChromeColors::from_theme(&theme);
    let primary_label = if live_spec.force_empty_results {
        "Run"
    } else {
        main_menu_preview_entries()
            .iter()
            .find_map(|entry| match entry {
                MainMenuPreviewEntry::Row(row) => Some(row.primary_action_label.as_str()),
                MainMenuPreviewEntry::Section { .. } => None,
            })
            .unwrap_or("Run")
    };

    gpui::div()
        .w_full()
        .h_full()
        .border_t_1()
        .border_color(theme.colors.ui.border.with_opacity(0.30))
        .bg(gpui::rgba(chrome.window_surface_rgba))
        .child(
            crate::components::render_universal_prompt_hint_strip_clickable_with_primary_label(
                primary_label,
                |_event, _window, _cx| {},
                |_event, _window, _cx| {},
                |_event, _window, _cx| {},
            ),
        )
        .into_any_element()
}

fn main_menu_preview_entries() -> &'static [MainMenuPreviewEntry] {
    static ENTRIES: OnceLock<Vec<MainMenuPreviewEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(build_main_menu_preview_entries)
        .as_slice()
}

fn build_main_menu_preview_entries() -> Vec<MainMenuPreviewEntry> {
    let mut entries = Vec::new();
    let mut seen_titles = HashSet::new();

    let suggested_rows = vec![
        MainMenuPreviewRow {
            title: "Theme Designer".to_string(),
            subtitle: "Design your color theme with live preview".to_string(),
            leading_icon: "Palette".to_string(),
            trailing_hint: "Built-in".to_string(),
            primary_action_label: "Open Theme Designer".to_string(),
        },
        MainMenuPreviewRow {
            title: "Quit Script Kit".to_string(),
            subtitle: "Quit the Script Kit application".to_string(),
            leading_icon: "Command".to_string(),
            trailing_hint: "Built-in".to_string(),
            primary_action_label: "Quit Script Kit".to_string(),
        },
        MainMenuPreviewRow {
            title: "Reset Window Positions".to_string(),
            subtitle: "Reset all Script Kit windows to their default positions".to_string(),
            leading_icon: "Command".to_string(),
            trailing_hint: "Built-in".to_string(),
            primary_action_label: "Reset Window Positions".to_string(),
        },
        MainMenuPreviewRow {
            title: "Open Notes".to_string(),
            subtitle: "Open the notes window".to_string(),
            leading_icon: "Command".to_string(),
            trailing_hint: "Built-in".to_string(),
            primary_action_label: "Open Notes".to_string(),
        },
        MainMenuPreviewRow {
            title: "Hello World".to_string(),
            subtitle: "Basic starter script".to_string(),
            leading_icon: "Code".to_string(),
            trailing_hint: "Script".to_string(),
            primary_action_label: "Run Hello World".to_string(),
        },
        MainMenuPreviewRow {
            title: "Manage Downloads".to_string(),
            subtitle: "Browse and manage your downloads folder".to_string(),
            leading_icon: "Code".to_string(),
            trailing_hint: "Script".to_string(),
            primary_action_label: "Run Manage Downloads".to_string(),
        },
        MainMenuPreviewRow {
            title: "Reverse Selected Text".to_string(),
            subtitle: "Read the clipboard, transform text, and copy the result".to_string(),
            leading_icon: "Code".to_string(),
            trailing_hint: "Script".to_string(),
            primary_action_label: "Run Reverse Selected Text".to_string(),
        },
    ];
    seen_titles.extend(suggested_rows.iter().map(|row| row.title.clone()));
    push_preview_section(&mut entries, "Suggested", suggested_rows);

    let script_rows: Vec<_> = crate::scripts::read_scripts()
        .into_iter()
        .filter(|script| seen_titles.insert(script.name.clone()))
        .take(8)
        .map(|script| MainMenuPreviewRow {
            title: script.name.clone(),
            subtitle: script
                .description
                .clone()
                .or_else(|| {
                    script
                        .plugin_title
                        .clone()
                        .map(|title| format!("Plugin: {title}"))
                })
                .unwrap_or_else(|| "Script".to_string()),
            leading_icon: script.icon.clone().unwrap_or_else(|| "Code".to_string()),
            trailing_hint: script
                .shortcut
                .clone()
                .unwrap_or_else(|| "Script".to_string()),
            primary_action_label: format!("Run {}", script.name),
        })
        .collect();
    push_preview_section(&mut entries, "Scripts", script_rows);

    let skill_rows: Vec<_> = crate::plugins::discover_plugins()
        .ok()
        .and_then(|index| crate::plugins::discover_plugin_skills(&index).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|skill| seen_titles.insert(skill.title.clone()))
        .take(8)
        .map(|skill| MainMenuPreviewRow {
            title: skill.title.clone(),
            subtitle: if skill.description.is_empty() {
                format!("Plugin skill from {}", skill.plugin_title)
            } else {
                format!("{} · {}", skill.plugin_title, skill.description)
            },
            leading_icon: "Command".to_string(),
            trailing_hint: skill.plugin_title.clone(),
            primary_action_label: format!("Open {}", skill.title),
        })
        .collect();
    push_preview_section(&mut entries, "Skills", skill_rows);

    let scriptlet_rows: Vec<_> = crate::scripts::load_scriptlets()
        .into_iter()
        .filter(|scriptlet| seen_titles.insert(scriptlet.name.clone()))
        .take(8)
        .map(|scriptlet| MainMenuPreviewRow {
            title: scriptlet.name.clone(),
            subtitle: scriptlet
                .description
                .clone()
                .or_else(|| {
                    scriptlet
                        .group
                        .clone()
                        .map(|group| format!("Group: {group}"))
                })
                .unwrap_or_else(|| scriptlet.tool_display_name().to_string()),
            leading_icon: "Code".to_string(),
            trailing_hint: scriptlet.tool_display_name().to_string(),
            primary_action_label: format!("Run {}", scriptlet.name),
        })
        .collect();
    push_preview_section(&mut entries, "Scriptlets", scriptlet_rows);

    let config = crate::config::load_config();
    let builtin_rows: Vec<_> = crate::builtins::get_builtin_entries(&config.get_builtins())
        .into_iter()
        .filter(|entry| seen_titles.insert(entry.name.clone()))
        .take(8)
        .map(|entry| {
            let title = entry.name.clone();
            MainMenuPreviewRow {
                title,
                subtitle: entry.description,
                leading_icon: entry.icon.unwrap_or_else(|| "Command".to_string()),
                trailing_hint: "Built-in".to_string(),
                primary_action_label: format!("Open {}", entry.name),
            }
        })
        .collect();
    push_preview_section(&mut entries, "Built-ins", builtin_rows);

    entries
}

fn push_preview_section(
    entries: &mut Vec<MainMenuPreviewEntry>,
    label: &'static str,
    rows: Vec<MainMenuPreviewRow>,
) {
    if rows.is_empty() {
        return;
    }

    entries.push(MainMenuPreviewEntry::Section {
        label,
        count: rows.len(),
    });
    entries.extend(rows.into_iter().map(MainMenuPreviewEntry::Row));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variation_ids_have_stable_roundtrip() {
        for id in MainMenuVariationId::ALL {
            let parsed = MainMenuVariationId::from_stable_id(id.as_str());
            assert_eq!(parsed, Some(id), "roundtrip failed for {:?}", id);
        }
    }

    #[test]
    fn specs_match_variation_count() {
        assert_eq!(SPECS.len(), MainMenuVariationId::ALL.len());
    }

    #[test]
    fn story_variants_generated_for_all_specs() {
        let variants = main_menu_story_variants();
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0].stable_id(), "current-main-menu");
        assert_eq!(variants[1].stable_id(), "empty-state");
        assert_eq!(variants[2].stable_id(), "selected-result");
    }

    #[test]
    fn resolve_unknown_variant_falls_back_to_current() {
        let (live, resolution) = resolve_main_menu_variant(Some("nonexistent"));
        // Default (current-main-menu) has no overrides
        assert!(!live.force_empty_results);
        assert!(!live.prefer_first_result_selected);
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_returns_current() {
        let (live, resolution) = resolve_main_menu_variant(None);
        assert!(!live.force_empty_results);
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_empty_state_returns_force_empty() {
        let (live, resolution) = resolve_main_menu_variant(Some("empty-state"));
        assert!(live.force_empty_results);
        assert!(!live.prefer_first_result_selected);
        assert_eq!(live.filter_text_override, Some("storybook-empty"));
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn resolve_selected_result_returns_prefer_first() {
        let (live, resolution) = resolve_main_menu_variant(Some("selected-result"));
        assert!(!live.force_empty_results);
        assert!(live.prefer_first_result_selected);
        assert_eq!(live.filter_text_override, None);
        assert!(!resolution.fallback_used);
    }

    #[test]
    fn adoptable_surface_story_id_matches() {
        assert_eq!(MainMenuSurface::STORY_ID, "main-menu");
    }

    #[test]
    fn specs_have_correct_live_values() {
        for spec in &SPECS {
            let live = MainMenuSurface::live_from_spec(spec);
            assert_eq!(live, spec.live, "live_from_spec mismatch for {:?}", spec.id);
        }
    }
}
