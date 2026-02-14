//! List item component stories for the storybook

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{
    code_block, story_container, story_divider, story_section, Story, StoryVariant,
};

/// Story showcasing the List Item component
pub struct ListItemStory;

impl Story for ListItemStory {
    fn id(&self) -> &'static str {
        "list-item"
    }

    fn name(&self) -> &'static str {
        "List Item"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn render(&self) -> AnyElement {
        story_container()
            .child(
                story_section("Basic List Items")
                    .child(render_list_item(
                        "Script Name",
                        Some("A helpful description"),
                        false,
                    ))
                    .child(render_list_item(
                        "Another Script",
                        Some("Does something useful"),
                        false,
                    ))
                    .child(render_list_item("Third Item", None, false)),
            )
            .child(story_divider())
            .child(
                story_section("Selected State")
                    .child(render_list_item(
                        "Selected Item",
                        Some("This item is selected"),
                        true,
                    ))
                    .child(render_list_item("Normal Item", Some("Not selected"), false)),
            )
            .child(story_divider())
            .child(
                story_section("With Icons")
                    .child(render_list_item_with_icon(
                        "File",
                        Some("text file"),
                        "📄",
                        false,
                    ))
                    .child(render_list_item_with_icon(
                        "Folder",
                        Some("directory"),
                        "📁",
                        false,
                    ))
                    .child(render_list_item_with_icon(
                        "Settings",
                        Some("configuration"),
                        "⚙️",
                        true,
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Shortcuts")
                    .child(render_list_item_with_shortcut(
                        "Open",
                        Some("Open file"),
                        "⌘O",
                        false,
                    ))
                    .child(render_list_item_with_shortcut(
                        "Save",
                        Some("Save file"),
                        "⌘S",
                        false,
                    ))
                    .child(render_list_item_with_shortcut(
                        "Close",
                        Some("Close window"),
                        "⌘W",
                        true,
                    )),
            )
            .child(story_divider())
            .child(story_section("Usage").child(code_block(
                r#"
use crate::list_item::{render_script_item, ListItemColors};

let colors = ListItemColors::from_theme(&theme);
render_script_item(
    &script,
    is_selected,
    colors,
    &filter,
    &design_variant,
    cx,
)
"#,
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "default".into(),
                description: Some("Default list item".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "selected".into(),
                description: Some("Selected list item".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-icon".into(),
                description: Some("List item with icon".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "with-shortcut".into(),
                description: Some("List item with keyboard shortcut".into()),
                ..Default::default()
            },
        ]
    }
}

fn render_list_item(name: &str, description: Option<&str>, is_selected: bool) -> impl IntoElement {
    let bg_color = if is_selected {
        rgb(0x4a90d9)
    } else {
        rgb(0x2d2d2d)
    };
    let text_color = if is_selected {
        rgb(0xffffff)
    } else {
        rgb(0xcccccc)
    };
    let desc_color = if is_selected {
        rgb(0xdddddd)
    } else {
        rgb(0x888888)
    };

    let base = div()
        .flex()
        .flex_col()
        .px_3()
        .py_2()
        .bg(bg_color)
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x3d3d3d)))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .child(name.to_string()),
        );

    if let Some(desc) = description {
        base.child(
            div()
                .text_xs()
                .text_color(desc_color)
                .child(desc.to_string()),
        )
    } else {
        base
    }
}

fn render_list_item_with_icon(
    name: &str,
    description: Option<&str>,
    icon: &str,
    is_selected: bool,
) -> impl IntoElement {
    let bg_color = if is_selected {
        rgb(0x4a90d9)
    } else {
        rgb(0x2d2d2d)
    };
    let text_color = if is_selected {
        rgb(0xffffff)
    } else {
        rgb(0xcccccc)
    };
    let desc_color = if is_selected {
        rgb(0xdddddd)
    } else {
        rgb(0x888888)
    };

    let inner = if let Some(desc) = description {
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(name.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(desc_color)
                    .child(desc.to_string()),
            )
    } else {
        div().flex().flex_col().child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .child(name.to_string()),
        )
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .bg(bg_color)
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x3d3d3d)))
        .child(div().text_lg().child(icon.to_string()))
        .child(inner)
}

fn render_list_item_with_shortcut(
    name: &str,
    description: Option<&str>,
    shortcut: &str,
    is_selected: bool,
) -> impl IntoElement {
    let bg_color = if is_selected {
        rgb(0x4a90d9)
    } else {
        rgb(0x2d2d2d)
    };
    let text_color = if is_selected {
        rgb(0xffffff)
    } else {
        rgb(0xcccccc)
    };
    let desc_color = if is_selected {
        rgb(0xdddddd)
    } else {
        rgb(0x888888)
    };
    let shortcut_color = if is_selected {
        rgb(0xdddddd)
    } else {
        rgb(0x666666)
    };

    let inner = if let Some(desc) = description {
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(name.to_string()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(desc_color)
                    .child(desc.to_string()),
            )
    } else {
        div().flex().flex_col().child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(text_color)
                .child(name.to_string()),
        )
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px_3()
        .py_2()
        .bg(bg_color)
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x3d3d3d)))
        .child(inner)
        .child(
            div()
                .px_2()
                .py_1()
                .bg(rgba(0x00000033))
                .rounded_sm()
                .text_xs()
                .text_color(shortcut_color)
                .font_family(FONT_MONO)
                .child(shortcut.to_string()),
        )
}

// Story is registered in stories/mod.rs via get_all_stories()
