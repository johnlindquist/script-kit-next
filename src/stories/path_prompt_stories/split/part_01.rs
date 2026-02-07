pub struct PathPromptStory;

impl Story for PathPromptStory {
    fn id(&self) -> &'static str {
        "path-prompt"
    }

    fn name(&self) -> &'static str {
        "Path Prompt"
    }

    fn category(&self) -> &'static str {
        "Prompts"
    }

    fn render(&self) -> AnyElement {
        let theme = Theme::default();
        let colors = PathPromptColors::from_theme(&theme);

        story_container()
            .child(story_section("File Browser View").child(variation_item(
                "Basic file browser with mixed files and folders",
                render_file_browser(colors),
            )))
            .child(story_divider())
            .child(story_section("Folder Selection").child(variation_item(
                "Folder-only view for directory selection",
                render_folder_selection(colors),
            )))
            .child(story_divider())
            .child(story_section("With Breadcrumbs").child(variation_item(
                "Path breadcrumbs navigation",
                render_with_breadcrumbs(colors),
            )))
            .child(story_divider())
            .child(story_section("With File Icons").child(variation_item(
                "Different file type icons",
                render_with_file_icons(colors),
            )))
            .child(story_divider())
            .child(story_section("Filtered View").child(variation_item(
                "Filtering results with search text",
                render_filtered_view(colors),
            )))
            .child(story_divider())
            .child(story_section("Search State").child(variation_item(
                "Active search with matching results highlighted",
                render_search_state(colors),
            )))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant {
                name: "file-browser".into(),
                description: Some("File browser with mixed content".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "folder-selection".into(),
                description: Some("Folder-only selection mode".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "breadcrumbs".into(),
                description: Some("With breadcrumb navigation".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "file-icons".into(),
                description: Some("With file type icons".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "filtered".into(),
                description: Some("Filtered results view".into()),
                ..Default::default()
            },
            StoryVariant {
                name: "search".into(),
                description: Some("Active search state".into()),
                ..Default::default()
            },
        ]
    }
}

/// Colors extracted from theme for PathPrompt styling
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct PathPromptColors {
    background: u32,
    surface: u32,
    border: u32,
    text_primary: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    selected_bg: u32,
    hover_bg: u32,
    breadcrumb_separator: u32,
    icon_folder: u32,
    icon_file: u32,
}

impl PathPromptColors {
    fn from_theme(_theme: &Theme) -> Self {
        PathPromptColors {
            background: 0x1e1e1e,
            surface: 0x252526,
            border: 0x3c3c3c,
            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_muted: 0x888888,
            accent: 0xf5a623,
            selected_bg: 0x094771,
            hover_bg: 0x2a2d2e,
            breadcrumb_separator: 0x606060,
            icon_folder: 0xdcb67a,
            icon_file: 0x8b8b8b,
        }
    }
}

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
                .bg(rgb(0x1e1e1e))
                .rounded_md()
                .border_1()
                .border_color(rgb(0x3c3c3c))
                .overflow_hidden()
                .child(content),
        )
}

fn render_file_browser(colors: PathPromptColors) -> impl IntoElement {
    let entries = vec![
        ("Documents", true, false),
        ("Downloads", true, false),
        ("Pictures", true, false),
        ("config.json", false, false),
        ("readme.md", false, true),
        ("script.ts", false, false),
    ];
    render_path_prompt_container(colors, "~/", "", &entries, Some("6 items"))
}

fn render_folder_selection(colors: PathPromptColors) -> impl IntoElement {
    let entries = vec![
        ("Applications", true, false),
        ("Desktop", true, true),
        ("Documents", true, false),
        ("Downloads", true, false),
        ("Library", true, false),
        ("Movies", true, false),
    ];
    render_path_prompt_container(colors, "~/", "", &entries, Some("Select a folder"))
}

fn render_with_breadcrumbs(colors: PathPromptColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(render_header_with_breadcrumbs(
            colors,
            &["~", "Documents", "Projects", "script-kit"],
            "",
        ))
        .child(div().w_full().h_px().bg(rgb(colors.border)))
        .child(render_path_list(
            colors,
            &[
                ("src", true, false),
                ("tests", true, false),
                ("package.json", false, true),
                ("tsconfig.json", false, false),
                ("README.md", false, false),
            ],
        ))
        .child(render_footer(colors, "5 items"))
}

fn render_with_file_icons(colors: PathPromptColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(render_header_simple(colors, "~/Projects/app/", ""))
        .child(div().w_full().h_px().bg(rgb(colors.border)))
        .child(render_file_list_with_icons(colors))
        .child(render_footer(colors, "8 items"))
}

fn render_file_list_with_icons(colors: PathPromptColors) -> impl IntoElement {
    let items = vec![
        ("📁", "node_modules", true, false, 0xdcb67a),
        ("📁", "src", true, false, 0xdcb67a),
        ("📄", "index.ts", false, true, 0x3178c6),
        ("📄", "styles.css", false, false, 0x264de4),
        ("📄", "logo.svg", false, false, 0xffb13b),
        ("📄", "data.json", false, false, 0xfbcb38),
        ("📄", "README.md", false, false, 0xffffff),
        ("📄", "package.json", false, false, 0xfbcb38),
    ];

    div()
        .flex()
        .flex_col()
        .px_2()
        .py_1()
        .gap_px()
        .children(
            items
                .into_iter()
                .map(|(icon, name, is_dir, selected, icon_color)| {
                    let bg = if selected {
                        rgb(colors.selected_bg)
                    } else {
                        rgb(0x00000000)
                    };
                    let text_color = if selected {
                        rgb(colors.text_primary)
                    } else {
                        rgb(colors.text_secondary)
                    };

                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .py_1()
                        .bg(bg)
                        .rounded_sm()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(icon_color))
                                .child(icon.to_string()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(text_color)
                                .child(name.to_string()),
                        )
                        .when(is_dir, |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(colors.text_muted))
                                    .child("→"),
                            )
                        })
                }),
        )
}

fn render_filtered_view(colors: PathPromptColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(render_header_simple(colors, "~/Documents/", "project"))
        .child(div().w_full().h_px().bg(rgb(colors.border)))
        .child(render_path_list(
            colors,
            &[
                ("my-project", true, true),
                ("project-notes.md", false, false),
                ("project-config.json", false, false),
            ],
        ))
        .child(render_footer(colors, "3 of 24 items"))
}

fn render_search_state(colors: PathPromptColors) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(render_search_header(colors, "config"))
        .child(div().w_full().h_px().bg(rgb(colors.border)))
        .child(render_search_results(colors))
        .child(render_footer(colors, "4 matches found"))
}

fn render_search_header(colors: PathPromptColors, search_text: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_2()
        .gap_2()
        .child(div().text_sm().text_color(rgb(colors.accent)).child("🔍"))
        .child(
            div()
                .text_sm()
                .text_color(rgb(colors.text_muted))
                .child("~/"),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_primary))
                        .child(search_text.to_string()),
                )
                .child(div().w_px().h_4().bg(rgb(colors.accent)).ml_px()),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded_sm()
                .text_xs()
                .text_color(rgb(colors.text_muted))
                .bg(rgb(colors.surface))
                .child("Clear"),
        )
}
