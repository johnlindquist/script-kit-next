pub struct DropPromptStory;

impl Story for DropPromptStory {
    fn id(&self) -> &'static str {
        "drop-prompt"
    }

    fn name(&self) -> &'static str {
        "Drop Prompt"
    }

    fn category(&self) -> &'static str {
        "Prompts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();

        story_container()
            .child(
                story_section("Empty States")
                    .child(variation_item(
                        "1. Default Empty State",
                        render_empty_state(&theme),
                    ))
                    .child(variation_item(
                        "2. Custom Placeholder",
                        render_custom_placeholder(&theme),
                    ))
                    .child(variation_item("3. Custom Hint", render_custom_hint(&theme))),
            )
            .child(story_divider())
            .child(
                story_section("Drag Hover States")
                    .child(variation_item(
                        "4. Drag Hover Active",
                        render_drag_hover(&theme),
                    ))
                    .child(variation_item(
                        "5. Drag Hover with Custom Text",
                        render_drag_hover_custom(&theme),
                    )),
            )
            .child(story_divider())
            .child(
                story_section("With Dropped Files")
                    .child(variation_item(
                        "6. Single File Dropped",
                        render_single_file(&theme),
                    ))
                    .child(variation_item(
                        "7. Multiple Files Dropped",
                        render_multiple_files(&theme),
                    ))
                    .child(variation_item("8. Many Files", render_many_files(&theme))),
            )
            .child(story_divider())
            .child(
                story_section("Different File Types")
                    .child(variation_item("9. Image Files", render_image_files(&theme)))
                    .child(variation_item(
                        "10. Document Files",
                        render_document_files(&theme),
                    ))
                    .child(variation_item("11. Code Files", render_code_files(&theme)))
                    .child(variation_item(
                        "12. Mixed File Types",
                        render_mixed_files(&theme),
                    )),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "empty".into(),
                description: Some("Empty drop zone states".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "hover".into(),
                description: Some("Drag hover states".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "files".into(),
                description: Some("With dropped files".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "types".into(),
                description: Some("Different file types".into()),
                ..Default::default()
            },
        ]
    }
}

/// Wrapper for each variation
fn variation_item(label: &str, content: impl IntoElement) -> Div {
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
// HELPER COMPONENTS
// ============================================================================

/// Simulated dropped file for stories
struct MockFile {
    name: &'static str,
    size: u64,
    icon: &'static str,
}

/// Render a drop zone container (shared layout)
fn drop_zone_container(theme: &Theme, is_hover: bool, placeholder: &str, hint: &str) -> Div {
    let border_color = if is_hover {
        rgb(theme.colors.accent.selected)
    } else {
        rgb(theme.colors.ui.border)
    };

    let bg_color = if is_hover {
        rgb(theme.colors.accent.selected_subtle)
    } else {
        rgb(theme.colors.background.search_box)
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .p_4()
        .child(
            // Drop zone
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w_full()
                .h(px(160.))
                .bg(bg_color)
                .border_2()
                .border_color(border_color)
                .rounded(px(8.))
                .child(div().text_2xl().child("📁"))
                .child(
                    div()
                        .mt_3()
                        .text_lg()
                        .text_color(rgb(theme.colors.text.secondary))
                        .child(placeholder.to_string()),
                ),
        )
        .child(
            div()
                .mt_2()
                .text_sm()
                .text_color(rgb(theme.colors.text.muted))
                .child(hint.to_string()),
        )
}

/// Render a file list item
fn file_list_item(theme: &Theme, file: &MockFile) -> Div {
    let size_str = format_file_size(file.size);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .bg(rgb(theme.colors.background.search_box))
        .rounded(px(6.))
        .child(div().text_lg().child(file.icon.to_string()))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.colors.text.primary))
                        .child(file.name.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(size_str),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(theme.colors.text.muted))
                .child("✕"),
        )
}

/// Format file size to human readable
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// ============================================================================
// VARIATIONS 1-3: Empty States
// ============================================================================

/// V1: Default empty state
fn render_empty_state(theme: &Theme) -> impl IntoElement {
    drop_zone_container(
        theme,
        false,
        "Drop files here",
        "Drag and drop files to upload",
    )
}

/// V2: Custom placeholder text
fn render_custom_placeholder(theme: &Theme) -> impl IntoElement {
    drop_zone_container(
        theme,
        false,
        "Drop images to process",
        "Supports PNG, JPG, GIF, and WebP",
    )
}

/// V3: Custom hint text
fn render_custom_hint(theme: &Theme) -> impl IntoElement {
    drop_zone_container(
        theme,
        false,
        "Drop your script here",
        "Maximum file size: 10MB",
    )
}

// ============================================================================
// VARIATIONS 4-5: Drag Hover States
// ============================================================================

/// V4: Drag hover active
fn render_drag_hover(theme: &Theme) -> impl IntoElement {
    drop_zone_container(theme, true, "Release to drop", "1 file ready to upload")
}

/// V5: Drag hover with custom text
fn render_drag_hover_custom(theme: &Theme) -> impl IntoElement {
    drop_zone_container(
        theme,
        true,
        "Release to add images",
        "3 images ready to process",
    )
}

// ============================================================================
// VARIATIONS 6-8: With Dropped Files
// ============================================================================

/// V6: Single file dropped
fn render_single_file(theme: &Theme) -> impl IntoElement {
    let file = MockFile {
        name: "document.pdf",
        size: 2_458_624,
        icon: "📄",
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .p_4()
        .gap_3()
        .child(drop_zone_container(
            theme,
            false,
            "Drop more files",
            "Or click to browse",
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.colors.text.secondary))
                        .child("1 file selected"),
                )
                .child(file_list_item(theme, &file)),
        )
}

/// V7: Multiple files dropped
fn render_multiple_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "photo-001.jpg",
            size: 3_145_728,
            icon: "🖼️",
        },
        MockFile {
            name: "photo-002.jpg",
            size: 2_097_152,
            icon: "🖼️",
        },
        MockFile {
            name: "photo-003.png",
            size: 5_242_880,
            icon: "🖼️",
        },
    ];

    div()
        .flex()
        .flex_col()
        .w_full()
        .p_4()
        .gap_3()
        .child(drop_zone_container(
            theme,
            false,
            "Drop more files",
            "Or click to browse",
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.colors.text.secondary))
                        .child("3 files selected"),
                )
                .children(files.iter().map(|f| file_list_item(theme, f))),
        )
}
