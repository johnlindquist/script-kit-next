//! SelectPrompt component stories for the storybook
//!
//! Showcases variations of the SelectPrompt component:
//! - Single select vs multi-select
//! - With icons and descriptions
//! - With groupings
//! - Different item counts

use gpui::*;

use crate::storybook::{story_container, story_divider, story_section, Story, StoryVariant};

/// Story showcasing the SelectPrompt component variations
pub struct SelectPromptStory;

impl Story for SelectPromptStory {
    fn id(&self) -> &'static str {
        "select-prompt"
    }

    fn name(&self) -> &'static str {
        "Select Prompt"
    }

    fn category(&self) -> &'static str {
        "Prompts"
    }

    fn render(&self) -> AnyElement {
        story_container()
            .child(
                story_section("Single Select")
                    .child(render_single_select_basic())
                    .child(render_single_select_selected()),
            )
            .child(story_divider())
            .child(
                story_section("Multi-Select")
                    .child(render_multi_select_none())
                    .child(render_multi_select_some())
                    .child(render_multi_select_all()),
            )
            .child(story_divider())
            .child(
                story_section("With Icons")
                    .child(render_with_icons_unselected())
                    .child(render_with_icons_selected()),
            )
            .child(story_divider())
            .child(
                story_section("With Descriptions")
                    .child(render_with_descriptions_single())
                    .child(render_with_descriptions_multi()),
            )
            .child(story_divider())
            .child(
                story_section("With Groupings")
                    .child(render_grouped_fruits())
                    .child(render_grouped_vegetables()),
            )
            .child(story_divider())
            .child(
                story_section("Different Item Counts")
                    .child(render_few_items())
                    .child(render_many_items()),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "single-select".into(),
                description: Some("Single selection mode".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "multi-select".into(),
                description: Some("Multiple selection mode".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-icons".into(),
                description: Some("Items with icons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-descriptions".into(),
                description: Some("Items with descriptions".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-groupings".into(),
                description: Some("Items organized in groups".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "different-counts".into(),
                description: Some("Various item counts".into()),
                ..Default::default()
            },
        ]
    }
}

// ============================================================================
// COLOR CONSTANTS
// ============================================================================

const BG_CONTAINER: u32 = 0x252526;
const BG_SEARCH: u32 = 0x3c3c3c;
const BG_ITEM: u32 = 0x2d2d2d;
const BG_FOCUSED: u32 = 0x37373d;
const BORDER_COLOR: u32 = 0x3c3c3c;
const TEXT_PRIMARY: u32 = 0xcccccc;
const TEXT_MUTED: u32 = 0x888888;
const TEXT_DIMMED: u32 = 0x666666;
const ACCENT_SELECTED: u32 = 0x4a90d9;
const GROUP_HEADER_BG: u32 = 0x1e1e1e;

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

/// Render the search input area
fn render_search_input(filter: &str, selected_count: usize) -> Div {
    let display_text = if filter.is_empty() {
        "Search...".to_string()
    } else {
        filter.to_string()
    };

    div()
        .w_full()
        .px_3()
        .py_2()
        .bg(rgb(BG_SEARCH))
        .border_b_1()
        .border_color(rgb(BORDER_COLOR))
        .flex()
        .flex_row()
        .gap_2()
        .items_center()
        .child(div().text_color(rgb(TEXT_MUTED)).child("🔍"))
        .child(
            div()
                .flex_1()
                .text_color(if filter.is_empty() {
                    rgb(TEXT_MUTED)
                } else {
                    rgb(TEXT_PRIMARY)
                })
                .child(display_text),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .child(format!("{} selected", selected_count)),
        )
}

/// Render a select item
fn render_select_item(
    name: &str,
    description: Option<&str>,
    icon: Option<&str>,
    is_focused: bool,
    is_selected: bool,
    is_multi: bool,
) -> Div {
    let bg = if is_focused {
        rgb(BG_FOCUSED)
    } else {
        rgb(BG_ITEM)
    };

    let checkbox = if is_multi {
        if is_selected {
            "☑"
        } else {
            "☐"
        }
    } else if is_selected {
        "●"
    } else {
        "○"
    };

    let mut row = div()
        .w_full()
        .px_3()
        .py_2()
        .bg(bg)
        .border_b_1()
        .border_color(rgb(BORDER_COLOR))
        .rounded_sm()
        .flex()
        .flex_row()
        .gap_2()
        .items_center()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(BG_FOCUSED)));

    // Checkbox/radio indicator
    row = row.child(
        div()
            .text_color(if is_selected {
                rgb(ACCENT_SELECTED)
            } else {
                rgb(TEXT_MUTED)
            })
            .child(checkbox),
    );

    // Icon if present
    if let Some(icon_char) = icon {
        row = row.child(div().text_lg().child(icon_char.to_string()));
    }

    // Name and description
    let mut content = div().flex().flex_col().flex_1();

    content = content.child(
        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(TEXT_PRIMARY))
            .child(name.to_string()),
    );

    if let Some(desc) = description {
        content = content.child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_DIMMED))
                .child(desc.to_string()),
        );
    }

    row.child(content)
}

/// Render a group header
fn render_group_header(title: &str) -> Div {
    div()
        .w_full()
        .px_3()
        .py_1()
        .bg(rgb(GROUP_HEADER_BG))
        .border_b_1()
        .border_color(rgb(BORDER_COLOR))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_MUTED))
                .child(title.to_string().to_uppercase()),
        )
}

/// Wrapper for a complete select prompt preview
fn select_prompt_container() -> Div {
    div()
        .w_full()
        .max_w(px(400.))
        .bg(rgb(BG_CONTAINER))
        .rounded_md()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(BORDER_COLOR))
        .mb_4()
}

// ============================================================================
// SINGLE SELECT VARIATIONS
// ============================================================================

fn render_single_select_basic() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 0))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Apple", None, None, true, false, false))
                .child(render_select_item(
                    "Banana", None, None, false, false, false,
                ))
                .child(render_select_item(
                    "Cherry", None, None, false, false, false,
                )),
        )
}

fn render_single_select_selected() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 1))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Apple", None, None, false, false, false))
                .child(render_select_item("Banana", None, None, true, true, false))
                .child(render_select_item(
                    "Cherry", None, None, false, false, false,
                )),
        )
}

// ============================================================================
// MULTI-SELECT VARIATIONS
// ============================================================================

fn render_multi_select_none() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 0))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Red", None, None, true, false, true))
                .child(render_select_item("Green", None, None, false, false, true))
                .child(render_select_item("Blue", None, None, false, false, true)),
        )
}

fn render_multi_select_some() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 2))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Red", None, None, false, true, true))
                .child(render_select_item("Green", None, None, true, false, true))
                .child(render_select_item("Blue", None, None, false, true, true)),
        )
}

fn render_multi_select_all() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 3))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Red", None, None, false, true, true))
                .child(render_select_item("Green", None, None, false, true, true))
                .child(render_select_item("Blue", None, None, true, true, true)),
        )
}

// ============================================================================
// WITH ICONS VARIATIONS
// ============================================================================

fn render_with_icons_unselected() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 0))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item(
                    "Documents",
                    None,
                    Some("📁"),
                    true,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Pictures",
                    None,
                    Some("🖼️"),
                    false,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Music",
                    None,
                    Some("🎵"),
                    false,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Videos",
                    None,
                    Some("🎬"),
                    false,
                    false,
                    true,
                )),
        )
}

fn render_with_icons_selected() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 2))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item(
                    "Documents",
                    None,
                    Some("📁"),
                    false,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Pictures",
                    None,
                    Some("🖼️"),
                    true,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Music",
                    None,
                    Some("🎵"),
                    false,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Videos",
                    None,
                    Some("🎬"),
                    false,
                    false,
                    true,
                )),
        )
}

// ============================================================================
// WITH DESCRIPTIONS VARIATIONS
// ============================================================================

fn render_with_descriptions_single() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 0))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item(
                    "Development",
                    Some("Build and test your code"),
                    None,
                    true,
                    false,
                    false,
                ))
                .child(render_select_item(
                    "Production",
                    Some("Deploy to live environment"),
                    None,
                    false,
                    false,
                    false,
                ))
                .child(render_select_item(
                    "Staging",
                    Some("Test in production-like environment"),
                    None,
                    false,
                    false,
                    false,
                )),
        )
}

fn render_with_descriptions_multi() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 2))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item(
                    "TypeScript",
                    Some("Typed superset of JavaScript"),
                    Some("📘"),
                    false,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Rust",
                    Some("Memory-safe systems language"),
                    Some("🦀"),
                    true,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Python",
                    Some("Versatile scripting language"),
                    Some("🐍"),
                    false,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Go",
                    Some("Fast, concurrent language"),
                    Some("🐹"),
                    false,
                    false,
                    true,
                )),
        )
}

// ============================================================================
// WITH GROUPINGS VARIATIONS
// ============================================================================

fn render_grouped_fruits() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 1))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_group_header("Citrus"))
                .child(render_select_item(
                    "Orange",
                    None,
                    Some("🍊"),
                    false,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Lemon",
                    None,
                    Some("🍋"),
                    false,
                    false,
                    true,
                ))
                .child(render_group_header("Berries"))
                .child(render_select_item(
                    "Strawberry",
                    None,
                    Some("🍓"),
                    true,
                    false,
                    true,
                ))
                .child(render_select_item(
                    "Blueberry",
                    None,
                    Some("🫐"),
                    false,
                    false,
                    true,
                )),
        )
}

fn render_grouped_vegetables() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 2))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_group_header("Leafy Greens"))
                .child(render_select_item(
                    "Spinach",
                    Some("Rich in iron"),
                    Some("🥬"),
                    false,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Kale",
                    Some("Superfood"),
                    Some("🥗"),
                    false,
                    false,
                    true,
                ))
                .child(render_group_header("Root Vegetables"))
                .child(render_select_item(
                    "Carrot",
                    Some("Good for eyes"),
                    Some("🥕"),
                    true,
                    true,
                    true,
                ))
                .child(render_select_item(
                    "Potato",
                    Some("Versatile staple"),
                    Some("🥔"),
                    false,
                    false,
                    true,
                )),
        )
}

// ============================================================================
// DIFFERENT ITEM COUNTS VARIATIONS
// ============================================================================

fn render_few_items() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 0))
        .child(
            div()
                .flex()
                .flex_col()
                .child(render_select_item("Yes", None, None, true, false, false))
                .child(render_select_item("No", None, None, false, false, false)),
        )
}

fn render_many_items() -> impl IntoElement {
    select_prompt_container()
        .child(render_search_input("", 3))
        .child(
            div()
                .flex()
                .flex_col()
                .max_h(px(200.))
                .overflow_y_hidden()
                .child(render_select_item("Item 1", None, None, false, true, true))
                .child(render_select_item("Item 2", None, None, false, false, true))
                .child(render_select_item("Item 3", None, None, true, true, true))
                .child(render_select_item("Item 4", None, None, false, false, true))
                .child(render_select_item("Item 5", None, None, false, true, true))
                .child(render_select_item("Item 6", None, None, false, false, true))
                .child(render_select_item("Item 7", None, None, false, false, true))
                .child(render_select_item("Item 8", None, None, false, false, true)),
        )
}

// Story is registered in stories/mod.rs via get_all_stories()
