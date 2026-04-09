/// Theming context for focused-info rendering.
/// Pre-extracted from theme and design tokens to avoid borrow conflicts.
#[derive(Clone)]
struct FocusedInfoStyle {
    text_primary: u32,
    text_muted: u32,
    text_secondary: u32,
    ui_border: u32,
    accent_color: u32,
    accent_selected: u32,
    section_label_font_size: f32,
    body_text_line_height: f32,
    spacing: designs::DesignSpacing,
    typography: designs::DesignTypography,
    visual: designs::DesignVisual,
    badge_bg_rgba: u32,
    badge_text_hex: u32,
    badge_border_rgba: u32,
    accent_badge_bg_rgba: u32,
    accent_badge_text_hex: u32,
    accent_badge_border_rgba: u32,
    divider_rgba: u32,
}

impl FocusedInfoStyle {
    fn from_theme_and_design(
        theme: &crate::theme::Theme,
        design: designs::DesignVariant,
    ) -> Self {
        let tokens = get_tokens(design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();
        let is_light_mode = !theme.is_dark_mode();
        let chrome = crate::theme::AppChromeColors::from_theme(theme);

        Self {
            text_primary: if is_light_mode {
                theme.colors.text.primary
            } else {
                colors.text_primary
            },
            text_muted: if is_light_mode {
                theme.colors.text.muted
            } else {
                colors.text_muted
            },
            text_secondary: if is_light_mode {
                theme.colors.text.secondary
            } else {
                colors.text_secondary
            },
            ui_border: colors.border,
            accent_color: colors.accent,
            accent_selected: theme.colors.accent.selected,
            section_label_font_size: preview_panel_typography_section_label_size(typography),
            body_text_line_height: preview_panel_typography_body_line_height(typography),
            spacing,
            typography,
            visual,
            badge_bg_rgba: chrome.badge_bg_rgba,
            badge_text_hex: chrome.badge_text_hex,
            badge_border_rgba: chrome.badge_border_rgba,
            accent_badge_bg_rgba: chrome.accent_badge_bg_rgba,
            accent_badge_text_hex: chrome.accent_badge_text_hex,
            accent_badge_border_rgba: chrome.accent_badge_border_rgba,
            divider_rgba: chrome.divider_rgba,
        }
    }
}

/// Render metadata content for an inline calculator result.
/// Returns a container div with expression, badges, result, and word representation.
fn render_focused_info_for_calculator(
    calculator: &crate::calculator::CalculatorInlineResult,
    style: &FocusedInfoStyle,
) -> Div {
    let s = &style.spacing;
    let t = &style.typography;

    div()
        .flex()
        .flex_col()
        .child(
            div()
                .text_xs()
                .font_family(t.font_family_mono)
                .text_color(rgba((style.text_muted << 8) | 0x99))
                .pb(px(s.padding_xs))
                .child("calculator: inline"),
        )
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(style.text_primary))
                .pb(px(s.padding_sm))
                .child(calculator.normalized_expr.clone()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(s.gap_sm))
                .pb(px(s.padding_md))
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(rgba(style.badge_bg_rgba))
                        .border_1()
                        .border_color(rgba(style.badge_border_rgba))
                        .text_xs()
                        .text_color(rgb(style.badge_text_hex))
                        .child(calculator.operation_name.clone()),
                )
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(rgba(style.accent_badge_bg_rgba))
                        .border_1()
                        .border_color(rgba(style.accent_badge_border_rgba))
                        .text_xs()
                        .text_color(rgb(style.accent_badge_text_hex))
                        .child("Enter copies result"),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(style.visual.border_thin))
                .bg(rgba(style.divider_rgba))
                .my(px(s.padding_sm)),
        )
        .child(
            div()
                .text_size(px(style.section_label_font_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba((style.text_muted << 8) | 0xCC))
                .pb(px(s.padding_sm))
                .child("RESULT"),
        )
        .child(
            div()
                .text_size(px(t.font_size_xl))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(style.accent_selected))
                .pb(px(s.padding_xs))
                .child(calculator.formatted.clone()),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(style.text_secondary))
                .child(calculator.words.clone()),
        )
}

/// Render metadata content for a search result.
/// Returns a container div with type-specific info (name, badges, shortcuts, description, type indicator).
/// Does NOT include code preview — that is handled separately by the preview panel.
fn render_focused_info_for_result(
    result: &scripts::SearchResult,
    shortcut_display: &Option<String>,
    match_indices: &scripts::MatchIndices,
    style: &FocusedInfoStyle,
) -> Div {
    let s = &style.spacing;
    let t = &style.typography;

    let mut content = div().flex().flex_col();

    match result {
        scripts::SearchResult::Script(script_match) => {
            let script = &script_match.script;

            // Source indicator with match highlighting
            let filename = &script_match.filename;
            let filename_indices = &match_indices.filename_indices;
            let path_segments =
                render_path_with_highlights(filename, filename, filename_indices);

            let mut path_div = div()
                .flex()
                .flex_row()
                .text_xs()
                .font_family(t.font_family_mono)
                .pb(px(s.padding_xs))
                .overflow_x_hidden()
                .child(
                    div()
                        .text_color(rgba((style.text_muted << 8) | 0x99))
                        .child("script: "),
                );

            for (text, is_highlighted) in path_segments {
                let color = if is_highlighted {
                    rgb(style.accent_color)
                } else {
                    rgba((style.text_muted << 8) | 0x99)
                };
                path_div = path_div.child(div().text_color(color).child(text));
            }

            content = content.child(path_div);

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_md))
                    .child(format!("{}.{}", script.name, script.extension)),
            );

            // Metadata badges: kit name, language, alias, author, tags
            {
                let mut script_badges = div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(s.gap_sm))
                    .pb(px(s.padding_sm));
                let mut show_script_badges = false;
                if let Some(ref kit) = script.kit_name {
                    show_script_badges = true;
                    script_badges = script_badges.child(
                        focused_info_badge(
                            &format!("kit: {}", kit),
                            style.badge_bg_rgba,
                            style.badge_border_rgba,
                            style.badge_text_hex,
                        ),
                    );
                }
                // Extension badge
                {
                    let ext_display = match script.extension.as_str() {
                        "ts" => "TypeScript",
                        "js" => "JavaScript",
                        "mjs" => "JavaScript",
                        "sh" => "Shell",
                        "bash" => "Bash",
                        "zsh" => "Zsh",
                        "py" => "Python",
                        "rb" => "Ruby",
                        _ => "",
                    };
                    if !ext_display.is_empty() {
                        show_script_badges = true;
                        script_badges = script_badges.child(
                            focused_info_badge(
                                ext_display,
                                style.badge_bg_rgba,
                                style.badge_border_rgba,
                                style.badge_text_hex,
                            ),
                        );
                    }
                }
                if let Some(ref alias) = script.alias {
                    show_script_badges = true;
                    script_badges = script_badges.child(
                        focused_info_badge(
                            &format!("alias: {}", alias),
                            style.accent_badge_bg_rgba,
                            style.accent_badge_border_rgba,
                            style.accent_badge_text_hex,
                        ),
                    );
                }
                // Author badge from typed metadata
                if let Some(ref typed_meta) = script.typed_metadata {
                    if let Some(ref author) = typed_meta.author {
                        show_script_badges = true;
                        script_badges = script_badges.child(
                            focused_info_badge(
                                &format!("by {}", author),
                                style.badge_bg_rgba,
                                style.badge_border_rgba,
                                style.badge_text_hex,
                            ),
                        );
                    }
                    for tag in &typed_meta.tags {
                        show_script_badges = true;
                        script_badges = script_badges.child(
                            focused_info_badge(
                                tag,
                                style.badge_bg_rgba,
                                style.badge_border_rgba,
                                style.badge_text_hex,
                            ),
                        );
                    }
                }
                if show_script_badges {
                    content = content.child(script_badges);
                }
            }

            // Keyboard shortcut
            let effective_shortcut =
                script.shortcut.clone().or_else(|| shortcut_display.clone());
            if let Some(shortcut_str) = effective_shortcut {
                content = content.child(focused_info_shortcut_section(
                    &shortcut_str,
                    style,
                ));
            }

            // Description
            if let Some(desc) = &script.description {
                content = content.child(
                    div()
                        .text_sm()
                        .line_height(px(style.body_text_line_height))
                        .text_color(rgb(style.text_secondary))
                        .pb(px(s.padding_lg))
                        .child(desc.clone()),
                );
            }
        }

        scripts::SearchResult::Scriptlet(scriptlet_match) => {
            let scriptlet = &scriptlet_match.scriptlet;

            // Source indicator with match highlighting
            if let Some(ref display_file_path) = scriptlet_match.display_file_path {
                let filename_indices = &match_indices.filename_indices;
                let path_segments = render_path_with_highlights(
                    display_file_path,
                    display_file_path,
                    filename_indices,
                );

                let mut path_div = div()
                    .flex()
                    .flex_row()
                    .text_xs()
                    .font_family(t.font_family_mono)
                    .pb(px(s.padding_xs))
                    .overflow_x_hidden()
                    .child(
                        div()
                            .text_color(rgba((style.text_muted << 8) | 0x99))
                            .child("scriptlet: "),
                    );

                for (text, is_highlighted) in path_segments {
                    let color = if is_highlighted {
                        rgb(style.accent_color)
                    } else {
                        rgba((style.text_muted << 8) | 0x99)
                    };
                    path_div = path_div.child(div().text_color(color).child(text));
                }

                content = content.child(path_div);
            }

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_md))
                    .child(scriptlet.name.clone()),
            );

            // Metadata badges: tool type, group, alias, keyword
            {
                let mut slet_badges = div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(s.gap_sm))
                    .pb(px(s.padding_sm));
                slet_badges = slet_badges.child(
                    focused_info_badge(
                        scriptlet.tool_display_name(),
                        style.badge_bg_rgba,
                        style.badge_border_rgba,
                        style.badge_text_hex,
                    ),
                );
                if let Some(ref group) = scriptlet.group {
                    if !group.is_empty() {
                        slet_badges = slet_badges.child(
                            focused_info_badge(
                                group,
                                style.badge_bg_rgba,
                                style.badge_border_rgba,
                                style.badge_text_hex,
                            ),
                        );
                    }
                }
                if let Some(ref alias) = scriptlet.alias {
                    slet_badges = slet_badges.child(
                        focused_info_badge(
                            &format!("alias: {}", alias),
                            style.accent_badge_bg_rgba,
                            style.accent_badge_border_rgba,
                            style.accent_badge_text_hex,
                        ),
                    );
                }
                if let Some(ref keyword) = scriptlet.keyword {
                    slet_badges = slet_badges.child(
                        focused_info_badge(
                            &format!("keyword: {}", keyword),
                            style.accent_badge_bg_rgba,
                            style.accent_badge_border_rgba,
                            style.accent_badge_text_hex,
                        ),
                    );
                }
                content = content.child(slet_badges);
            }

            // Description
            if let Some(desc) = &scriptlet.description {
                content = content.child(
                    div()
                        .text_sm()
                        .line_height(px(style.body_text_line_height))
                        .text_color(rgb(style.text_secondary))
                        .pb(px(s.padding_lg))
                        .child(desc.clone()),
                );
            }

            // Shortcut
            let effective_shortcut = scriptlet
                .shortcut
                .clone()
                .or_else(|| shortcut_display.clone());
            if let Some(shortcut) = effective_shortcut {
                content = content.child(focused_info_shortcut_section(
                    &shortcut,
                    style,
                ));
            }
        }

        scripts::SearchResult::BuiltIn(builtin_match) => {
            let builtin = &builtin_match.entry;

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_md))
                    .child(builtin.name.clone()),
            );

            // Keyboard shortcut
            if let Some(ref shortcut_str) = shortcut_display {
                content = content.child(focused_info_shortcut_section(
                    shortcut_str,
                    style,
                ));
            }

            // Description
            content = content.child(
                div()
                    .text_sm()
                    .line_height(px(style.body_text_line_height))
                    .text_color(rgb(style.text_secondary))
                    .pb(px(s.padding_lg))
                    .child(builtin.description.clone()),
            );

            // Keywords and feature type as subtle inline tags
            let mut metadata_tags = preview_keyword_tags(&builtin.keywords);
            let feature_tag =
                builtin_feature_annotation(&builtin.feature).to_lowercase();
            if !metadata_tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(&feature_tag))
            {
                metadata_tags.push(feature_tag);
            }

            if !metadata_tags.is_empty() {
                let mut tags_row = div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(s.gap_sm))
                    .pb(px(s.padding_md));

                for tag in metadata_tags {
                    tags_row = tags_row.child(
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(999.0))
                            .bg(rgba(style.badge_bg_rgba))
                            .border_1()
                            .border_color(rgba(style.badge_border_rgba))
                            .text_xs()
                            .text_color(rgb(style.badge_text_hex))
                            .child(tag),
                    );
                }

                content = content.child(tags_row);
            }
        }

        scripts::SearchResult::App(app_match) => {
            let app = &app_match.app;

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_md))
                    .child(app.name.clone()),
            );

            // Keyboard shortcut
            if let Some(ref shortcut_str) = shortcut_display {
                content = content.child(focused_info_shortcut_section(
                    shortcut_str,
                    style,
                ));
            }

            // Path
            content = content.child(focused_info_labeled_section(
                "PATH",
                &app.path.to_string_lossy(),
                style,
            ));

            // Bundle ID
            if let Some(bundle_id) = &app.bundle_id {
                content = content.child(focused_info_labeled_section(
                    "BUNDLE ID",
                    bundle_id,
                    style,
                ));
            }

            // Divider + Type indicator
            content = content
                .child(focused_info_divider(style))
                .child(focused_info_type_indicator("Application", style));
        }

        scripts::SearchResult::Window(window_match) => {
            let window = &window_match.window;

            // Title header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_sm))
                    .child(window.title.clone()),
            );

            // Application
            content = content.child(focused_info_labeled_section(
                "APPLICATION",
                &window.app,
                style,
            ));

            // Position & Size
            content = content.child(focused_info_labeled_section(
                "POSITION & SIZE",
                &format!(
                    "{}×{} at ({}, {})",
                    window.bounds.width,
                    window.bounds.height,
                    window.bounds.x,
                    window.bounds.y
                ),
                style,
            ));

            // Divider + Type indicator
            content = content
                .child(focused_info_divider(style))
                .child(focused_info_type_indicator("Window", style));
        }

        scripts::SearchResult::Agent(agent_match) => {
            let agent = &agent_match.agent;

            // Source indicator
            let filename = agent
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "agent".to_string());

            let mut path_div = div()
                .flex()
                .flex_row()
                .text_xs()
                .font_family(t.font_family_mono)
                .pb(px(s.padding_xs))
                .overflow_x_hidden()
                .child(
                    div()
                        .text_color(rgba((style.text_muted << 8) | 0x99))
                        .child("agent: "),
                );

            path_div = path_div.child(
                div()
                    .text_color(rgba((style.text_muted << 8) | 0x99))
                    .child(filename),
            );

            content = content.child(path_div);

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_sm))
                    .child(agent.name.clone()),
            );

            // Description
            if let Some(desc) = &agent.description {
                content = content.child(
                    div()
                        .text_sm()
                        .text_color(rgb(style.text_secondary))
                        .pb(px(s.padding_md))
                        .child(desc.clone()),
                );
            }

            // Backend
            content = content.child(focused_info_labeled_section(
                "BACKEND",
                &format!("{:?}", agent.backend),
                style,
            ));

            // Kit
            if let Some(kit) = &agent.kit {
                content = content.child(focused_info_labeled_section(
                    "KIT",
                    kit,
                    style,
                ));
            }

            // Divider + Type indicator
            content = content
                .child(focused_info_divider(style))
                .child(focused_info_type_indicator("Agent", style));
        }

        scripts::SearchResult::Fallback(fallback_match) => {
            let fallback = &fallback_match.fallback;

            // Source indicator
            let mut path_div = div()
                .flex()
                .flex_row()
                .text_xs()
                .font_family(t.font_family_mono)
                .pb(px(s.padding_xs))
                .overflow_x_hidden()
                .child(
                    div()
                        .text_color(rgba((style.text_muted << 8) | 0x99))
                        .child("fallback: "),
                );

            path_div = path_div.child(
                div()
                    .text_color(rgba((style.text_muted << 8) | 0x99))
                    .child(fallback.name().to_string()),
            );

            content = content.child(path_div);

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_sm))
                    .child(fallback.label().to_string()),
            );

            // Description
            content = content.child(
                div()
                    .text_sm()
                    .text_color(rgb(style.text_secondary))
                    .pb(px(s.padding_md))
                    .child(fallback.description().to_string()),
            );

            // Divider + Type indicator
            content = content
                .child(focused_info_divider(style))
                .child(focused_info_type_indicator("Fallback", style));
        }

        scripts::SearchResult::Skill(skill_match) => {
            let skill = &skill_match.skill;

            // Source indicator
            let mut path_div = div()
                .flex()
                .flex_row()
                .text_xs()
                .font_family(t.font_family_mono)
                .pb(px(s.padding_xs))
                .overflow_x_hidden()
                .child(
                    div()
                        .text_color(rgba((style.text_muted << 8) | 0x99))
                        .child("skill: "),
                );

            path_div = path_div.child(
                div()
                    .text_color(rgba((style.text_muted << 8) | 0x99))
                    .child(format!("{}/{}", skill.plugin_id, skill.skill_id)),
            );

            content = content.child(path_div);

            // Name header
            content = content.child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(style.text_primary))
                    .pb(px(s.padding_sm))
                    .child(skill.title.clone()),
            );

            // Description
            if !skill.description.is_empty() {
                content = content.child(
                    div()
                        .text_sm()
                        .text_color(rgb(style.text_secondary))
                        .pb(px(s.padding_md))
                        .child(skill.description.clone()),
                );
            }

            // Plugin badge
            content = content.child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(s.gap_sm))
                    .pb(px(s.padding_sm))
                    .child(focused_info_badge(
                        &format!("plugin: {}", skill.plugin_title),
                        style.badge_bg_rgba,
                        style.badge_border_rgba,
                        style.badge_text_hex,
                    )),
            );

            // Divider + Type indicator
            content = content
                .child(focused_info_divider(style))
                .child(focused_info_type_indicator("Skill", style));
        }
    }

    content
}

// --- Shared rendering helpers ---

/// Render a small badge element with the given text and colors.
fn focused_info_badge(text: &str, bg_rgba: u32, border_rgba: u32, text_hex: u32) -> Div {
    div()
        .px(px(6.))
        .py(px(2.))
        .rounded(px(4.))
        .bg(rgba(bg_rgba))
        .border_1()
        .border_color(rgba(border_rgba))
        .text_xs()
        .text_color(rgb(text_hex))
        .child(text.to_string())
}

/// Render a keyboard shortcut section with label and value.
fn focused_info_shortcut_section(shortcut_str: &str, style: &FocusedInfoStyle) -> Div {
    div()
        .flex()
        .flex_col()
        .pb(px(style.spacing.padding_lg))
        .child(
            div()
                .text_size(px(style.section_label_font_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba((style.text_muted << 8) | 0xCC))
                .pb(px(style.spacing.padding_xs))
                .child("KEYBOARD SHORTCUT"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(style.spacing.gap_sm))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(style.accent_badge_text_hex))
                        .child(shortcut_str.to_string()),
                ),
        )
}

/// Render a labeled metadata section (label + value).
fn focused_info_labeled_section(label: &str, value: &str, style: &FocusedInfoStyle) -> Div {
    div()
        .flex()
        .flex_col()
        .pb(px(style.spacing.padding_lg))
        .child(
            div()
                .text_size(px(style.section_label_font_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba((style.text_muted << 8) | 0xCC))
                .pb(px(style.spacing.padding_xs))
                .child(label.to_string()),
        )
        .child(
            div()
                .text_sm()
                .line_height(px(style.body_text_line_height))
                .text_color(rgb(style.text_secondary))
                .child(value.to_string()),
        )
}

/// Render a horizontal divider line.
fn focused_info_divider(style: &FocusedInfoStyle) -> Div {
    div()
        .w_full()
        .h(px(style.visual.border_thin))
        .bg(rgba((style.ui_border << 8) | 0x60))
        .my(px(style.spacing.padding_sm))
}

/// Render a type indicator section (e.g., "Application", "Window", "Agent").
fn focused_info_type_indicator(type_name: &str, style: &FocusedInfoStyle) -> Div {
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .text_size(px(style.section_label_font_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba((style.text_muted << 8) | 0xCC))
                .pb(px(style.spacing.padding_xs))
                .child("TYPE"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(style.text_secondary))
                .child(type_name.to_string()),
        )
}

// --- Existing ScriptListApp methods ---

impl ScriptListApp {
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the minimum score threshold for suggested items
        let min_score = self.config.get_suggested().min_score;

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        tracing::trace!(
            event = "get_focused_script_info",
            selected_index = self.selected_index,
            grouped_items_len = grouped_items.len(),
            result_idx = ?result_idx,
            "get_focused_script_info",
        );

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                // Compute frecency path for each result type (same logic as app_impl.rs)
                let frecency_path: Option<String> = match result {
                    scripts::SearchResult::Script(m) => {
                        Some(m.script.path.to_string_lossy().to_string())
                    }
                    scripts::SearchResult::Scriptlet(m) => {
                        Some(format!("scriptlet:{}", m.scriptlet.name))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Check if excluded from frecency tracking
                        let excluded = &self.config.get_suggested().excluded_commands;
                        if m.entry.should_exclude_from_frecency(excluded) {
                            None
                        } else {
                            Some(format!("builtin:{}", m.entry.id))
                        }
                    }
                    scripts::SearchResult::App(m) => Some(m.app.path.to_string_lossy().to_string()),
                    scripts::SearchResult::Window(m) => {
                        Some(format!("window:{}:{}", m.window.app, m.window.title))
                    }
                    scripts::SearchResult::Agent(m) => {
                        Some(format!("agent:{}", m.agent.path.to_string_lossy()))
                    }
                    scripts::SearchResult::Skill(_) => None, // Skills don't track frecency
                    scripts::SearchResult::Fallback(_) => None, // Fallbacks don't track frecency
                };

                // Check if this item is "suggested" (has frecency data above min_score)
                let is_suggested = frecency_path
                    .as_ref()
                    .map(|path| self.frecency_store.get_score(path) >= min_score)
                    .unwrap_or(false);

                // Pre-compute launcher command ID and override lookups before the match
                // to avoid partial-move borrow issues within match arms.
                let launcher_cmd_id = result.launcher_command_id();
                let shortcut_overrides = crate::shortcuts::get_cached_shortcut_overrides();
                let alias_overrides = crate::aliases::get_cached_alias_overrides();
                let override_shortcut = launcher_cmd_id
                    .as_ref()
                    .and_then(|id| shortcut_overrides.get(id).map(|s| s.to_string()));
                let override_alias = launcher_cmd_id
                    .as_ref()
                    .and_then(|id| alias_overrides.get(id).cloned());

                match result {
                    scripts::SearchResult::Script(m) => {
                        // Launcher-managed overrides take precedence over inline metadata.
                        let shortcut = override_shortcut.or_else(|| m.script.shortcut.clone());
                        let alias = override_alias.or_else(|| m.script.alias.clone());
                        Some(
                            ScriptInfo::with_shortcut_and_alias(
                                &m.script.name,
                                m.script.path.to_string_lossy(),
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets use the markdown file path for edit/reveal actions
                        // Extract the path without anchor for file operations
                        let markdown_path = m
                            .scriptlet
                            .file_path
                            .as_ref()
                            .map(|p| p.split('#').next().unwrap_or(p).to_string())
                            .unwrap_or_else(|| format!("scriptlet:{}", &m.scriptlet.name));
                        // Launcher-managed overrides take precedence over inline metadata.
                        let shortcut = override_shortcut.or_else(|| m.scriptlet.shortcut.clone());
                        let alias = override_alias.or_else(|| m.scriptlet.alias.clone());
                        Some(
                            ScriptInfo::scriptlet(
                                &m.scriptlet.name,
                                markdown_path,
                                shortcut,
                                alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        // is_script=false: no editable file, hide "Edit Script" etc.
                        Some(
                            ScriptInfo::with_all(
                                &m.entry.name,
                                format!("builtin:{}", &m.entry.id),
                                false,
                                "Run",
                                override_shortcut,
                                override_alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        // is_app=true: enables app-specific actions (Finder, Process, Copy)
                        Some(
                            ScriptInfo::app(
                                &m.app.name,
                                m.app.path.to_string_lossy().to_string(),
                                m.app.bundle_id.clone(),
                                override_shortcut,
                                override_alias,
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        // is_script=false: windows aren't editable scripts
                        Some(
                            ScriptInfo::with_action_verb(
                                &m.window.title,
                                format!("window:{}", m.window.id),
                                false,
                                "Switch to",
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Agent(m) => {
                        // Agents use their path as identifier
                        Some(
                            ScriptInfo::new(
                                &m.agent.name,
                                format!("agent:{}", m.agent.path.to_string_lossy()),
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Skill(m) => {
                        // Skills use plugin_id/skill_id as identifier
                        // is_script=false: skills aren't editable scripts
                        Some(
                            ScriptInfo::with_action_verb(
                                &m.skill.title,
                                format!("skill:{}:{}", m.skill.plugin_id, m.skill.skill_id),
                                false,
                                "Open",
                            )
                            .with_frecency(is_suggested, frecency_path),
                        )
                    }
                    scripts::SearchResult::Fallback(m) => {
                        // Fallbacks use their name as identifier
                        // is_script depends on whether it's a built-in fallback or script-based
                        // Fallbacks don't track frecency, so is_suggested is always false
                        Some(ScriptInfo::with_action_verb(
                            m.fallback.name(),
                            format!("fallback:{}", m.fallback.name()),
                            !m.fallback.is_builtin(),
                            "Run",
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the full scriptlet with actions for the currently focused item
    ///
    /// This re-parses the markdown file to get the scriptlet's H3 actions
    /// and shared actions from the companion .actions.md file.
    /// Returns None if the focused item is not a scriptlet.
    pub fn get_focused_scriptlet_with_actions(&mut self) -> Option<crate::scriptlets::Scriptlet> {
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(scripts::SearchResult::Scriptlet(m)) = flat_results.get(idx) {
                // Get the file path from the UI scriptlet type
                let file_path = m.scriptlet.file_path.clone()?;
                let scriptlet_command = m.scriptlet.command.clone()?;

                // Extract just the file path (before #anchor)
                let file_only = file_path.split('#').next().unwrap_or(&file_path);

                // Read and parse the markdown file to get full scriptlet with actions
                if let Ok(content) = std::fs::read_to_string(file_only) {
                    let parsed_scriptlets =
                        crate::scriptlets::parse_markdown_as_scriptlets(&content, Some(file_only));

                    // Find the matching scriptlet by command
                    return parsed_scriptlets
                        .into_iter()
                        .find(|s| s.command == scriptlet_command);
                }
            }
        }

        None
    }
}
