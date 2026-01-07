//! Actions Window Variations - Raycast-style action panel designs
//!
//! This story explores 20 variations of the Actions Window design,
//! heavily inspired by Raycast's action panel UI:
//! - Header with context title
//! - Action items with icons, labels, and keyboard shortcut keycaps
//! - Search input (typically at bottom)
//! - Footer with primary action and Actions shortcut
//!
//! Reference: Raycast's ⌘K action panel

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

pub struct ActionsWindowStory;

impl Story for ActionsWindowStory {
    fn id(&self) -> &'static str {
        "actions-window"
    }

    fn name(&self) -> &'static str {
        "Actions Window (20 variations)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = ActionColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Actions Window - Raycast Style")
                    .child(variation_item(
                        "1. Base Raycast Design",
                        actions_window_base(colors),
                    ))
                    .child(variation_item(
                        "2. Search at Top",
                        actions_window_search_top(colors),
                    ))
                    .child(variation_item(
                        "3. Compact Items (smaller height)",
                        actions_window_compact(colors),
                    ))
                    .child(variation_item(
                        "4. Without Icons",
                        actions_window_no_icons(colors),
                    ))
                    .child(variation_item(
                        "5. With Section Headers",
                        actions_window_with_sections(colors),
                    ))
                    .child(variation_item(
                        "6. Pill-style Selection",
                        actions_window_pill_selection(colors),
                    ))
                    .child(variation_item(
                        "7. Larger Keycaps",
                        actions_window_large_keycaps(colors),
                    ))
                    .child(variation_item(
                        "8. Monochrome Icons",
                        actions_window_mono_icons(colors),
                    ))
                    .child(variation_item(
                        "9. With Descriptions",
                        actions_window_with_descriptions(colors),
                    ))
                    .child(variation_item(
                        "10. Minimal Footer",
                        actions_window_minimal_footer(colors),
                    )),
            )
            .child(
                story_section("Actions Window - Extended Variations")
                    .child(variation_item(
                        "11. Dense Layout",
                        actions_window_dense(colors),
                    ))
                    .child(variation_item(
                        "12. With Action Count",
                        actions_window_with_count(colors),
                    ))
                    .child(variation_item(
                        "13. Accent Colored Selection",
                        actions_window_accent_selection(colors),
                    ))
                    .child(variation_item(
                        "14. Bordered Items",
                        actions_window_bordered(colors),
                    ))
                    .child(variation_item(
                        "15. With Dividers",
                        actions_window_with_dividers(colors),
                    ))
                    .child(variation_item(
                        "16. Keyboard-Only (no mouse hints)",
                        actions_window_keyboard_only(colors),
                    ))
                    .child(variation_item(
                        "17. Extra Wide",
                        actions_window_wide(colors),
                    ))
                    .child(variation_item(
                        "18. Floating Style",
                        actions_window_floating(colors),
                    ))
                    .child(variation_item(
                        "19. With Categories Sidebar",
                        actions_window_categories(colors),
                    ))
                    .child(variation_item(
                        "20. Full Raycast Clone",
                        actions_window_raycast_clone(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant {
            name: "base".into(),
            description: Some("Base actions window layout".into()),
            ..Default::default()
        }]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
struct ActionColors {
    background: u32,
    background_elevated: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
    selection: u32,
    keycap_bg: u32,
    keycap_border: u32,
}

impl ActionColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            background_elevated: theme.colors.background.title_bar,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.muted,
            text_muted: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
            selection: theme.colors.accent.selected,
            keycap_bg: 0x3a3a3c,
            keycap_border: 0x4a4a4c,
        }
    }
}

/// Action item data
struct ActionItem {
    icon: &'static str,
    label: &'static str,
    shortcut: &'static [&'static str],
}

/// Sample actions for demo
fn sample_actions() -> Vec<ActionItem> {
    vec![
        ActionItem {
            icon: "□",
            label: "Open Application",
            shortcut: &["↵"],
        },
        ActionItem {
            icon: "🔍",
            label: "Show in Finder",
            shortcut: &["⌘", "↵"],
        },
        ActionItem {
            icon: "ℹ",
            label: "Show Info in Finder",
            shortcut: &["⌘", "I"],
        },
        ActionItem {
            icon: "📦",
            label: "Show Package Contents",
            shortcut: &["⌥", "⌘", "I"],
        },
        ActionItem {
            icon: "⭐",
            label: "Add to Favorites",
            shortcut: &["⇧", "⌘", "F"],
        },
    ]
}

// =============================================================================
// HELPERS
// =============================================================================

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .mb_6()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Keycap component for keyboard shortcuts
fn keycap(key: &str, colors: ActionColors) -> impl IntoElement {
    div()
        .min_w(px(20.))
        .h(px(20.))
        .px(px(6.))
        .flex()
        .items_center()
        .justify_center()
        .bg(colors.keycap_bg.to_rgb())
        .border_1()
        .border_color(colors.keycap_border.to_rgb())
        .rounded(px(4.))
        .text_xs()
        .text_color(colors.text_secondary.to_rgb())
        .child(key.to_string())
}

/// Shortcut badge row
fn shortcut_badge(shortcuts: &[&str], colors: ActionColors) -> impl IntoElement {
    let mut row = div().flex().flex_row().items_center().gap(px(2.));
    for key in shortcuts {
        row = row.child(keycap(key, colors));
    }
    row
}

/// Action row item
fn action_row(
    action: &ActionItem,
    is_selected: bool,
    colors: ActionColors,
    row_height: f32,
) -> impl IntoElement {
    let selection_bg = if is_selected {
        rgba((colors.selection << 8) | 0x20)
    } else {
        rgba(0x00000000)
    };

    div()
        .w_full()
        .h(px(row_height))
        .px(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .bg(selection_bg)
        .rounded(px(6.))
        // Icon
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(colors.text_secondary.to_rgb())
                .child(action.icon),
        )
        // Label
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(colors.text_primary.to_rgb())
                .child(action.label),
        )
        // Shortcut
        .child(shortcut_badge(action.shortcut, colors))
}

/// Header with title
fn header_bar(title: &str, colors: ActionColors) -> impl IntoElement {
    div()
        .w_full()
        .px(px(12.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.text_primary.to_rgb())
                .child(title.to_string()),
        )
}

/// Search input
fn search_input(placeholder: &str, colors: ActionColors) -> impl IntoElement {
    div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .child(
            div()
                .w_full()
                .h(px(28.))
                .px(px(10.))
                .bg(rgba((colors.border << 8) | 0x30))
                .rounded(px(6.))
                .flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child(placeholder.to_string()),
                ),
        )
}

/// Footer with primary action
fn footer_bar(action_label: &str, colors: ActionColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Primary action
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child(action_label.to_string()),
                )
                .child(keycap("↵", colors)),
        )
        // Right: Actions button
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(8.))
                .py(px(4.))
                .bg(rgba((colors.border << 8) | 0x50))
                .rounded(px(6.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Actions"),
                )
                .child(keycap("⌘", colors))
                .child(keycap("K", colors)),
        )
}

// =============================================================================
// VARIATIONS
// =============================================================================

/// 1. Base Raycast Design
fn actions_window_base(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Header
        .child(header_bar("Activity Monitor", colors))
        // Actions list
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        // Search
        .child(search_input("Search for actions...", colors))
        // Footer
        .child(footer_bar("Open Application", colors))
}

/// 2. Search at Top
fn actions_window_search_top(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Header
        .child(header_bar("Activity Monitor", colors))
        // Search at top
        .child(
            div().w_full().px(px(12.)).py(px(8.)).child(
                div()
                    .w_full()
                    .h(px(32.))
                    .px(px(10.))
                    .bg(rgba((colors.border << 8) | 0x30))
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.text_muted.to_rgb())
                            .child("🔍"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.text_muted.to_rgb())
                            .child("Search for actions..."),
                    ),
            ),
        )
        // Actions list
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        // Footer
        .child(footer_bar("Open Application", colors))
}

/// 3. Compact Items
fn actions_window_compact(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(320.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(10.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(2.))
                .px(px(4.))
                .flex()
                .flex_col()
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 32.)),
                ),
        )
        .child(search_input("Search...", colors))
}

/// 4. Without Icons
fn actions_window_no_icons(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(340.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(8.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(36.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .bg(selection_bg)
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 5. With Section Headers
fn actions_window_with_sections(colors: ActionColors) -> impl IntoElement {
    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        // Section: Application
        .child(
            div().w_full().px(px(12.)).pt(px(8.)).pb(px(4.)).child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.text_muted.to_rgb())
                    .child("APPLICATION"),
            ),
        )
        .child(
            div()
                .w_full()
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(action_row(&sample_actions()[0], true, colors, 40.))
                .child(action_row(&sample_actions()[1], false, colors, 40.)),
        )
        // Section: Finder
        .child(
            div().w_full().px(px(12.)).pt(px(12.)).pb(px(4.)).child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.text_muted.to_rgb())
                    .child("FINDER"),
            ),
        )
        .child(
            div()
                .w_full()
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(action_row(&sample_actions()[2], false, colors, 40.))
                .child(action_row(&sample_actions()[3], false, colors, 40.)),
        )
        .child(search_input("Search for actions...", colors))
}

/// 6. Pill-style Selection
fn actions_window_pill_selection(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let bg: Hsla = if is_selected {
                        colors.accent.to_rgb()
                    } else {
                        rgba(0x00000000).into()
                    };
                    let text_color: Hsla = if is_selected {
                        rgb(0x000000).into()
                    } else {
                        colors.text_primary.to_rgb()
                    };
                    let keycap_text: Hsla = if is_selected {
                        rgb(0x000000).into()
                    } else {
                        colors.text_secondary.to_rgb()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .rounded(px(20.)) // Pill shape
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(2.));
                            for key in action.shortcut {
                                let keycap_bg: Hsla = if is_selected {
                                    rgba(0x00000020).into()
                                } else {
                                    colors.keycap_bg.to_rgb()
                                };
                                let keycap_border: Hsla = if is_selected {
                                    rgba(0x00000040).into()
                                } else {
                                    colors.keycap_border.to_rgb()
                                };
                                row = row.child(
                                    div()
                                        .min_w(px(20.))
                                        .h(px(20.))
                                        .px(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(keycap_bg)
                                        .border_1()
                                        .border_color(keycap_border)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(keycap_text)
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 7. Larger Keycaps
fn actions_window_large_keycaps(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(380.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(44.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(selection_bg)
                        .rounded(px(6.))
                        .child(
                            div()
                                .w(px(22.))
                                .h(px(22.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_base()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_base()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(3.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(26.))
                                        .h(px(26.))
                                        .px(px(8.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(colors.keycap_bg.to_rgb())
                                        .border_1()
                                        .border_color(colors.keycap_border.to_rgb())
                                        .rounded(px(5.))
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 8. Monochrome Icons
fn actions_window_mono_icons(colors: ActionColors) -> impl IntoElement {
    let mono_actions = [
        ActionItem {
            icon: "◯",
            label: "Open Application",
            shortcut: &["↵"],
        },
        ActionItem {
            icon: "◎",
            label: "Show in Finder",
            shortcut: &["⌘", "↵"],
        },
        ActionItem {
            icon: "◉",
            label: "Show Info in Finder",
            shortcut: &["⌘", "I"],
        },
        ActionItem {
            icon: "◍",
            label: "Show Package Contents",
            shortcut: &["⌥", "⌘", "I"],
        },
        ActionItem {
            icon: "☆",
            label: "Add to Favorites",
            shortcut: &["⇧", "⌘", "F"],
        },
    ];

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    mono_actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
}

/// 9. With Descriptions
fn actions_window_with_descriptions(colors: ActionColors) -> impl IntoElement {
    let described_actions = [
        (
            "□",
            "Open Application",
            "Launch the selected app",
            &["↵"][..],
        ),
        (
            "🔍",
            "Show in Finder",
            "Reveal in Finder window",
            &["⌘", "↵"][..],
        ),
        (
            "ℹ",
            "Show Info in Finder",
            "Open Get Info panel",
            &["⌘", "I"][..],
        ),
    ];

    div()
        .w(px(400.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(described_actions.iter().enumerate().map(
                    |(i, (icon, label, desc, shortcut))| {
                        let is_selected = i == 0;
                        let selection_bg = if is_selected {
                            rgba((colors.selection << 8) | 0x20)
                        } else {
                            rgba(0x00000000)
                        };

                        div()
                            .w_full()
                            .h(px(52.))
                            .px(px(8.))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.))
                            .bg(selection_bg)
                            .rounded(px(6.))
                            .child(
                                div()
                                    .w(px(24.))
                                    .h(px(24.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_base()
                                    .text_color(colors.text_secondary.to_rgb())
                                    .child(*icon),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors.text_primary.to_rgb())
                                            .child(*label),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors.text_muted.to_rgb())
                                            .child(*desc),
                                    ),
                            )
                            .child(shortcut_badge(shortcut, colors))
                    },
                )),
        )
        .child(search_input("Search for actions...", colors))
}

/// 10. Minimal Footer
fn actions_window_minimal_footer(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(340.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
        // Minimal footer - just shortcuts
        .child(
            div()
                .w_full()
                .h(px(32.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .gap(px(12.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div().flex().flex_row().items_center().gap(px(4.)).child(
                        div()
                            .text_xs()
                            .text_color(colors.text_muted.to_rgb())
                            .child("↵ Run"),
                    ),
                )
                .child(
                    div().flex().flex_row().items_center().gap(px(4.)).child(
                        div()
                            .text_xs()
                            .text_color(colors.text_muted.to_rgb())
                            .child("⌘K More"),
                    ),
                ),
        )
}

/// 11. Dense Layout
fn actions_window_dense(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(300.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(8.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(6.))
                .border_b_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child("Activity Monitor"),
                ),
        )
        .child(
            div()
                .w_full()
                .py(px(2.))
                .px(px(2.))
                .flex()
                .flex_col()
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(26.))
                        .px(px(6.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .bg(selection_bg)
                        .rounded(px(4.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(1.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(14.))
                                        .h(px(14.))
                                        .px(px(3.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(colors.keycap_bg.to_rgb())
                                        .rounded(px(2.))
                                        .text_color(colors.text_muted.to_rgb())
                                        .child(
                                            div()
                                                .text_color(colors.text_muted.to_rgb())
                                                .child(key.to_string()),
                                        ),
                                );
                            }
                            row
                        })
                })),
        )
        .child(
            div()
                .w_full()
                .px(px(6.))
                .py(px(4.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .w_full()
                        .h(px(22.))
                        .px(px(6.))
                        .bg(rgba((colors.border << 8) | 0x30))
                        .rounded(px(4.))
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Search..."),
                        ),
                ),
        )
}

/// 12. With Action Count
fn actions_window_with_count(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .px(px(12.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_primary.to_rgb())
                        .child("Activity Monitor"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(rgba((colors.accent << 8) | 0x30))
                        .rounded(px(10.))
                        .text_xs()
                        .text_color(colors.accent.to_rgb())
                        .child("5 actions"),
                ),
        )
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
}

/// 13. Accent Colored Selection
fn actions_window_accent_selection(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let (bg, text_color, border): (Hsla, Hsla, Hsla) = if is_selected {
                        (
                            colors.accent.to_rgb(),
                            rgb(0x000000).into(),
                            colors.accent.to_rgb(),
                        )
                    } else {
                        (
                            rgba(0x00000000).into(),
                            colors.text_primary.to_rgb(),
                            rgba(0x00000000).into(),
                        )
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .border_1()
                        .border_color(border)
                        .rounded(px(6.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(text_color)
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(2.));
                            for key in action.shortcut {
                                let keycap_bg: Hsla = if is_selected {
                                    rgba(0x00000030).into()
                                } else {
                                    colors.keycap_bg.to_rgb()
                                };
                                row = row.child(
                                    div()
                                        .min_w(px(20.))
                                        .h(px(20.))
                                        .px(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(keycap_bg)
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 14. Bordered Items
fn actions_window_bordered(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let border: Hsla = if is_selected {
                        colors.accent.to_rgb()
                    } else {
                        rgba((colors.border << 8) | 0x60).into()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(10.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .border_1()
                        .border_color(border)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 15. With Dividers
fn actions_window_with_dividers(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .px(px(8.))
                .py(px(4.))
                .flex()
                .flex_col()
                .children(actions.iter().enumerate().flat_map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    let row = div()
                        .w_full()
                        .h(px(40.))
                        .px(px(8.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(selection_bg)
                        .rounded(px(6.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors));

                    let mut items: Vec<Div> = vec![row];
                    if i < actions.len() - 1 {
                        items.push(
                            div()
                                .w_full()
                                .h(px(1.))
                                .mx(px(8.))
                                .my(px(2.))
                                .bg(rgba((colors.border << 8) | 0x30)),
                        );
                    }
                    items
                })),
        )
        .child(search_input("Search for actions...", colors))
}

/// 16. Keyboard-Only (no mouse hints)
fn actions_window_keyboard_only(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                ),
        )
        // No search input - keyboard navigation only
        .child(
            div()
                .w_full()
                .h(px(36.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(16.))
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("↑", colors))
                        .child(keycap("↓", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Navigate"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("↵", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Select"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(keycap("Esc", colors))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("Close"),
                        ),
                ),
        )
}

/// 17. Extra Wide
fn actions_window_wide(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(480.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(4.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let selection_bg = if is_selected {
                        rgba((colors.selection << 8) | 0x20)
                    } else {
                        rgba(0x00000000)
                    };

                    div()
                        .w_full()
                        .h(px(44.))
                        .px(px(12.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .bg(selection_bg)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(28.))
                                .h(px(28.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgba((colors.accent << 8) | 0x20))
                                .rounded(px(6.))
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_base()
                                .text_color(colors.text_primary.to_rgb())
                                .child(action.label),
                        )
                        .child(shortcut_badge(action.shortcut, colors))
                })),
        )
        .child(search_input("Search for actions...", colors))
        .child(footer_bar("Open Application", colors))
}

/// 18. Floating Style (more shadow, subtle border)
fn actions_window_floating(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(16.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x20))
        .shadow_xl()
        .overflow_hidden()
        .child(header_bar("Activity Monitor", colors))
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(
                    actions
                        .iter()
                        .enumerate()
                        .map(|(i, action)| action_row(action, i == 0, colors, 42.)),
                ),
        )
        .child(search_input("Search for actions...", colors))
        .child(footer_bar("Open Application", colors))
}

/// 19. With Categories Sidebar
fn actions_window_categories(colors: ActionColors) -> impl IntoElement {
    let actions = sample_actions();

    div()
        .w(px(480.))
        .h(px(300.))
        .flex()
        .flex_row()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
        // Left sidebar
        .child(
            div()
                .w(px(120.))
                .h_full()
                .flex()
                .flex_col()
                .border_r_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .bg(rgba((colors.background_elevated << 8) | 0x60))
                .py(px(8.))
                .px(px(6.))
                .gap(px(2.))
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .bg(rgba((colors.accent << 8) | 0x20))
                        .rounded(px(6.))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.accent.to_rgb())
                        .child("All"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Application"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Finder"),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(8.))
                        .py(px(6.))
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("Favorites"),
                ),
        )
        // Right content
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .child(header_bar("Activity Monitor", colors))
                .child(
                    div()
                        .flex_1()
                        .py(px(4.))
                        .px(px(4.))
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .children(
                            actions
                                .iter()
                                .enumerate()
                                .map(|(i, action)| action_row(action, i == 0, colors, 40.)),
                        ),
                )
                .child(search_input("Search for actions...", colors)),
        )
}

/// 20. Full Raycast Clone - exact Raycast reproduction
fn actions_window_raycast_clone(_colors: ActionColors) -> impl IntoElement {
    // Raycast exact colors (dark theme)
    let raycast_bg = 0x1e1e1e;
    let raycast_border = 0x3a3a3c;
    let raycast_text = 0xffffff;
    let raycast_text_secondary = 0xa0a0a0;
    let raycast_selection = 0x3a3a3c;

    let actions = [
        ActionItem {
            icon: "□",
            label: "Open Application",
            shortcut: &["↵"],
        },
        ActionItem {
            icon: "📁",
            label: "Show in Finder",
            shortcut: &["⌘", "↵"],
        },
        ActionItem {
            icon: "ℹ️",
            label: "Show Info in Finder",
            shortcut: &["⌘", "I"],
        },
        ActionItem {
            icon: "📦",
            label: "Show Package Contents",
            shortcut: &["⌥", "⌘", "I"],
        },
        ActionItem {
            icon: "⭐",
            label: "Add to Favorites",
            shortcut: &["⇧", "⌘", "F"],
        },
    ];

    div()
        .w(px(360.))
        .flex()
        .flex_col()
        .bg(raycast_bg.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((raycast_border << 8) | 0x80))
        .shadow_2xl()
        .overflow_hidden()
        // Header
        .child(
            div()
                .w_full()
                .px(px(14.))
                .py(px(10.))
                .border_b_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(raycast_text.to_rgb())
                        .child("Activity Monitor"),
                ),
        )
        // Actions list
        .child(
            div()
                .w_full()
                .py(px(6.))
                .px(px(6.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(actions.iter().enumerate().map(|(i, action)| {
                    let is_selected = i == 0;
                    let bg: Hsla = if is_selected {
                        raycast_selection.to_rgb()
                    } else {
                        rgba(0x00000000).into()
                    };

                    div()
                        .w_full()
                        .h(px(40.))
                        .px(px(10.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.))
                        .bg(bg)
                        .rounded(px(8.))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child(action.icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(raycast_text.to_rgb())
                                .child(action.label),
                        )
                        .child({
                            let mut row = div().flex().flex_row().items_center().gap(px(3.));
                            for key in action.shortcut {
                                row = row.child(
                                    div()
                                        .min_w(px(22.))
                                        .h(px(22.))
                                        .px(px(6.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(rgb(0x4a4a4c))
                                        .border_1()
                                        .border_color(rgb(0x5a5a5c))
                                        .rounded(px(5.))
                                        .text_xs()
                                        .text_color(raycast_text_secondary.to_rgb())
                                        .child(key.to_string()),
                                );
                            }
                            row
                        })
                })),
        )
        // Search input (Raycast style - at bottom)
        .child(
            div()
                .w_full()
                .px(px(10.))
                .py(px(8.))
                .border_t_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .child(
                    div()
                        .w_full()
                        .h(px(32.))
                        .px(px(12.))
                        .bg(rgba((raycast_border << 8) | 0x40))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgba((raycast_text_secondary << 8) | 0xA0))
                                .child("Search for actions..."),
                        ),
                ),
        )
        // Footer (Raycast style)
        .child(
            div()
                .w_full()
                .h(px(44.))
                .px(px(14.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .border_t_1()
                .border_color(rgba((raycast_border << 8) | 0x60))
                .bg(rgba((raycast_bg << 8) | 0xE0))
                // Left: Primary action
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(raycast_text.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .min_w(px(22.))
                                .h(px(22.))
                                .px(px(6.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x4a4a4c))
                                .border_1()
                                .border_color(rgb(0x5a5a5c))
                                .rounded(px(5.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("↵"),
                        ),
                )
                // Right: Actions button
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .px(px(10.))
                        .py(px(5.))
                        .bg(rgb(0x4a4a4c))
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .min_w(px(18.))
                                .h(px(18.))
                                .px(px(4.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x5a5a5c))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("⌘"),
                        )
                        .child(
                            div()
                                .min_w(px(18.))
                                .h(px(18.))
                                .px(px(4.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(rgb(0x5a5a5c))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(raycast_text_secondary.to_rgb())
                                .child("K"),
                        ),
                ),
        )
}
