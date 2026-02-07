
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
