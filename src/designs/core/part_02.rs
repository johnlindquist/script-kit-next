///
/// # Arguments
/// * `variant` - The design variant to render
/// * `result` - The search result to render
/// * `index` - The item index (for element ID and alternating styles)
/// * `is_selected` - Whether this item is currently selected (full focus styling)
/// * `is_hovered` - Whether this item is currently hovered (subtle visual feedback)
/// * `list_colors` - Pre-computed theme colors for the default design
/// * `enable_hover_effect` - Whether to enable instant hover effects (false during keyboard navigation)
/// * `filter_text` - Current search filter text (empty when not filtering; used for fuzzy match highlighting)
///
/// # Returns
/// An `AnyElement` containing the rendered item
#[allow(clippy::too_many_arguments)]
pub fn render_design_item(
    variant: DesignVariant,
    result: &SearchResult,
    index: usize,
    is_selected: bool,
    is_hovered: bool,
    list_colors: ListItemColors,
    enable_hover_effect: bool,
    filter_text: &str,
) -> AnyElement {
    // NOTE: Removed per-item DEBUG log that was causing log spam.
    // This function is called for every visible list item on every render frame.
    // With cursor blink triggering renders every 530ms, this logged 8-9 items × 2 renders/sec.

    match variant {
        DesignVariant::Minimal => {
            let colors = MinimalColors {
                text_primary: list_colors.text_primary,
                text_muted: list_colors.text_muted,
                accent_selected: list_colors.accent_selected,
                background: list_colors.background,
            };
            MinimalRenderer::new()
                .render_item(result, index, is_selected, colors)
                .into_any_element()
        }
        DesignVariant::RetroTerminal => RetroTerminalRenderer::new()
            .render_item(result, index, is_selected)
            .into_any_element(),
        // All other variants use the default ListItem renderer
        _ => {
            use crate::list_item::{IconKind, ListItem};

            // Compute fuzzy match indices for highlighting when actively filtering
            let (highlight_indices, description_highlight_indices) = if !filter_text.is_empty() {
                let indices =
                    crate::scripts::search::compute_match_indices_for_result(result, filter_text);
                let name_hi = if indices.name_indices.is_empty() {
                    None
                } else {
                    Some(indices.name_indices)
                };
                let desc_hi = if indices.description_indices.is_empty() {
                    None
                } else {
                    Some(indices.description_indices)
                };
                (name_hi, desc_hi)
            } else {
                (None, None)
            };

            // Extract name, description, shortcut, and icon based on result type
            let (name, description, shortcut, icon_kind) = match result {
                SearchResult::Script(sm) => {
                    // Use script's icon metadata if present, otherwise default to "Code" SVG
                    let icon = match &sm.script.icon {
                        Some(icon_name) => IconKind::Svg(icon_name.clone()),
                        None => {
                            IconKind::Svg(extension_default_icon(&sm.script.extension).to_string())
                        }
                    };
                    // Scripts: show shortcut or alias as badge (matching scriptlet pattern)
                    let badge = sm
                        .script
                        .shortcut
                        .clone()
                        .or_else(|| sm.script.alias.clone());
                    // Auto-generate description for scripts without one.
                    // Uses extracted helper: priority is explicit > property detail > filename
                    let description = auto_description_for_script(&sm.script);
                    (sm.script.name.clone(), description, badge, Some(icon))
                }
                SearchResult::Scriptlet(sm) => {
                    // Scriptlets: show shortcut, keyword, or alias as badge
                    let badge = sm
                        .scriptlet
                        .shortcut
                        .clone()
                        .or_else(|| sm.scriptlet.keyword.clone())
                        .or_else(|| sm.scriptlet.alias.clone());
                    // Differentiate scriptlet icon by tool type
                    let icon = match sm.scriptlet.tool.as_str() {
                        "bash" | "sh" | "zsh" => IconKind::Svg("Terminal".to_string()),
                        "paste" | "snippet" => IconKind::Svg("Copy".to_string()),
                        "open" => IconKind::Svg("PlayFilled".to_string()),
                        _ => IconKind::Svg("BoltFilled".to_string()),
                    };
                    // Auto-generate description: prefer code preview, fall back to tool name
                    // Code preview gives users immediate insight into what the scriptlet does
                    let description = sm.scriptlet.description.clone().or_else(|| {
                        code_preview_for_scriptlet(&sm.scriptlet)
                            .or_else(|| Some(sm.scriptlet.tool_display_name().to_string()))
                    });
                    (sm.scriptlet.name.clone(), description, badge, Some(icon))
                }
                SearchResult::BuiltIn(bm) => {
                    // Built-ins: try to map their icon to SVG, fallback to Settings
                    let icon = match &bm.entry.icon {
                        Some(emoji) => {
                            // Try to infer SVG from common emoji patterns
                            match emoji.as_str() {
                                "⚙️" | "🔧" => IconKind::Svg("Settings".to_string()),
                                "📋" => IconKind::Svg("Copy".to_string()),
                                "🔍" | "🔎" => IconKind::Svg("MagnifyingGlass".to_string()),
                                "📁" => IconKind::Svg("Folder".to_string()),
                                "🖥️" | "💻" => IconKind::Svg("Terminal".to_string()),
                                "⚡" | "🔥" => IconKind::Svg("BoltFilled".to_string()),
                                "⭐" | "🌟" => IconKind::Svg("StarFilled".to_string()),
                                "✓" | "✅" => IconKind::Svg("Check".to_string()),
                                "▶️" | "🎬" => IconKind::Svg("PlayFilled".to_string()),
                                "🗑️" => IconKind::Svg("Trash".to_string()),
                                "➕" => IconKind::Svg("Plus".to_string()),
                                _ => IconKind::Svg("Settings".to_string()),
                            }
                        }
                        None => IconKind::Svg("Settings".to_string()),
                    };
                    (
                        bm.entry.name.clone(),
                        Some(bm.entry.description.clone()),
                        None,
                        Some(icon),
                    )
                }
                SearchResult::App(am) => {
                    // Apps use pre-decoded icons, fallback to File SVG
                    let icon = match &am.app.icon {
                        Some(img) => IconKind::Image(img.clone()),
                        None => IconKind::Svg("File".to_string()),
                    };
                    (am.app.name.clone(), None, None, Some(icon))
                }
                SearchResult::Window(wm) => {
                    // Windows get a generic File icon, title as name, app as description
                    (
                        wm.window.title.clone(),
                        Some(wm.window.app.clone()),
                        None,
                        Some(IconKind::Svg("File".to_string())),
                    )
                }
                SearchResult::Agent(am) => {
                    // Agents use backend-specific icons, with backend label in description
                    let icon_name = am
                        .agent
                        .icon
                        .clone()
                        .unwrap_or_else(|| am.agent.backend.icon().to_string());
                    let backend_label = am.agent.backend.label();
                    let description = am
                        .agent
                        .description
                        .clone()
                        .or_else(|| Some(format!("{} Agent", backend_label)));
                    (
                        am.agent.name.clone(),
                        description,
                        am.agent.shortcut.clone(),
                        Some(IconKind::Svg(icon_name)),
                    )
                }
                SearchResult::Fallback(fm) => {
                    // Fallback commands from "Use with..." section
                    // Map fallback icon names to SVG icons
                    let icon_name = match fm.fallback.icon() {
                        "external-link" => "ExternalLink",
                        "calculator" => "Calculator",
                        "file" => "File",
                        "terminal" => "Terminal",
                        "sticky-note" => "StickyNote",
                        "clipboard-copy" => "Copy",
                        "search" => "MagnifyingGlass",
                        other => other, // Pass through if already a valid icon name
                    };
                    (
                        fm.fallback.label().to_string(),
                        Some(fm.fallback.description().to_string()),
                        None,
                        Some(IconKind::Svg(icon_name.to_string())),
                    )
                }
            };

            // During search mode, keep only quiet type labels.
            // During grouped mode, use discoverability hints.
            let (type_tag, source_hint) = if !filter_text.is_empty() {
                let accessories = resolve_search_accessories(result, filter_text);
                (accessories.type_tag, accessories.source_hint)
            } else {
                // Grouped view: use extracted helpers for discoverability hints
                let hint = match result {
                    SearchResult::Script(sm) => grouped_view_hint_for_script(&sm.script),
                    SearchResult::Scriptlet(sm) => grouped_view_hint_for_scriptlet(&sm.scriptlet),
                    _ => None,
                };
                (None, hint)
            };

            // Tool/language badge for scriptlets (e.g., "ts", "bash", "paste")
            // For scripts: show property indicator if the script has special runtime behavior
            // (cron/schedule, file watch, background, system)
            let tool_badge = resolve_tool_badge(result, !filter_text.is_empty());

            ListItem::new(name, list_colors)
                .index(index)
                .icon_kind_opt(icon_kind)
                .shortcut_opt(shortcut)
                .description_opt(description)
                .selected(is_selected)
                .hovered(is_hovered)
                .with_accent_bar(true)
                .with_hover_effect(enable_hover_effect)
                .highlight_indices_opt(highlight_indices)
                .description_highlight_indices_opt(description_highlight_indices)
                .type_tag_opt(type_tag)
                .source_hint_opt(source_hint)
                .tool_badge_opt(tool_badge)
                .into_any_element()
        }
    }
}

/// Map a file extension to a human-readable language/tool name.
/// Used as a last-resort fallback description for scripts with no other context.
pub(crate) fn extension_language_label(extension: &str) -> Option<&'static str> {
    match extension {
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("JavaScript"),
        "sh" | "bash" => Some("Shell script"),
        "zsh" => Some("Zsh script"),
        "py" => Some("Python script"),
        "rb" => Some("Ruby script"),
        "applescript" | "scpt" => Some("AppleScript"),
        _ => None,
    }
}

/// Auto-generate a fallback description for scripts that have no explicit description.
/// Priority: schedule expression > cron expression > watch pattern > background > system > filename
pub(crate) fn auto_description_for_script(script: &crate::scripts::Script) -> Option<String> {
    // If the script has an explicit description, return it as-is
    if script.description.is_some() {
        return script.description.clone();
    }

    // Try metadata-based descriptions
    if let Some(ref meta) = script.typed_metadata {
        if let Some(ref schedule) = meta.schedule {
            return Some(format!("Scheduled: {}", schedule));
        }
        if let Some(ref cron) = meta.cron {
            return Some(format!("Cron: {}", cron));
        }
        if let Some(first_pattern) = meta.watch.first() {
            let display = if first_pattern.len() > 40 {
                format!("{}...", &first_pattern[..37])
            } else {
                first_pattern.clone()
            };
            return Some(format!("Watches: {}", display));
        }
        if meta.background {
            return Some("Background process".to_string());
        }
        if meta.system {
            return Some("System event handler".to_string());
        }
    }

    // Fallback: show filename when it differs from the display name
    let filename = crate::scripts::search::extract_filename(&script.path);
    if !filename.is_empty() && filename != script.name {
        Some(filename)
    } else {
        // Last resort: show language name based on extension
        extension_language_label(&script.extension).map(|s| s.to_string())
    }
}

/// Determine the grouped-view source hint for a script.
/// Priority: alias (when shortcut is badge) > tags > kit name (non-main)
pub(crate) fn grouped_view_hint_for_script(script: &crate::scripts::Script) -> Option<String> {
    if script.shortcut.is_some() {
        // Shortcut is the badge → show alias as trigger hint, then tags, then kit
        script
            .alias
            .as_ref()
            .map(|a| format!("/{}", a))
            .or_else(|| {
                script.typed_metadata.as_ref().and_then(|meta| {
                    if !meta.tags.is_empty() {
                        Some(
                            meta.tags
                                .iter()
                                .take(2)
                                .map(|t| t.as_str())
                                .collect::<Vec<_>>()
                                .join(" · "),
                        )
                    } else {
                        None
                    }
                })
            })
    } else if script.alias.is_some() {
        // Alias is the badge → show tags, then kit
        script
            .typed_metadata
            .as_ref()
            .and_then(|meta| {
                if !meta.tags.is_empty() {
                    Some(
                        meta.tags
                            .iter()
                            .take(2)
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                } else {
                    None
                }
            })
            .or_else(|| {
                script
                    .kit_name
                    .as_deref()
                    .filter(|k| *k != "main")
                    .map(|k| k.to_string())
            })
    } else {
        // No badge → show tags, then kit name, then custom enter text as action hint
        script
            .typed_metadata
            .as_ref()
            .and_then(|meta| {
                if !meta.tags.is_empty() {
                    Some(
                        meta.tags
                            .iter()
                            .take(2)
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                } else {
                    None
                }
            })
            .or_else(|| {
                script
                    .kit_name
                    .as_deref()
                    .filter(|k| *k != "main")
                    .map(|k| k.to_string())
            })
            .or_else(|| {
                // Final fallback: custom enter text as action hint (e.g., "→ Execute")
                script
                    .typed_metadata
                    .as_ref()
                    .and_then(|m| m.enter.as_deref())
                    .filter(|e| *e != "Run" && *e != "Run Script")
                    .map(|e| format!("→ {}", e))
            })
    }
}

/// Determine the grouped-view source hint for a scriptlet.
/// Priority: hidden trigger keyword/alias > group name (non-main)
pub(crate) fn grouped_view_hint_for_scriptlet(
    scriptlet: &crate::scripts::Scriptlet,
) -> Option<String> {
    if scriptlet.shortcut.is_some() {
        scriptlet
            .keyword
            .as_ref()
            .or(scriptlet.alias.as_ref())
            .map(|k| format!("/{}", k))
    } else if scriptlet.keyword.is_some() {
        scriptlet.alias.as_ref().map(|a| format!("/{}", a))
    } else {
        scriptlet
            .group
            .as_deref()
            .filter(|g| *g != "main")
            .map(|g| g.to_string())
    }
}

/// Generate a code preview for scriptlets without explicit descriptions.
/// Shows the first meaningful line(s) of code, truncated to fit the description area.
/// For paste/snippet tools, this shows the pasted content; for open, the URL;
/// for code tools, the first non-comment line.
/// When the first line is very short (< 20 chars), appends the second line for richer context.
pub(crate) fn code_preview_for_scriptlet(scriptlet: &crate::scripts::Scriptlet) -> Option<String> {
    let code = &scriptlet.code;
    if code.is_empty() {
        return None;
    }

    // Collect meaningful (non-empty, non-comment) lines
    let meaningful_lines: Vec<&str> = code
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("//")
                && !l.starts_with("/*")
                && !l.starts_with('*')
                && !l.starts_with("#!/")
        })
        .collect();

    let first_line = meaningful_lines.first()?;
    if first_line.is_empty() {
        return None;
    }

    let first_len = first_line.chars().count();

    // For very short first lines, append the second line for richer context
    // e.g., "cd ~/projects → npm start"
    let preview = if first_len < 20 {
        if let Some(second_line) = meaningful_lines.get(1) {
            let combined = format!("{} → {}", first_line, second_line);
            let combined_len = combined.chars().count();
            if combined_len > 60 {
                let truncated: String = combined.chars().take(57).collect();
                format!("{}...", truncated)
            } else {
                combined
            }
        } else {
            first_line.to_string()
        }
    } else if first_len > 60 {
        let truncated: String = first_line.chars().take(57).collect();
        format!("{}...", truncated)
    } else {
        first_line.to_string()
    };

    Some(preview)
}

