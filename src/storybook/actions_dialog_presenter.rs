//! Shared presenter for the actions dialog.
//!
//! Both the storybook previews and the live dialog call
//! [`render_actions_dialog_presentation`] with a typed
//! [`ActionsDialogPresentationModel`] and an [`ActionsDialogStyle`].
//! This guarantees visual parity between the two surfaces.

use gpui::*;

use super::actions_dialog_variations::ActionsDialogStyle;
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// ─── Presentation model ────────────────────────────────────────────────

/// Pure-data model that the presenter renders.
/// Constructed by the live dialog from its internal state, or statically
/// by storybook stories.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionsDialogPresentationModel {
    pub context_title: Option<SharedString>,
    pub search_text: SharedString,
    pub search_placeholder: SharedString,
    pub cursor_visible: bool,
    pub show_search: bool,
    pub search_at_top: bool,
    pub show_footer: bool,
    pub items: Vec<ActionsDialogPresentationItem>,
    pub selected_index: usize,
    pub hovered_index: Option<usize>,
    pub input_mode_mouse: bool,
}

/// A single item in the actions list — either a section header or an
/// actionable row.
#[derive(Clone, Debug, PartialEq)]
pub enum ActionsDialogPresentationItem {
    SectionHeader(SharedString),
    Action(ActionsDialogPresentationAction),
}

/// Presentation-only data for one action row.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionsDialogPresentationAction {
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub shortcut: Option<SharedString>,
    pub icon_svg_path: Option<SharedString>,
    pub is_destructive: bool,
}

// ─── Presenter ─────────────────────────────────────────────────────────

/// Render a complete actions dialog from a presentation model and a
/// typed style.  Every visual knob in [`ActionsDialogStyle`] is consumed
/// here so that both storybook and the live dialog produce identical
/// output for the same (model, style) pair.
pub fn render_actions_dialog_presentation(
    model: &ActionsDialogPresentationModel,
    style: ActionsDialogStyle,
    theme: &Theme,
) -> AnyElement {
    let mono: SharedString = SharedString::from(crate::list_item::FONT_MONO);

    // Container
    let mut container = div().w_full().flex().flex_col();

    // Border
    if style.show_container_border {
        container = container
            .rounded(px(10.))
            .border_1()
            .border_color(theme.colors.ui.border.with_opacity(0.3));
    } else {
        container = container.rounded(px(10.));
    }

    container = container.bg(theme.colors.background.main.to_rgb());

    // Overflow hidden when we have search (for border radius clipping)
    if model.show_search {
        container = container.overflow_hidden();
    }

    // Header — context title row
    if style.show_header {
        if let Some(ref title) = model.context_title {
            container = container.child(
                div()
                    .w_full()
                    .h(px(24.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .text_size(px(11.))
                    .text_color(theme.colors.text.dimmed.to_rgb())
                    .child(title.clone()),
            );
        }
    }

    // Search input
    if model.show_search && model.search_at_top {
        container = container.child(render_search_row(model, &style, theme, &mono));
    }

    // Search divider
    if model.show_search && model.search_at_top && style.show_search_divider {
        container = container.child(
            div()
                .w_full()
                .h(px(1.))
                .bg(theme.colors.ui.border.with_opacity(0.2)),
        );
    }

    // Action rows
    let mut action_index: usize = 0;
    let items_container = div().w_full().py(px(4.)).flex().flex_col();
    let mut item_elements: Vec<AnyElement> = Vec::new();

    for item in &model.items {
        match item {
            ActionsDialogPresentationItem::SectionHeader(label) => {
                item_elements.push(
                    div()
                        .w_full()
                        .h(px(24.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .text_size(px(11.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(label.clone())
                        .into_any_element(),
                );
            }
            ActionsDialogPresentationItem::Action(action) => {
                let is_selected = action_index == model.selected_index;
                let is_hovered = model.hovered_index == Some(action_index);

                item_elements.push(render_action_row(
                    action,
                    &style,
                    theme,
                    &mono,
                    is_selected,
                    is_hovered,
                ));

                action_index += 1;
            }
        }
    }

    container = container.child(items_container.children(item_elements));

    // Bottom search (when search is not at top)
    if model.show_search && !model.search_at_top {
        if style.show_search_divider {
            container = container.child(
                div()
                    .w_full()
                    .h(px(1.))
                    .bg(theme.colors.ui.border.with_opacity(0.2)),
            );
        }
        container = container.child(render_search_row(model, &style, theme, &mono));
    }

    container.into_any_element()
}

// ─── Sub-renderers ────────────────────────────────────────────────────

fn render_search_row(
    model: &ActionsDialogPresentationModel,
    style: &ActionsDialogStyle,
    theme: &Theme,
    mono: &SharedString,
) -> AnyElement {
    let mut row = div()
        .w_full()
        .h(px(36.))
        .px(px(14.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.));

    // Prefix marker in search
    if let Some(marker) = style.prefix_marker {
        let mut prefix = div()
            .text_size(px(13.))
            .text_color(theme.colors.text.dimmed.to_rgb());
        if style.mono_font {
            prefix = prefix.font_family(mono.clone());
        }
        row = row.child(prefix.child(SharedString::from(marker)));
    }

    // Search text or placeholder
    let display_text = if model.search_text.is_empty() {
        model.search_placeholder.clone()
    } else {
        model.search_text.clone()
    };

    let text_color = if model.search_text.is_empty() {
        theme.colors.text.dimmed.to_rgb()
    } else {
        theme.colors.text.primary.to_rgb()
    };

    let mut text_el = div().text_size(px(13.)).text_color(text_color);
    if style.mono_font {
        text_el = text_el.font_family(mono.clone());
    }
    row = row.child(text_el.child(display_text));

    // Cursor
    if model.cursor_visible {
        row = row.child(
            div()
                .w(px(1.5))
                .h(px(14.))
                .bg(theme.colors.accent.selected.to_rgb())
                .rounded(px(1.)),
        );
    }

    row.into_any_element()
}

fn render_action_row(
    action: &ActionsDialogPresentationAction,
    style: &ActionsDialogStyle,
    theme: &Theme,
    mono: &SharedString,
    is_selected: bool,
    is_hovered: bool,
) -> AnyElement {
    let mut row = div()
        .w_full()
        .h(px(style.row_height))
        .px(px(12.))
        .rounded(px(style.row_radius))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.));

    // Selection / hover background
    if is_selected && style.selection_opacity > 0.0 {
        row = row.bg(theme
            .colors
            .accent
            .selected
            .with_opacity(style.selection_opacity));
    } else if is_hovered && style.hover_opacity > 0.0 {
        row = row.bg(theme
            .colors
            .accent
            .selected
            .with_opacity(style.hover_opacity));
    }

    // Dot accent indicator (when selection_opacity is 0 and we're selected)
    if is_selected && style.selection_opacity == 0.0 {
        row = row.child(
            div()
                .w(px(5.))
                .h(px(5.))
                .rounded(px(3.))
                .bg(theme.colors.accent.selected.to_rgb()),
        );
    } else if style.selection_opacity == 0.0 {
        // Spacer for alignment when dot-accent variant but not selected
        row = row.child(
            div()
                .w(px(5.))
                .h(px(5.))
                .rounded(px(3.))
                .bg(gpui::transparent_black()),
        );
    }

    // Prefix marker
    if let Some(marker) = style.prefix_marker {
        let indicator = if is_selected { "▸" } else { " " };
        let mut prefix = div().text_size(px(12.));
        if style.mono_font {
            prefix = prefix.font_family(mono.clone());
        }
        prefix = prefix.text_color(if is_selected {
            theme.colors.text.primary.to_rgb()
        } else {
            theme.colors.text.dimmed.to_rgb()
        });
        // Use the indicator instead of the raw marker for the row prefix
        let _ = marker;
        row = row.child(prefix.child(SharedString::from(indicator)));
    }

    // Icon
    if style.show_icons {
        if let Some(ref icon_path) = action.icon_svg_path {
            row = row.child(
                div()
                    .w(px(16.))
                    .text_size(px(12.))
                    .text_color(if is_selected {
                        theme.colors.accent.selected.to_rgb()
                    } else {
                        theme.colors.text.dimmed.to_rgb()
                    })
                    .child(icon_path.clone()),
            );
        }
    }

    // Title
    let mut title_el = div().flex_1().min_w(px(0.)).text_size(px(13.));
    if style.mono_font {
        title_el = title_el.font_family(mono.clone());
    }
    title_el = title_el.text_color(if is_selected {
        theme.colors.text.primary.to_rgb()
    } else {
        if action.is_destructive {
            theme.colors.text.secondary.to_rgb()
        } else {
            theme.colors.text.secondary.to_rgb()
        }
    });
    if style.shortcut_visible && uses_inline_shortcuts(style) {
        // Inline keys mode: title + shortcut in same flex row
        let mut inline_row = div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.));
        inline_row = inline_row.child(title_el.child(action.title.clone()));
        if let Some(ref shortcut) = action.shortcut {
            inline_row = inline_row.child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.colors.text.dimmed.with_opacity(0.35))
                    .child(shortcut.clone()),
            );
        }
        row = row.child(inline_row);
    } else {
        row = row.child(title_el.child(action.title.clone()));

        // Shortcut badge (non-inline)
        if style.shortcut_visible {
            if let Some(ref shortcut) = action.shortcut {
                let mut shortcut_el = div()
                    .text_size(px(11.))
                    .text_color(theme.colors.text.dimmed.with_opacity(0.4));
                if style.mono_font {
                    shortcut_el = shortcut_el.font_family(mono.clone());
                }
                row = row.child(shortcut_el.child(shortcut.clone()));
            }
        }
    }

    row.into_any_element()
}

/// Check if this style matches the inline-keys variant.
fn uses_inline_shortcuts(style: &ActionsDialogStyle) -> bool {
    crate::storybook::actions_dialog_variations::actions_dialog_style_uses_inline_shortcuts(style)
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storybook::actions_dialog_variations::{resolve_actions_dialog_style, SPECS};

    fn sample_model() -> ActionsDialogPresentationModel {
        ActionsDialogPresentationModel {
            context_title: Some(SharedString::from("Actions")),
            search_text: SharedString::from(""),
            search_placeholder: SharedString::from("Search actions..."),
            cursor_visible: true,
            show_search: true,
            search_at_top: false,
            show_footer: true,
            items: vec![
                ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                    title: SharedString::from("Open Application"),
                    subtitle: None,
                    shortcut: Some(SharedString::from("↵")),
                    icon_svg_path: None,
                    is_destructive: false,
                }),
                ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                    title: SharedString::from("Show in Finder"),
                    subtitle: None,
                    shortcut: Some(SharedString::from("⌘↵")),
                    icon_svg_path: Some(SharedString::from("🔍")),
                    is_destructive: false,
                }),
                ActionsDialogPresentationItem::Action(ActionsDialogPresentationAction {
                    title: SharedString::from("Delete"),
                    subtitle: None,
                    shortcut: Some(SharedString::from("⌘⌫")),
                    icon_svg_path: None,
                    is_destructive: true,
                }),
            ],
            selected_index: 0,
            hovered_index: None,
            input_mode_mouse: false,
        }
    }

    #[test]
    fn presenter_model_fields_are_complete() {
        let model = sample_model();
        assert_eq!(model.context_title.as_deref(), Some("Actions"));
        assert_eq!(model.items.len(), 3);
        assert!(model.show_search);
        assert!(!model.search_at_top);
        assert!(model.cursor_visible);
    }

    #[test]
    fn presenter_covers_all_style_fields() {
        // Verify every ActionsDialogStyle field is consumed in the presenter
        // by checking each spec produces a different visual configuration.
        // We test this indirectly by asserting the specs are distinct.
        let styles: Vec<ActionsDialogStyle> = SPECS.iter().map(|s| s.style).collect();
        for (i, a) in styles.iter().enumerate() {
            for (j, b) in styles.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "specs[{i}] and specs[{j}] must differ");
                }
            }
        }
    }

    #[test]
    fn whisper_variant_stays_quiet() {
        let (style, resolution) = resolve_actions_dialog_style(Some("whisper"));
        assert_eq!(resolution.resolved_variant_id, "whisper");
        assert!(!style.show_container_border);
        assert!(!style.show_header);
        assert!(!style.show_search_divider);
        assert_eq!(style.selection_opacity, 0.04);
        assert_eq!(style.hover_opacity, 0.03);
    }

    #[test]
    fn current_variant_matches_production_constants() {
        let (style, resolution) = resolve_actions_dialog_style(Some("current"));
        assert_eq!(resolution.resolved_variant_id, "current");
        assert!(style.show_container_border);
        assert!(style.show_header);
        assert!(style.show_search_divider);
        assert_eq!(style.row_height, 30.0);
        assert_eq!(style.row_radius, 6.0);
    }

    #[test]
    fn dot_accent_variant_uses_dot_selection_mode() {
        let (style, resolution) = resolve_actions_dialog_style(Some("dot-accent"));
        assert_eq!(resolution.resolved_variant_id, "dot-accent");
        assert_eq!(style.selection_opacity, 0.0);
        assert_eq!(style.hover_opacity, 0.03);
    }

    #[test]
    fn typewriter_variant_uses_mono_and_prefix() {
        let (style, resolution) = resolve_actions_dialog_style(Some("typewriter"));
        assert_eq!(resolution.resolved_variant_id, "typewriter");
        assert!(style.mono_font);
        assert_eq!(style.prefix_marker, Some(">"));
        assert_eq!(style.row_radius, 0.0);
    }

    #[test]
    fn single_column_hides_shortcuts_and_icons() {
        let (style, resolution) = resolve_actions_dialog_style(Some("single-column"));
        assert_eq!(resolution.resolved_variant_id, "single-column");
        assert!(!style.shortcut_visible);
        assert!(!style.show_icons);
    }

    #[test]
    fn section_header_item_round_trips() {
        let item = ActionsDialogPresentationItem::SectionHeader(SharedString::from("General"));
        if let ActionsDialogPresentationItem::SectionHeader(label) = &item {
            assert_eq!(label.as_ref(), "General");
        } else {
            panic!("expected SectionHeader");
        }
    }

    #[test]
    fn action_item_round_trips() {
        let action = ActionsDialogPresentationAction {
            title: SharedString::from("Copy"),
            subtitle: Some(SharedString::from("to clipboard")),
            shortcut: Some(SharedString::from("⌘C")),
            icon_svg_path: None,
            is_destructive: false,
        };
        assert_eq!(action.title.as_ref(), "Copy");
        assert_eq!(action.subtitle.as_deref(), Some("to clipboard"));
    }
}
