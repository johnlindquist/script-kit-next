
/// V8: Many files
fn render_many_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "file-01.txt",
            size: 1024,
            icon: "📝",
        },
        MockFile {
            name: "file-02.txt",
            size: 2048,
            icon: "📝",
        },
        MockFile {
            name: "file-03.txt",
            size: 3072,
            icon: "📝",
        },
        MockFile {
            name: "file-04.txt",
            size: 4096,
            icon: "📝",
        },
        MockFile {
            name: "file-05.txt",
            size: 5120,
            icon: "📝",
        },
        MockFile {
            name: "file-06.txt",
            size: 6144,
            icon: "📝",
        },
    ];

    let total_size: u64 = files.iter().map(|f| f.size).sum();

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
                        .flex()
                        .flex_row()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.colors.text.secondary))
                                .child(format!("{} files selected", files.len())),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.colors.text.muted))
                                .child(format!("Total: {}", format_file_size(total_size))),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .max_h(px(200.))
                        .overflow_hidden()
                        .children(files.iter().map(|f| file_list_item(theme, f))),
                ),
        )
}

// ============================================================================
// VARIATIONS 9-12: Different File Types
// ============================================================================

/// V9: Image files
fn render_image_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "vacation.jpg",
            size: 4_194_304,
            icon: "🖼️",
        },
        MockFile {
            name: "screenshot.png",
            size: 1_048_576,
            icon: "🖼️",
        },
        MockFile {
            name: "animation.gif",
            size: 2_621_440,
            icon: "🖼️",
        },
    ];

    render_file_type_section(theme, "Images", &files)
}

/// V10: Document files
fn render_document_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "report.pdf",
            size: 5_242_880,
            icon: "📄",
        },
        MockFile {
            name: "notes.docx",
            size: 524_288,
            icon: "📝",
        },
        MockFile {
            name: "data.xlsx",
            size: 1_572_864,
            icon: "📊",
        },
    ];

    render_file_type_section(theme, "Documents", &files)
}

/// V11: Code files
fn render_code_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "app.ts",
            size: 8_192,
            icon: "📜",
        },
        MockFile {
            name: "styles.css",
            size: 4_096,
            icon: "🎨",
        },
        MockFile {
            name: "config.json",
            size: 2_048,
            icon: "⚙️",
        },
    ];

    render_file_type_section(theme, "Code", &files)
}

/// V12: Mixed file types
fn render_mixed_files(theme: &Theme) -> impl IntoElement {
    let files = [
        MockFile {
            name: "photo.jpg",
            size: 3_145_728,
            icon: "🖼️",
        },
        MockFile {
            name: "document.pdf",
            size: 2_097_152,
            icon: "📄",
        },
        MockFile {
            name: "script.ts",
            size: 16_384,
            icon: "📜",
        },
        MockFile {
            name: "data.json",
            size: 4_096,
            icon: "⚙️",
        },
        MockFile {
            name: "video.mp4",
            size: 52_428_800,
            icon: "🎬",
        },
    ];

    render_file_type_section(theme, "Mixed", &files)
}

/// Helper to render a file type section
fn render_file_type_section(theme: &Theme, type_name: &str, files: &[MockFile]) -> Div {
    let total_size: u64 = files.iter().map(|f| f.size).sum();

    div()
        .flex()
        .flex_col()
        .w_full()
        .p_4()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(theme.colors.text.primary))
                        .child(format!("{} Files", type_name)),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(theme.colors.accent.selected_subtle))
                        .rounded(px(4.))
                        .text_xs()
                        .text_color(rgb(theme.colors.accent.selected))
                        .child(format!(
                            "{} files - {}",
                            files.len(),
                            format_file_size(total_size)
                        )),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .children(files.iter().map(|f| file_list_item(theme, f))),
        )
}
