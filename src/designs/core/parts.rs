use super::*;

// --- merged from part_01.rs ---
/// Design variant enumeration
///
/// Each variant represents a distinct visual style for the script list.
/// Use `Cmd+1` through `Cmd+0` to switch between designs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DesignVariant {
    /// Default design (uses existing implementation)
    /// Keyboard: Cmd+1
    #[default]
    Default = 1,

    /// Minimal design with reduced visual elements
    /// Keyboard: Cmd+2
    Minimal = 2,

    /// Retro terminal aesthetic with monospace fonts and green-on-black
    /// Keyboard: Cmd+3
    RetroTerminal = 3,

    /// Glassmorphism with frosted glass effects and transparency
    /// Keyboard: Cmd+4
    Glassmorphism = 4,

    /// Brutalist design with raw, bold typography
    /// Keyboard: Cmd+5
    Brutalist = 5,

    /// Neon cyberpunk with glowing accents and dark backgrounds
    /// Keyboard: Cmd+6
    NeonCyberpunk = 6,

    /// Paper-like design with warm tones and subtle shadows
    /// Keyboard: Cmd+7
    Paper = 7,

    /// Apple Human Interface Guidelines inspired design
    /// Keyboard: Cmd+8
    AppleHIG = 8,

    /// Material Design 3 (Material You) inspired design
    /// Keyboard: Cmd+9
    Material3 = 9,

    /// Compact design with smaller items for power users
    /// Keyboard: Cmd+0
    Compact = 10,

    /// Playful design with rounded corners and vibrant colors
    /// Not directly accessible via keyboard shortcut
    Playful = 11,
}

impl DesignVariant {
    /// Get all available design variants
    pub fn all() -> &'static [DesignVariant] {
        &[
            DesignVariant::Default,
            DesignVariant::Minimal,
            DesignVariant::RetroTerminal,
            DesignVariant::Glassmorphism,
            DesignVariant::Brutalist,
            DesignVariant::NeonCyberpunk,
            DesignVariant::Paper,
            DesignVariant::AppleHIG,
            DesignVariant::Material3,
            DesignVariant::Compact,
            DesignVariant::Playful,
        ]
    }

    /// Get the next design variant in the cycle
    ///
    /// Cycles through all designs: Default -> Minimal -> RetroTerminal -> ... -> Playful -> Default
    pub fn next(self) -> DesignVariant {
        let all = Self::all();
        let current_idx = all.iter().position(|&v| v == self).unwrap_or(0);
        let next_idx = (current_idx + 1) % all.len();
        all[next_idx]
    }

    /// Get the previous design variant in the cycle
    #[allow(dead_code)]
    pub fn prev(self) -> DesignVariant {
        let all = Self::all();
        let current_idx = all.iter().position(|&v| v == self).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            all.len() - 1
        } else {
            current_idx - 1
        };
        all[prev_idx]
    }

    /// Get the display name for this variant
    pub fn name(&self) -> &'static str {
        match self {
            DesignVariant::Default => "Default",
            DesignVariant::Minimal => "Minimal",
            DesignVariant::RetroTerminal => "Retro Terminal",
            DesignVariant::Glassmorphism => "Glassmorphism",
            DesignVariant::Brutalist => "Brutalist",
            DesignVariant::NeonCyberpunk => "Neon Cyberpunk",
            DesignVariant::Paper => "Paper",
            DesignVariant::AppleHIG => "Apple HIG",
            DesignVariant::Material3 => "Material 3",
            DesignVariant::Compact => "Compact",
            DesignVariant::Playful => "Playful",
        }
    }

    /// Get the keyboard shortcut number for this variant (1-10, where 0 = 10)
    #[allow(dead_code)]
    pub fn shortcut_number(&self) -> Option<u8> {
        match self {
            DesignVariant::Default => Some(1),
            DesignVariant::Minimal => Some(2),
            DesignVariant::RetroTerminal => Some(3),
            DesignVariant::Glassmorphism => Some(4),
            DesignVariant::Brutalist => Some(5),
            DesignVariant::NeonCyberpunk => Some(6),
            DesignVariant::Paper => Some(7),
            DesignVariant::AppleHIG => Some(8),
            DesignVariant::Material3 => Some(9),
            DesignVariant::Compact => Some(0), // Cmd+0 maps to 10
            DesignVariant::Playful => None,    // No direct shortcut
        }
    }

    /// Create a variant from a keyboard number (1-9, 0 for 10)
    #[allow(dead_code)]
    pub fn from_keyboard_number(num: u8) -> Option<DesignVariant> {
        match num {
            1 => Some(DesignVariant::Default),
            2 => Some(DesignVariant::Minimal),
            3 => Some(DesignVariant::RetroTerminal),
            4 => Some(DesignVariant::Glassmorphism),
            5 => Some(DesignVariant::Brutalist),
            6 => Some(DesignVariant::NeonCyberpunk),
            7 => Some(DesignVariant::Paper),
            8 => Some(DesignVariant::AppleHIG),
            9 => Some(DesignVariant::Material3),
            0 => Some(DesignVariant::Compact),
            _ => None,
        }
    }

    /// Get a short description of this design variant
    pub fn description(&self) -> &'static str {
        match self {
            DesignVariant::Default => "The standard Script Kit appearance",
            DesignVariant::Minimal => "Clean and minimal with reduced visual noise",
            DesignVariant::RetroTerminal => "Classic terminal aesthetics with green phosphor glow",
            DesignVariant::Glassmorphism => "Frosted glass effects with soft transparency",
            DesignVariant::Brutalist => "Bold, raw typography with strong contrast",
            DesignVariant::NeonCyberpunk => "Dark backgrounds with vibrant neon accents",
            DesignVariant::Paper => "Warm, paper-like tones with subtle textures",
            DesignVariant::AppleHIG => "Following Apple Human Interface Guidelines",
            DesignVariant::Material3 => "Google Material Design 3 (Material You) inspired",
            DesignVariant::Compact => "Dense layout for power users with many scripts",
            DesignVariant::Playful => "Fun, rounded design with vibrant colors",
        }
    }
}

/// Check if a variant uses the default renderer
///
/// When true, ScriptListApp should use its built-in render_script_list()
/// instead of delegating to a custom design renderer.
///
/// Currently all variants use the default renderer until custom implementations
/// are added. In the future, only DesignVariant::Default will return true here.
#[allow(dead_code)]
pub fn uses_default_renderer(variant: DesignVariant) -> bool {
    // When a custom renderer is added for a variant, remove it from this match
    // Minimal, RetroTerminal now have custom renderers
    matches!(
        variant,
        DesignVariant::Default
            | DesignVariant::Glassmorphism
            | DesignVariant::Brutalist
            | DesignVariant::NeonCyberpunk
            | DesignVariant::Paper
            | DesignVariant::AppleHIG
            | DesignVariant::Material3
            | DesignVariant::Compact
            | DesignVariant::Playful
    )
}

/// Get the item height for a design variant
///
/// Different designs use different item heights for their aesthetic.
/// This should be used when setting up uniform_list.
///
/// Note: This function now uses the DesignTokens system for consistency.
/// The constants MINIMAL_ITEM_HEIGHT, TERMINAL_ITEM_HEIGHT, etc. are
/// kept for backward compatibility with existing renderers.
#[allow(dead_code)]
pub fn get_item_height(variant: DesignVariant) -> f32 {
    // Use tokens for authoritative item heights
    get_tokens(variant).item_height()
}

/// Get design tokens for a design variant
///
/// Returns a boxed trait object that provides the complete design token set
/// for the specified variant. Use this when you need dynamic dispatch.
///
pub fn get_tokens(variant: DesignVariant) -> Box<dyn DesignTokens> {
    match variant {
        DesignVariant::Default => Box::new(DefaultDesignTokens),
        DesignVariant::Minimal => Box::new(MinimalDesignTokens),
        DesignVariant::RetroTerminal => Box::new(RetroTerminalDesignTokens),
        DesignVariant::Glassmorphism => Box::new(GlassmorphismDesignTokens),
        DesignVariant::Brutalist => Box::new(BrutalistDesignTokens),
        DesignVariant::NeonCyberpunk => Box::new(NeonCyberpunkDesignTokens),
        DesignVariant::Paper => Box::new(PaperDesignTokens),
        DesignVariant::AppleHIG => Box::new(AppleHIGDesignTokens),
        DesignVariant::Material3 => Box::new(Material3DesignTokens),
        DesignVariant::Compact => Box::new(CompactDesignTokens),
        DesignVariant::Playful => Box::new(PlayfulDesignTokens),
    }
}

/// Get design tokens for a design variant (static dispatch version)
///
/// Returns the concrete token type for the specified variant.
/// Use this when you know the variant at compile time for better performance.
///
#[allow(dead_code)]
pub fn get_tokens_static<T: DesignTokens + Copy + Default>() -> T {
    T::default()
}

use crate::list_item::ListItemColors;
use crate::scripts::SearchResult;
use gpui::{AnyElement, IntoElement};

/// Render a single list item for the given design variant
///
/// This is the main dispatch function for design-specific item rendering.
/// It renders a single item based on the current design, with proper styling.
/// Map a script's file extension to a more appropriate default icon.
/// Returns an icon name like "Terminal" for shell scripts, "Code" for everything else.
/// Only used when the script has no explicit `// Icon:` metadata.
pub(crate) fn extension_default_icon(extension: &str) -> &'static str {
    match extension {
        "sh" | "bash" | "zsh" => "Terminal",
        "applescript" | "scpt" => "Terminal",
        _ => "Code",
    }
}

#[derive(Debug, Default)]
struct SearchAccessories {
    type_tag: Option<crate::list_item::TypeTag>,
    source_hint: Option<String>,
}

fn resolve_search_accessories(result: &SearchResult, filter_text: &str) -> SearchAccessories {
    if filter_text.is_empty() {
        return SearchAccessories::default();
    }

    // Search rows should stay calm: keep only a quiet type label.
    // Category/match-reason metadata is intentionally hidden.
    let (label, color) = result.type_tag_info();
    SearchAccessories {
        type_tag: Some(crate::list_item::TypeTag { label, color }),
        source_hint: None,
    }
}

fn resolve_tool_badge(result: &SearchResult, is_filtering: bool) -> Option<String> {
    if is_filtering {
        // Action/tool badges ("paste", "open", etc.) are too noisy during search.
        return None;
    }

    match result {
        SearchResult::Script(sm) => sm.script.typed_metadata.as_ref().and_then(|meta| {
            if meta.cron.is_some() || meta.schedule.is_some() {
                Some("cron".to_string())
            } else if !meta.watch.is_empty() {
                Some("watch".to_string())
            } else if meta.background {
                Some("bg".to_string())
            } else if meta.system {
                Some("sys".to_string())
            } else {
                None
            }
        }),
        SearchResult::Scriptlet(sm) => Some(sm.scriptlet.tool.clone()),
        _ => None,
    }
}

// --- merged from part_02.rs ---
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

fn truncate_str_chars(s: &str, max_chars: usize) -> &str {
    s.char_indices()
        .nth(max_chars)
        .map_or(s, |(index, _)| &s[..index])
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
            let display = if first_pattern.chars().count() > 40 {
                format!("{}...", truncate_str_chars(first_pattern, 37))
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

// --- merged from part_03.rs ---
/// Detect why a script matched the search query when the name didn't match directly.
/// Returns a concise reason string (e.g., "tag: productivity", "shortcut") for
/// display in the search source hint area, helping users understand search results.
#[cfg(test)]
pub(crate) fn detect_match_reason_for_script(
    script: &crate::scripts::Script,
    query: &str,
) -> Option<String> {
    if query.len() < 2 {
        return None;
    }
    let q = query.to_lowercase();

    // If name already matches, no need for a "via" indicator
    if crate::scripts::search::contains_ignore_ascii_case(&script.name, &q) {
        return None;
    }

    // Check metadata fields in priority order
    if let Some(ref meta) = script.typed_metadata {
        // Tags
        for tag in &meta.tags {
            if crate::scripts::search::contains_ignore_ascii_case(tag, &q) {
                return Some(format!("tag: {}", tag));
            }
        }
        // Author
        if let Some(ref author) = meta.author {
            if crate::scripts::search::contains_ignore_ascii_case(author, &q) {
                return Some(format!("by {}", author));
            }
        }
    }

    // Shortcut
    if let Some(ref shortcut) = script.shortcut {
        if crate::scripts::search::contains_ignore_ascii_case(shortcut, &q) {
            return Some("shortcut".to_string());
        }
    }

    // Kit name
    if let Some(ref kit) = script.kit_name {
        if kit != "main" && crate::scripts::search::contains_ignore_ascii_case(kit, &q) {
            return Some(format!("kit: {}", kit));
        }
    }

    // Alias (when not shown as badge - if shortcut exists, alias isn't the badge)
    if let Some(ref alias) = script.alias {
        if crate::scripts::search::contains_ignore_ascii_case(alias, &q) {
            return Some(format!("alias: /{}", alias));
        }
    }

    // Description excerpt - show brief matching context when description matched
    if let Some(ref desc) = script.description {
        if crate::scripts::search::contains_ignore_ascii_case(desc, &q) {
            let excerpt = excerpt_around_match(desc, &q, 40);
            return Some(format!("desc: {}", excerpt));
        }
    }

    // Path match - when path matched but nothing else above did
    if crate::scripts::search::contains_ignore_ascii_case(&script.path.to_string_lossy(), &q) {
        return Some("path match".to_string());
    }

    None
}

/// Extract a brief excerpt from text around the first match of a query.
/// Returns a truncated substring centered on the match, with ellipsis as needed.
/// `max_len` is the maximum character length of the returned excerpt.
#[cfg(test)]
pub(crate) fn excerpt_around_match(text: &str, query_lower: &str, max_len: usize) -> String {
    let text_chars: Vec<char> = text.chars().collect();
    let text_len = text_chars.len();

    if text_len <= max_len {
        return text.to_string();
    }

    // Find the match position (char-level search via lowercased text)
    let text_lower: String = text_chars.iter().map(|c| c.to_ascii_lowercase()).collect();
    let match_byte_pos = text_lower.find(query_lower).unwrap_or(0);
    // Convert byte position to char position
    let char_pos = text_lower[..match_byte_pos.min(text_lower.len())]
        .chars()
        .count();

    // Center the excerpt around the match
    let half = max_len / 2;
    let start = char_pos.saturating_sub(half);
    let end = (start + max_len).min(text_len);
    let start = if end == text_len && text_len > max_len {
        text_len - max_len
    } else {
        start
    };

    let excerpt: String = text_chars[start..end].iter().collect();
    if start > 0 && end < text_len {
        format!("...{}...", excerpt.trim())
    } else if start > 0 {
        format!("...{}", excerpt.trim())
    } else if end < text_len {
        format!("{}...", excerpt.trim())
    } else {
        excerpt
    }
}

/// Detect why a scriptlet matched the search query when the name didn't match directly.
/// Returns a concise reason string for display in search source hints.
#[cfg(test)]
pub(crate) fn detect_match_reason_for_scriptlet(
    scriptlet: &crate::scripts::Scriptlet,
    query: &str,
) -> Option<String> {
    if query.len() < 2 {
        return None;
    }
    let q = query.to_lowercase();

    // If name already matches, no need for indicator
    if crate::scripts::search::contains_ignore_ascii_case(&scriptlet.name, &q) {
        return None;
    }

    // Keyword
    if let Some(ref keyword) = scriptlet.keyword {
        if crate::scripts::search::contains_ignore_ascii_case(keyword, &q) {
            return Some(format!("keyword: {}", keyword));
        }
    }

    // Shortcut
    if let Some(ref shortcut) = scriptlet.shortcut {
        if crate::scripts::search::contains_ignore_ascii_case(shortcut, &q) {
            return Some("shortcut".to_string());
        }
    }

    // Group
    if let Some(ref group) = scriptlet.group {
        if group != "main" && crate::scripts::search::contains_ignore_ascii_case(group, &q) {
            return Some(format!("group: {}", group));
        }
    }

    // Alias
    if let Some(ref alias) = scriptlet.alias {
        if crate::scripts::search::contains_ignore_ascii_case(alias, &q) {
            return Some(format!("alias: /{}", alias));
        }
    }

    // Tool type (e.g., searching "bash" finds bash scriptlets)
    if crate::scripts::search::contains_ignore_ascii_case(&scriptlet.tool, &q) {
        return Some(format!("tool: {}", scriptlet.tool_display_name()));
    }

    // Description excerpt
    if let Some(ref desc) = scriptlet.description {
        if crate::scripts::search::contains_ignore_ascii_case(desc, &q) {
            let excerpt = excerpt_around_match(desc, &q, 35);
            return Some(format!("desc: {}", excerpt));
        }
    }

    // Code content (only for longer queries to avoid noise)
    if q.len() >= 4 && crate::scripts::search::contains_ignore_ascii_case(&scriptlet.code, &q) {
        return Some("code match".to_string());
    }

    None
}

// --- merged from part_04.rs ---
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
mod truncate_str_chars_tests {
    use super::truncate_str_chars;

    #[test]
    fn test_truncate_str_chars_returns_original_when_string_is_shorter() {
        assert_eq!(truncate_str_chars("short", 10), "short");
    }

    #[test]
    fn test_truncate_str_chars_truncates_at_char_boundary_when_utf8_input_is_long() {
        let input = "你好🙂abc";
        assert_eq!(truncate_str_chars(input, 3), "你好🙂");
    }
}
