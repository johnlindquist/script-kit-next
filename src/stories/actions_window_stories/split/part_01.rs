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
