//! Main menu variation system for storybook.
//!
//! This module follows the same adoption contract as `notes_window_variations`:
//! typed `VariationId` → `AdoptableSurface` → `resolve_surface_live` → live id.

use std::{collections::HashSet, sync::OnceLock};

use gpui::{prelude::FluentBuilder as _, IntoElement as _, ParentElement as _, Styled as _};
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
    Section {
        label: &'static str,
        count: usize,
        icon: Option<&'static str>,
    },
    Row(MainMenuPreviewRow),
}

#[derive(Clone)]
struct MainMenuPreviewRow {
    title: String,
    subtitle: String,
    leading_icon: Option<String>,
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
    let typography = crate::theme::TypographyResolver::new_theme_first(
        &theme,
        crate::designs::DesignVariant::Default,
    );
    let search_text = live_spec.filter_text_override.unwrap_or("");
    let input_font_size = if compact {
        15.0
    } else {
        // Match the visual output of the bare gpui_component Input used by the
        // live main menu. Raw `div().text_size(20)` renders too large.
        16.0
    };
    let mut input_content = gpui::div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .font_family(typography.primary_font().to_string())
        .text_size(gpui::px(input_font_size))
        .text_color(theme.colors.text.primary.to_rgb())
        .child(
            crate::components::text_input::render_text_input_cursor_selection(
                crate::components::text_input::TextInputRenderConfig {
                    cursor: search_text.len(),
                    selection: None,
                    cursor_visible: true,
                    cursor_color: theme.colors.text.primary,
                    text_color: theme.colors.text.primary,
                    selection_color: theme.colors.accent.selected,
                    selection_text_color: theme.colors.text.primary,
                    ..crate::components::text_input::TextInputRenderConfig::default_for_prompt(
                        search_text,
                    )
                },
            ),
        );
    if search_text.is_empty() {
        input_content = input_content.child(
            gpui::div()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(crate::panel::DEFAULT_PLACEHOLDER),
        );
    }

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
                        .h(gpui::px(
                            crate::panel::CURSOR_HEIGHT_LG + (crate::panel::CURSOR_MARGIN_Y * 2.0),
                        ))
                        .flex()
                        .items_center()
                        .child(input_content),
                ),
        );

    if !compact {
        header = header.child(crate::components::render_launcher_ask_ai_hint(chrome));
    }

    header.into_any_element()
}

fn render_main_menu_rows(live_spec: MainMenuLiveSpec, compact: bool) -> gpui::AnyElement {
    let theme = crate::theme::get_cached_theme();
    let colors = crate::list_item::ListItemColors::from_theme(&theme);
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
                MainMenuPreviewEntry::Section { label, count, icon } => {
                    crate::list_item::render_section_header(
                        &format!("{label} · {count}"),
                        *icon,
                        colors,
                        false,
                    )
                    .into_any_element()
                }
                MainMenuPreviewEntry::Row(row) => {
                    let is_selected = live_spec.prefer_first_result_selected && real_row_index == 0;
                    real_row_index += 1;

                    let icon_kind = row
                        .leading_icon
                        .as_deref()
                        .and_then(crate::list_item::IconKind::from_icon_hint);

                    crate::list_item::ListItem::new(row.title.clone(), colors)
                        .description(row.subtitle.clone())
                        .icon_kind_opt(icon_kind)
                        .selected(is_selected)
                        .hovered(false)
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
    let footer_text = theme
        .colors
        .text
        .primary
        .with_opacity(crate::window_resize::mini_layout::HINT_TEXT_OPACITY)
        .to_rgb();
    let shortcut_colors = crate::components::hint_strip::whisper_inline_shortcut_colors(
        footer_text.into(),
        theme.colors.text.primary.to_rgb(),
        false,
    );

    gpui::div()
        .w_full()
        .h_full()
        .border_t_1()
        .border_color(gpui::rgba(chrome.divider_rgba))
        .bg(gpui::rgba(chrome.window_surface_rgba))
        .child(
            gpui::div()
                .w_full()
                .h_full()
                .px(gpui::px(
                    crate::window_resize::mini_layout::HINT_STRIP_PADDING_X,
                ))
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .gap(gpui::px(4.0))
                .children(
                    [
                        ("↵", primary_label, 96.0_f32, true),
                        ("⌘↵", "AI", 56.0_f32, false),
                        ("⌘K", "Actions", 96.0_f32, false),
                    ]
                    .into_iter()
                    .map(|(shortcut, label, width, align_end)| {
                        let shortcut_tokens =
                            crate::components::hint_strip::shortcut_tokens_from_hint(shortcut);

                        gpui::div()
                            .w(gpui::px(width))
                            .h_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_center()
                            .when(align_end, |d| d.justify_end())
                            .child(
                                gpui::div()
                                    .px(gpui::px(4.0))
                                    .py(gpui::px(2.0))
                                    .rounded(gpui::px(4.0))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(gpui::px(3.0))
                                    .child(
                                        gpui::div()
                                            .text_size(gpui::px(12.5))
                                            .text_color(footer_text)
                                            .child(label),
                                    )
                                    .child(
                                        crate::components::hint_strip::render_inline_shortcut_keys(
                                            shortcut_tokens.iter().map(|token| token.as_str()),
                                            shortcut_colors,
                                        ),
                                    ),
                            )
                    }),
                ),
        )
        .into_any_element()
}

fn preview_entry_icon(icon_hint: &str) -> Option<String> {
    crate::list_item::IconKind::from_icon_hint(icon_hint).map(|_| icon_hint.to_string())
}

fn preview_row(
    title: impl Into<String>,
    subtitle: impl Into<String>,
    leading_icon: Option<&str>,
    primary_action_label: impl Into<String>,
) -> MainMenuPreviewRow {
    MainMenuPreviewRow {
        title: title.into(),
        subtitle: subtitle.into(),
        leading_icon: leading_icon.and_then(preview_entry_icon),
        primary_action_label: primary_action_label.into(),
    }
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
        preview_row(
            "Theme Designer",
            "Design your color theme with live preview",
            Some("Palette"),
            "Open Theme Designer",
        ),
        preview_row(
            "Quit Script Kit",
            "Quit the Script Kit application",
            Some("ArrowRightFromLine"),
            "Quit Script Kit",
        ),
        preview_row(
            "Reset Window Positions",
            "Reset all Script Kit windows to their default positions",
            Some("RefreshCw"),
            "Reset Window Positions",
        ),
        preview_row(
            "Open Notes",
            "Open the notes window",
            Some("NotebookPen"),
            "Open Notes",
        ),
        preview_row(
            "Hello World",
            "Basic starter script",
            Some("Code"),
            "Run Hello World",
        ),
        preview_row(
            "Manage Downloads",
            "Browse and manage your downloads folder",
            Some("Code"),
            "Run Manage Downloads",
        ),
        preview_row(
            "Reverse Selected Text",
            "Read the clipboard, transform text, and copy the result",
            Some("Code"),
            "Run Reverse Selected Text",
        ),
    ];
    seen_titles.extend(suggested_rows.iter().map(|row| row.title.clone()));
    push_preview_section(&mut entries, "Suggested", Some("Star"), suggested_rows);

    let script_rows: Vec<_> = crate::scripts::read_scripts()
        .into_iter()
        .filter(|script| seen_titles.insert(script.name.clone()))
        .take(8)
        .map(|script| {
            preview_row(
                script.name.clone(),
                script
                    .description
                    .clone()
                    .or_else(|| {
                        script
                            .plugin_title
                            .clone()
                            .map(|title| format!("Plugin: {title}"))
                    })
                    .unwrap_or_else(|| "Script".to_string()),
                script.icon.as_deref().or(Some("Code")),
                format!("Run {}", script.name),
            )
        })
        .collect();
    push_preview_section(&mut entries, "Main", None, script_rows);

    let skill_rows: Vec<_> = crate::plugins::discover_plugins()
        .ok()
        .and_then(|index| crate::plugins::discover_plugin_skills(&index).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|skill| seen_titles.insert(skill.title.clone()))
        .take(8)
        .map(|skill| {
            preview_row(
                skill.title.clone(),
                if skill.description.is_empty() {
                    format!("Plugin skill from {}", skill.plugin_title)
                } else {
                    format!("{} · {}", skill.plugin_title, skill.description)
                },
                Some("Sparkles"),
                format!("Open {}", skill.title),
            )
        })
        .collect();
    push_preview_section(&mut entries, "Skills", None, skill_rows);

    let scriptlet_rows: Vec<_> = crate::scripts::load_scriptlets()
        .into_iter()
        .filter(|scriptlet| seen_titles.insert(scriptlet.name.clone()))
        .take(8)
        .map(|scriptlet| {
            preview_row(
                scriptlet.name.clone(),
                scriptlet
                    .description
                    .clone()
                    .or_else(|| {
                        scriptlet
                            .group
                            .clone()
                            .map(|group| format!("Group: {group}"))
                    })
                    .unwrap_or_else(|| scriptlet.tool_display_name().to_string()),
                Some("Code"),
                format!("Run {}", scriptlet.name),
            )
        })
        .collect();
    push_preview_section(&mut entries, "Scriptlets", None, scriptlet_rows);

    let config = crate::config::load_config();
    let builtin_rows: Vec<_> = crate::builtins::get_builtin_entries(&config.get_builtins())
        .into_iter()
        .filter(|entry| seen_titles.insert(entry.name.clone()))
        .take(8)
        .map(|entry| {
            preview_row(
                entry.name.clone(),
                entry.description,
                entry.icon.as_deref().or(Some("Command")),
                format!("Open {}", entry.name),
            )
        })
        .collect();
    push_preview_section(&mut entries, "Built-ins", None, builtin_rows);

    entries
}

fn push_preview_section(
    entries: &mut Vec<MainMenuPreviewEntry>,
    label: &'static str,
    icon: Option<&'static str>,
    rows: Vec<MainMenuPreviewRow>,
) {
    if rows.is_empty() {
        return;
    }

    entries.push(MainMenuPreviewEntry::Section {
        label,
        count: rows.len(),
        icon,
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
        assert!(live.prefer_first_result_selected);
        assert!(resolution.fallback_used);
    }

    #[test]
    fn resolve_none_returns_current() {
        let (live, resolution) = resolve_main_menu_variant(None);
        assert!(!live.force_empty_results);
        assert!(live.prefer_first_result_selected);
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
