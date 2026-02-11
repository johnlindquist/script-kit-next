//! Footer Layout Variations - Raycast-inspired designs
//!
//! Raycast's layout:
//! - Header: Input field + "Ask AI" button (minimal, clean)
//! - List: Results label + items (icon | name | subtitle | type)
//! - Footer: Logo left, contextual action + "↵" + divider + "Actions ⌘K" right
//!
//! This moves the Run/Action buttons OUT of the header into a footer,
//! keeping the header clean and focused on input.

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};
use crate::theme::Theme;
use crate::ui_foundation::HexColorExt;

// --- merged from part_01.rs ---
pub struct FooterLayoutVariationsStory;

impl Story for FooterLayoutVariationsStory {
    fn id(&self) -> &'static str {
        "footer-layout-variations"
    }

    fn name(&self) -> &'static str {
        "Footer Layout (Raycast-style)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = LayoutColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Raycast-style Footer Layouts")
                    .child(variation_label("Header for input, footer for actions"))
                    .child(variation_item(
                        "1. Exact Raycast clone",
                        render_raycast_exact(colors),
                    ))
                    .child(variation_item(
                        "2. Script Kit branding (yellow accent)",
                        render_scriptkit_branded(colors),
                    ))
                    .child(variation_item(
                        "3. Minimal footer (just shortcuts)",
                        render_minimal_footer(colors),
                    ))
                    .child(variation_item(
                        "4. Footer with breadcrumb context",
                        render_breadcrumb_footer(colors),
                    ))
                    .child(variation_item(
                        "5. Centered action in footer",
                        render_centered_action(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Footer Action Variations")
                    .child(variation_label("Different ways to show the primary action"))
                    .child(variation_item(
                        "6. Icon + text action",
                        render_icon_text_action(colors),
                    ))
                    .child(variation_item(
                        "7. Primary button style",
                        render_primary_button_footer(colors),
                    ))
                    .child(variation_item(
                        "8. Ghost button style",
                        render_ghost_button_footer(colors),
                    ))
                    .child(variation_item(
                        "9. Split action (Run | More)",
                        render_split_action_footer(colors),
                    ))
                    .child(variation_item(
                        "10. Contextual actions row",
                        render_contextual_footer(colors),
                    )),
            )
            .into_any()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "raycast-exact".into(),
                description: Some("Exact Raycast layout".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "scriptkit-branded".into(),
                description: Some("Script Kit styling".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "minimal".into(),
                description: Some("Minimal footer".into()),
                ..Default::default()
            },
        ]
    }
}

// =============================================================================
// TYPES
// =============================================================================

#[derive(Clone, Copy)]
struct LayoutColors {
    background: u32,
    background_elevated: u32,
    background_selected: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    border: u32,
}

impl LayoutColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.background.main,
            background_elevated: theme.colors.background.title_bar,
            background_selected: theme.colors.accent.selected_subtle,
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.muted,
            text_muted: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            border: theme.colors.ui.border,
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn variation_label(text: &str) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(rgb(0x666666))
        .italic()
        .mb_2()
        .child(text.to_string())
}

fn variation_item(label: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(content)
}

/// Full window container
fn window_container(colors: LayoutColors) -> Div {
    div()
        .w_full()
        .h(px(320.))
        .flex()
        .flex_col()
        .bg(colors.background.to_rgb())
        .rounded(px(12.))
        .border_1()
        .border_color(rgba((colors.border << 8) | 0x40))
        .overflow_hidden()
}

/// Clean header with input (Raycast-style)
fn header_input(colors: LayoutColors, placeholder: &str) -> Div {
    div()
        .w_full()
        .px(px(16.))
        .py(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(
            // Input area (simulated)
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child(placeholder.to_string()),
        )
}

/// Header with Ask AI button
fn header_with_ask_ai(colors: LayoutColors, placeholder: &str) -> Div {
    let hover_bg = (colors.accent << 8) | 0x20;

    header_input(colors, placeholder).child(
        div()
            .id("ask-ai-header")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_secondary.to_rgb())
                    .child("Ask AI"),
            )
            .child(
                div()
                    .px(px(6.))
                    .py(px(2.))
                    .bg(rgba((colors.border << 8) | 0x60))
                    .rounded(px(4.))
                    .text_xs()
                    .text_color(colors.text_muted.to_rgb())
                    .child("Tab"),
            ),
    )
}

/// Results section label
fn results_label(colors: LayoutColors) -> impl IntoElement {
    div()
        .px(px(16.))
        .py(px(6.))
        .text_xs()
        .text_color(colors.text_muted.to_rgb())
        .child("Results")
}

/// List item (Raycast-style)
fn list_item(
    colors: LayoutColors,
    icon: &'static str,
    icon_bg: u32,
    name: &str,
    subtitle: &str,
    item_type: &str,
    is_selected: bool,
) -> impl IntoElement {
    let bg = if is_selected {
        Some(colors.background_selected.to_rgb())
    } else {
        None
    };

    let mut item = div()
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .rounded(px(8.))
        .mx(px(4.));

    if let Some(bg_color) = bg {
        item = item.bg(bg_color);
    }

    item
        // Icon
        .child(
            div()
                .w(px(28.))
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .bg(icon_bg.to_rgb())
                .rounded(px(6.))
                .child(div().text_sm().text_color(rgb(0xFFFFFF)).child(icon)),
        )
        // Name + subtitle
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_primary.to_rgb())
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_secondary.to_rgb())
                        .child(subtitle.to_string()),
                ),
        )
        // Type badge
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child(item_type.to_string()),
        )
}

/// Sample list content
fn sample_list(colors: LayoutColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(results_label(colors))
        .child(list_item(
            colors,
            "⚙",
            0x5856D6,
            "System Settings",
            "",
            "Application",
            true,
        ))
        .child(list_item(
            colors,
            "W",
            0x5856D6,
            "Rewrite Selected Text",
            "AI Writing Assistant",
            "Command",
            false,
        ))
        .child(list_item(
            colors,
            "⬆",
            0x5856D6,
            "Export Settings & Data",
            "Raycast",
            "Command",
            false,
        ))
        .child(list_item(
            colors,
            "●",
            0x007AFF,
            "Top Center Sixth",
            "Window Management",
            "Command",
            false,
        ))
}

// =============================================================================
// VARIATION 1: Exact Raycast Clone
// =============================================================================

fn render_raycast_exact(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_raycast_exact(colors))
}

// --- merged from part_02.rs ---

fn footer_raycast_exact(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Logo/icon
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.text_muted.to_rgb())
                        .child("🖐"),
                ),
        )
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Primary action
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                // Divider
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                // Actions
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(2.))
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba((colors.border << 8) | 0x60))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘"),
                                )
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(2.))
                                        .bg(rgba((colors.border << 8) | 0x60))
                                        .rounded(px(4.))
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("K"),
                                ),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 2: Script Kit Branded
// =============================================================================

fn render_scriptkit_branded(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_scriptkit(colors, "testing"))
        .child(sample_list_scriptkit(colors))
        .child(footer_scriptkit(colors))
}

fn header_scriptkit(colors: LayoutColors, placeholder: &str) -> Div {
    let hover_bg = (colors.accent << 8) | 0x20;
    let tab_bg = (colors.border << 8) | 0x40;

    div()
        .w_full()
        .px(px(16.))
        .py(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .child(
            div()
                .flex_1()
                .text_base()
                .text_color(colors.text_primary.to_rgb())
                .child(placeholder.to_string()),
        )
        .child(
            div()
                .id("ask-ai-sk")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(8.))
                .py(px(4.))
                .rounded(px(6.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.accent.to_rgb()) // Yellow accent
                        .child("Ask AI"),
                )
                .child(
                    div()
                        .px(px(6.))
                        .py(px(2.))
                        .bg(rgba(tab_bg))
                        .rounded(px(4.))
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Tab"),
                ),
        )
}

fn sample_list_scriptkit(colors: LayoutColors) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(results_label(colors))
        .child(list_item(
            colors,
            "📋",
            0xFBBF24,
            "Clipboard History",
            "View and manage clipboard",
            "Built-in",
            true,
        ))
        .child(list_item(
            colors,
            "🔍",
            0xFBBF24,
            "Search Files",
            "Find files on your system",
            "Script",
            false,
        ))
        .child(list_item(
            colors,
            "⌨️",
            0xFBBF24,
            "Snippet Manager",
            "Quick text expansion",
            "Built-in",
            false,
        ))
        .child(list_item(
            colors,
            "🚀",
            0xFBBF24,
            "App Launcher",
            "Launch applications",
            "Built-in",
            false,
        ))
}

fn footer_scriptkit(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Script Kit logo
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0xFBBF24D9)) // Yellow with alpha
                .rounded(px(4.))
                .child(div().text_xs().text_color(rgb(0x000000)).child("SK")),
        )
        // Right: Actions with yellow accent
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb()) // Yellow
                                .child("Run Script"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent.to_rgb()) // Yellow
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 3: Minimal Footer
// =============================================================================

fn render_minimal_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_minimal(colors))
}

fn footer_minimal(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(32.))
        .px(px(16.))
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x20))
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("↵ Open  •  ⌘K Actions  •  Tab AI"),
        )
}

// =============================================================================
// VARIATION 4: Breadcrumb Footer
// =============================================================================

fn render_breadcrumb_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_breadcrumb(colors))
}

// --- merged from part_03.rs ---

fn footer_breadcrumb(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Breadcrumb path
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Main Menu"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("›"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("System Settings"),
                ),
        )
        // Right: Actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_muted.to_rgb())
                        .child("⌘K"),
                ),
        )
}

// =============================================================================
// VARIATION 5: Centered Action
// =============================================================================

fn render_centered_action(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_centered(colors))
}

fn footer_centered(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        .child(
            div()
                .id("centered-action")
                .flex()
                .flex_row()
                .items_center()
                .gap(px(16.))
                .px(px(16.))
                .py(px(6.))
                .rounded(px(8.))
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xFFFFFF10)))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.text_primary.to_rgb())
                                .child("Open Application"),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .bg(rgba((colors.border << 8) | 0x60))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("More"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 6: Icon + Text Action
// =============================================================================

fn render_icon_text_action(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_icon_text(colors))
}

fn footer_icon_text(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Item count
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("4 results"),
        )
        // Right: Icon + text actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .id("icon-action-1")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.accent.to_rgb())
                                .child("▶"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_primary.to_rgb())
                                .child("Run"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("↵"),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(16.))
                        .bg(rgba((colors.border << 8) | 0x40)),
                )
                .child(
                    div()
                        .id("icon-action-2")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_base()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("⚡"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_secondary.to_rgb())
                                .child("Actions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 7: Primary Button Footer
// =============================================================================

fn render_primary_button_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_primary_button(colors))
}

fn footer_primary_button(colors: LayoutColors) -> impl IntoElement {
    div()
        .w_full()
        .h(px(48.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Esc to close
        .child(
            div()
                .text_xs()
                .text_color(colors.text_muted.to_rgb())
                .child("Esc to close"),
        )
        // Right: Buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Secondary
                .child(
                    div()
                        .id("secondary-btn")
                        .px(px(12.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba((colors.border << 8) | 0x60))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xFFFFFF10)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Actions"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                )
                // Primary
                .child(
                    div()
                        .id("primary-btn")
                        .px(px(12.))
                        .py(px(6.))
                        .bg(colors.accent.to_rgb())
                        .rounded(px(6.))
                        .cursor_pointer()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x000000))
                                        .child("Open"),
                                )
                                .child(div().text_xs().text_color(rgba(0x00000080)).child("↵")),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 8: Ghost Button Footer
// =============================================================================

fn render_ghost_button_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_ghost_button(colors))
}

// --- merged from part_04.rs ---

fn footer_ghost_button(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Logo
        .child(
            div()
                .w(px(20.))
                .h(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0xFBBF24D9))
                .rounded(px(4.))
                .child(div().text_xs().text_color(rgb(0x000000)).child("SK")),
        )
        // Right: Ghost buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .id("ghost-action")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgba((colors.accent << 8) | 0x40))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Run Script"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ghost-more")
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Actions"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘K"),
                                ),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 9: Split Action Footer
// =============================================================================

fn render_split_action_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_split_action(colors))
}

fn footer_split_action(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = (colors.accent << 8) | 0x20;

    div()
        .w_full()
        .h(px(44.))
        .px(px(12.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Left: Selected item info
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .w(px(16.))
                        .h(px(16.))
                        .bg(rgb(0x5856D6))
                        .rounded(px(4.)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.text_secondary.to_rgb())
                        .child("System Settings"),
                ),
        )
        // Right: Split button
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .border_1()
                .border_color(rgba((colors.accent << 8) | 0x40))
                .rounded(px(6.))
                .overflow_hidden()
                .child(
                    div()
                        .id("split-main")
                        .px(px(10.))
                        .py(px(5.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Open"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .w(px(1.))
                        .h(px(20.))
                        .bg(rgba((colors.accent << 8) | 0x40)),
                )
                .child(
                    div()
                        .id("split-more")
                        .px(px(8.))
                        .py(px(5.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("▼"),
                        ),
                ),
        )
}

// =============================================================================
// VARIATION 10: Contextual Actions Row
// =============================================================================

fn render_contextual_footer(colors: LayoutColors) -> impl IntoElement {
    window_container(colors)
        .child(header_with_ask_ai(colors, "testing"))
        .child(sample_list(colors))
        .child(footer_contextual(colors))
}

fn footer_contextual(colors: LayoutColors) -> impl IntoElement {
    let hover_bg = 0xFFFFFF15;

    div()
        .w_full()
        .h(px(40.))
        .px(px(8.))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(rgba((colors.border << 8) | 0x30))
        .bg(rgba((colors.background_elevated << 8) | 0x80))
        // Multiple contextual actions
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(2.))
                .child(
                    div()
                        .id("ctx-1")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.accent.to_rgb())
                                        .child("Open"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-2")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Edit"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘E"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-3")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Copy"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘C"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("ctx-4")
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgba(hover_bg)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.text_secondary.to_rgb())
                                        .child("Delete"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors.text_muted.to_rgb())
                                        .child("⌘⌫"),
                                ),
                        ),
                ),
        )
        // More actions
        .child(
            div()
                .id("ctx-more")
                .px(px(8.))
                .py(px(4.))
                .rounded(px(4.))
                .cursor_pointer()
                .hover(move |s| s.bg(rgba(hover_bg)))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.text_muted.to_rgb())
                                .child("More"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors.text_muted.to_rgb())
                                .child("⌘K"),
                        ),
                ),
        )
}
