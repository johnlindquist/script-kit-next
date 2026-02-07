fn preview_keyword_tags(keywords: &[String]) -> Vec<String> {
    let mut tags = Vec::new();

    for keyword in keywords {
        let normalized = keyword.trim().to_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if tags.iter().any(|tag| tag == &normalized) {
            continue;
        }
        tags.push(normalized);
        if tags.len() >= 6 {
            break;
        }
    }

    tags
}

fn builtin_feature_annotation(feature: &builtins::BuiltInFeature) -> String {
    match feature {
        builtins::BuiltInFeature::ClipboardHistory => "Clipboard History Manager".to_string(),
        builtins::BuiltInFeature::AppLauncher => "Application Launcher".to_string(),
        builtins::BuiltInFeature::App(name) => name.clone(),
        builtins::BuiltInFeature::WindowSwitcher => "Window Manager".to_string(),
        builtins::BuiltInFeature::DesignGallery => "Design Gallery".to_string(),
        builtins::BuiltInFeature::AiChat => "AI Assistant".to_string(),
        builtins::BuiltInFeature::Notes => "Notes & Scratchpad".to_string(),
        builtins::BuiltInFeature::MenuBarAction(_) => "Menu Bar Action".to_string(),
        builtins::BuiltInFeature::SystemAction(_) => "System Action".to_string(),
        builtins::BuiltInFeature::NotesCommand(_) => "Notes Command".to_string(),
        builtins::BuiltInFeature::AiCommand(_) => "AI Command".to_string(),
        builtins::BuiltInFeature::ScriptCommand(_) => "Script Creation".to_string(),
        builtins::BuiltInFeature::PermissionCommand(_) => "Permission Management".to_string(),
        builtins::BuiltInFeature::FrecencyCommand(_) => "Suggested Items".to_string(),
        builtins::BuiltInFeature::UtilityCommand(_) => "Quick Utility".to_string(),
        builtins::BuiltInFeature::SettingsCommand(_) => "Settings".to_string(),
        builtins::BuiltInFeature::FileSearch => "File Browser".to_string(),
        builtins::BuiltInFeature::Webcam => "Webcam Capture".to_string(),
    }
}

/// Helper function to render a group header style item with actual visual styling
fn render_group_header_item(
    ix: usize,
    is_selected: bool,
    style: &designs::group_header_variations::GroupHeaderStyle,
    spacing: &designs::DesignSpacing,
    typography: &designs::DesignTypography,
    visual: &designs::DesignVisual,
    colors: &designs::DesignColors,
) -> AnyElement {
    use designs::group_header_variations::GroupHeaderStyle;

    let name_owned = style.name().to_string();
    let desc_owned = style.description().to_string();

    let mut item_div = div()
        .id(ElementId::NamedInteger("gallery-header".into(), ix as u64))
        .w_full()
        .h(px(LIST_ITEM_HEIGHT))
        .px(px(spacing.padding_lg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.gap_md));

    if is_selected {
        // Use low-opacity white for vibrancy support (see VIBRANCY.md)
        item_div = item_div.bg(rgba((colors.background_selected << 8) | 0x0f)); // ~6% opacity
    }

    let preview = render_group_header_preview(style, typography, visual, colors);

    item_div
        // Preview element
        .child(preview)
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child(name_owned),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(desc_owned),
                ),
        )
        .into_any_element()
}
