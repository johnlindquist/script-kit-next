//! List Item State Variations
//!
//! Explores different visual states for list items in the main menu:
//! - Default/idle state
//! - Hover state
//! - Focused/selected state
//! - Focused + hover state
//! - Different highlight intensities

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

pub struct ListItemStateVariationsStory;

impl Story for ListItemStateVariationsStory {
    fn id(&self) -> &'static str {
        "list-item-state-variations"
    }

    fn name(&self) -> &'static str {
        "List Item States"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = ListColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Basic States (1-4)")
                    .child(variation_item(
                        "1. Default (idle)",
                        render_list_item(
                            colors,
                            ItemState::Default,
                            "Clipboard History",
                            "View and manage your clipboard history",
                            "clipboard",
                        ),
                    ))
                    .child(variation_item(
                        "2. Hover (mouse over)",
                        render_list_item(
                            colors,
                            ItemState::Hover,
                            "Window Switcher",
                            "Switch, tile, and manage open windows",
                            "windows",
                        ),
                    ))
                    .child(variation_item(
                        "3. Selected/Focused",
                        render_list_item(
                            colors,
                            ItemState::Selected,
                            "Quick Terminal",
                            "Open a terminal for running quick commands",
                            "terminal",
                        ),
                    ))
                    .child(variation_item(
                        "4. Selected + Hover",
                        render_list_item(
                            colors,
                            ItemState::SelectedHover,
                            "Scratch Pad",
                            "Quick editor for notes and code",
                            "file-text",
                        ),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Selection Highlight Intensity (5-8)")
                    .child(variation_item(
                        "5. Selection 8% opacity",
                        render_list_item_with_intensity(
                            colors,
                            0.08,
                            "ACP Chat",
                            "Chat with AI assistants",
                        ),
                    ))
                    .child(variation_item(
                        "6. Selection 12% opacity",
                        render_list_item_with_intensity(
                            colors,
                            0.12,
                            "Bluetooth Settings",
                            "Open Bluetooth settings",
                        ),
                    ))
                    .child(variation_item(
                        "7. Selection 15% opacity (current)",
                        render_list_item_with_intensity(
                            colors,
                            0.15,
                            "Clear Suggested",
                            "Clear suggested items",
                        ),
                    ))
                    .child(variation_item(
                        "8. Selection 20% opacity",
                        render_list_item_with_intensity(
                            colors,
                            0.20,
                            "Design Gallery",
                            "Browse design variations",
                        ),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Hover Highlight Intensity (9-12) - hover to see")
                    .child(variation_item(
                        "9. Hover 5% opacity",
                        render_hover_item(colors, 0.05, "Script One", "A sample script"),
                    ))
                    .child(variation_item(
                        "10. Hover 8% opacity (current)",
                        render_hover_item(colors, 0.08, "Script Two", "Another sample script"),
                    ))
                    .child(variation_item(
                        "11. Hover 10% opacity",
                        render_hover_item(colors, 0.10, "Script Three", "Yet another script"),
                    ))
                    .child(variation_item(
                        "12. Hover 12% opacity",
                        render_hover_item(colors, 0.12, "Script Four", "One more script"),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Selection Border Styles (13-16)")
                    .child(variation_item(
                        "13. No border (bg only)",
                        render_selection_style(
                            colors,
                            SelectionStyle::BackgroundOnly,
                            "No Border Item",
                        ),
                    ))
                    .child(variation_item(
                        "14. Left accent border",
                        render_selection_style(
                            colors,
                            SelectionStyle::LeftBorder,
                            "Left Border Item",
                        ),
                    ))
                    .child(variation_item(
                        "15. Full border (subtle)",
                        render_selection_style(
                            colors,
                            SelectionStyle::FullBorder,
                            "Full Border Item",
                        ),
                    ))
                    .child(variation_item(
                        "16. Left border + bg",
                        render_selection_style(
                            colors,
                            SelectionStyle::LeftBorderWithBg,
                            "Combined Style Item",
                        ),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "default".into(),
                description: Some("Default state".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover".into(),
                description: Some("Hover state".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "selected".into(),
                description: Some("Selected state".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "selected-hover".into(),
                description: Some("Selected + hover".into()),
                ..Default::default()
            },
        ]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct ListColors {
    background: u32,
    text_primary: u32,
    text_secondary: u32,
    accent: u32,
    hover_bg: u32,
    selected_bg: u32,
    border: u32,
}

impl ListColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.muted,
            accent: theme.colors.accent.selected,
            // Use subtle selection color for hover (or derive from background)
            hover_bg: theme.colors.accent.selected_subtle,
            selected_bg: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }
}

#[derive(Clone, Copy)]
enum ItemState {
    Default,
    Hover,
    Selected,
    SelectedHover,
}

#[derive(Clone, Copy)]
enum SelectionStyle {
    BackgroundOnly,
    LeftBorder,
    FullBorder,
    LeftBorderWithBg,
}

// =============================================================================
// HELPERS
// =============================================================================

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

fn icon_placeholder(colors: ListColors) -> impl IntoElement {
    div()
        .w(px(20.))
        .h(px(20.))
        .rounded(px(4.))
        .bg(rgba((colors.accent << 8) | 0x30))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_xs()
                .text_color(colors.accent.to_rgb())
                .child("📋"),
        )
}

// =============================================================================
// LIST ITEM RENDERERS
// =============================================================================

fn render_list_item(
    colors: ListColors,
    state: ItemState,
    name: &str,
    description: &str,
    _icon: &str,
) -> impl IntoElement {
    let (bg_color, show_hover) = match state {
        ItemState::Default => (None, false),
        ItemState::Hover => (Some(rgba((colors.hover_bg << 8) | 0x14)), false), // 8%
        ItemState::Selected => (Some(rgba((colors.selected_bg << 8) | 0x26)), false), // 15%
        ItemState::SelectedHover => (Some(rgba((colors.selected_bg << 8) | 0x33)), false), // 20%
    };

    let mut item = div()
        .w_full()
        .px(px(12.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(6.))
        .cursor_pointer();

    if let Some(bg) = bg_color {
        item = item.bg(bg);
    }

    if show_hover {
        let hover_bg = (colors.hover_bg << 8) | 0x14;
        item = item.hover(move |s| s.bg(rgba(hover_bg)));
    }

    item.child(icon_placeholder(colors)).child(
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.text_primary.to_rgb())
                    .child(name.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_secondary.to_rgb())
                    .child(description.to_string()),
            ),
    )
}

fn render_list_item_with_intensity(
    colors: ListColors,
    intensity: f32,
    name: &str,
    description: &str,
) -> impl IntoElement {
    let alpha = (intensity * 255.0) as u32;
    let bg_color = rgba((colors.selected_bg << 8) | alpha);

    div()
        .w_full()
        .px(px(12.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(6.))
        .cursor_pointer()
        .bg(bg_color)
        .child(icon_placeholder(colors))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child(description.to_string()),
                ),
        )
}

fn render_hover_item(
    colors: ListColors,
    hover_intensity: f32,
    name: &str,
    description: &str,
) -> impl IntoElement {
    let hover_alpha = (hover_intensity * 255.0) as u32;
    let hover_bg = (colors.hover_bg << 8) | hover_alpha;

    div()
        .id(SharedString::from(format!("hover-item-{}", name)))
        .w_full()
        .px(px(12.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(6.))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(hover_bg)))
        .child(icon_placeholder(colors))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child(description.to_string()),
                ),
        )
}

fn render_selection_style(
    colors: ListColors,
    style: SelectionStyle,
    name: &str,
) -> impl IntoElement {
    let mut item = div()
        .w_full()
        .px(px(12.))
        .py(px(10.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(6.))
        .cursor_pointer();

    match style {
        SelectionStyle::BackgroundOnly => {
            item = item.bg(rgba((colors.selected_bg << 8) | 0x26));
        }
        SelectionStyle::LeftBorder => {
            item = item.border_l_2().border_color(colors.accent.to_rgb());
        }
        SelectionStyle::FullBorder => {
            item = item
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40));
        }
        SelectionStyle::LeftBorderWithBg => {
            item = item
                .bg(rgba((colors.selected_bg << 8) | 0x15))
                .border_l_2()
                .border_color(colors.accent.to_rgb());
        }
    }

    item.child(icon_placeholder(colors)).child(
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.text_primary.to_rgb())
                    .child(name.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_secondary.to_rgb())
                    .child("Sample description text"),
            ),
    )
}
