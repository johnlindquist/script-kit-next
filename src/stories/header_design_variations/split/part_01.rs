pub struct HeaderDesignVariationsStory;

impl Story for HeaderDesignVariationsStory {
    fn id(&self) -> &'static str {
        "header-design-variations"
    }

    fn name(&self) -> &'static str {
        "Header Design Variations"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PromptHeaderColors::from_theme(&theme);

        story_container()
            .child(
                story_section("Layout Variations (1-5)")
                    .child(header_variation_item(
                        "1. Current Production",
                        render_variation_1(colors),
                    ))
                    .child(header_variation_item(
                        "2. Compact - No Separators",
                        render_variation_2(colors),
                    ))
                    .child(header_variation_item(
                        "3. Buttons Left",
                        render_variation_3(colors),
                    ))
                    .child(header_variation_item(
                        "4. Centered Input",
                        render_variation_4(colors),
                    ))
                    .child(header_variation_item(
                        "5. Logo Left",
                        render_variation_5(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (6-10)")
                    .child(header_variation_item(
                        "6. Two Rows",
                        render_variation_6(colors),
                    ))
                    .child(header_variation_item(
                        "7. Pill Buttons",
                        render_variation_7(colors),
                    ))
                    .child(header_variation_item(
                        "8. Minimal - Input + Enter Only",
                        render_variation_8(colors),
                    ))
                    .child(header_variation_item(
                        "9. Search Box Style",
                        render_variation_9(colors),
                    ))
                    .child(header_variation_item(
                        "10. Tab Bar Style",
                        render_variation_10(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (11-15)")
                    .child(header_variation_item(
                        "11. Floating Actions",
                        render_variation_11(colors),
                    ))
                    .child(header_variation_item(
                        "12. Breadcrumb Style",
                        render_variation_12(colors),
                    ))
                    .child(header_variation_item(
                        "13. Command Palette",
                        render_variation_13(colors),
                    ))
                    .child(header_variation_item(
                        "14. Toolbar Style",
                        render_variation_14(colors),
                    ))
                    .child(header_variation_item(
                        "15. Split Header",
                        render_variation_15(colors),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("Layout Variations (16-20)")
                    .child(header_variation_item(
                        "16. Icon Buttons",
                        render_variation_16(colors),
                    ))
                    .child(header_variation_item(
                        "17. Grouped Actions",
                        render_variation_17(colors),
                    ))
                    .child(header_variation_item(
                        "18. Spotlight Style",
                        render_variation_18(colors),
                    ))
                    .child(header_variation_item(
                        "19. Alfred Style",
                        render_variation_19(colors),
                    ))
                    .child(header_variation_item(
                        "20. Raycast Style",
                        render_variation_20(colors),
                    )),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        (1..=20)
            .map(|i| StoryVariant {
                name: format!("variation-{}", i),
                description: Some(format!("Layout variation {}", i)),
                ..Default::default()
            })
            .collect()
    }
}

/// Wrapper for each header variation
fn header_variation_item(label: &str, content: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .mb_4()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(label.to_string()),
        )
        .child(
            div()
                .w_full()
                .bg(rgb(0x252526))
                .rounded_md()
                .overflow_hidden()
                .child(content),
        )
}

// ============================================================================
// VARIATION 1: Current Production Layout
// [Input] ................ [Ask AI Tab] | [Run ↵] | [Actions ⌘K] | [Logo]
// ============================================================================
fn render_variation_1(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        // Input area
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        // Ask AI hint
        .child(render_ask_ai_hint(colors))
        // Separator
        .child(render_separator(colors))
        // Run button
        .child(render_button("Run", "↵", colors))
        // Separator
        .child(render_separator(colors))
        // Actions button
        .child(render_button("Actions", "⌘K", colors))
        // Separator
        .child(render_separator(colors))
        // Logo
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 2: Compact - No Separators
// [Input] ................ [Ask AI Tab] [Run ↵] [Actions ⌘K] [Logo]
// ============================================================================
fn render_variation_2(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_logo(colors))
}

// ============================================================================
// VARIATION 3: Buttons Left
// [Logo] [Run ↵] [Actions ⌘K] | [Input] ................ [Ask AI Tab]
// ============================================================================
fn render_variation_3(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
}

// ============================================================================
// VARIATION 4: Centered Input
// [Logo] | [Actions ⌘K] ...... [Input] ...... [Ask AI Tab] | [Run ↵]
// ============================================================================
fn render_variation_4(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(render_separator(colors))
        .child(render_button("Actions", "⌘K", colors))
        .child(
            div()
                .flex_1()
                .flex()
                .justify_center()
                .text_lg()
                .text_color(colors.text_muted.to_rgb())
                .child("Script Kit"),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_separator(colors))
        .child(render_button("Run", "↵", colors))
}

// ============================================================================
// VARIATION 5: Logo Left with Title
// [Logo] Script Kit | [Input] .......... [Ask AI Tab] [Run ↵] [Actions]
// ============================================================================
fn render_variation_5(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px_4()
        .py_2()
        .gap_3()
        .child(render_logo(colors))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.text_primary.to_rgb())
                .child("Script Kit"),
        )
        .child(render_separator(colors))
        .child(
            div()
                .flex_1()
                .px_3()
                .py_1()
                .bg(colors.search_box_bg.to_rgb())
                .rounded_md()
                .text_color(colors.text_muted.to_rgb())
                .child("Type to search..."),
        )
        .child(render_ask_ai_hint(colors))
        .child(render_button("Run", "↵", colors))
        .child(render_button("Actions", "⌘K", colors))
}

// ============================================================================
// VARIATION 6: Two Rows
// Row 1: [Logo] Script Kit .......................... [Ask AI Tab]
// Row 2: [Input] .................. [Run ↵] | [Actions ⌘K]
// ============================================================================
fn render_variation_6(colors: PromptHeaderColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .px_4()
        .py_2()
        .gap_2()
        // Row 1: Title bar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(render_logo(colors))
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.text_dimmed.to_rgb())
                        .child("SCRIPT KIT"),
                )
                .child(render_ask_ai_hint(colors)),
        )
        // Row 2: Input + buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .text_lg()
                        .text_color(colors.text_muted.to_rgb())
                        .child("Type to search..."),
                )
                .child(render_button("Run", "↵", colors))
                .child(render_separator(colors))
                .child(render_button("Actions", "⌘K", colors)),
        )
}
