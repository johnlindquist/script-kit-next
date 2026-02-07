#![allow(unused_imports)]

//! Design System Module
//!
//! This module provides a pluggable design system for the script list UI.
//! Each design variant implements the `DesignRenderer` trait to provide
//! its own visual style while maintaining the same functionality.
//!

pub mod apple_hig;
pub mod brutalist;
pub mod compact;
mod glassmorphism;
pub mod group_header_variations;
pub mod icon_variations;
pub mod material3;
mod minimal;
pub mod neon_cyberpunk;
pub mod paper;
pub mod playful;
pub mod retro_terminal;
pub mod separator_variations;
mod traits;

// Re-export the trait and types
pub use apple_hig::{
    render_apple_hig_header, render_apple_hig_log_panel, render_apple_hig_preview_panel,
    render_apple_hig_window_container, AppleHIGRenderer, ITEM_HEIGHT as APPLE_HIG_ITEM_HEIGHT,
};
pub use brutalist::{
    render_brutalist_header, render_brutalist_list, render_brutalist_log_panel,
    render_brutalist_preview_panel, render_brutalist_window_container, BrutalistColors,
    BrutalistRenderer,
};
pub use compact::{
    render_compact_header, render_compact_log_panel, render_compact_preview_panel,
    render_compact_window_container, CompactListItem, CompactRenderer, COMPACT_ITEM_HEIGHT,
};
pub use glassmorphism::{
    render_glassmorphism_header, render_glassmorphism_log_panel,
    render_glassmorphism_preview_panel, render_glassmorphism_window_container, GlassColors,
    GlassmorphismRenderer,
};
pub use material3::{
    render_material3_header, render_material3_log_panel, render_material3_preview_panel,
    render_material3_window_container, Material3Renderer,
};
pub use minimal::{
    render_minimal_action_button, render_minimal_divider, render_minimal_empty_state,
    render_minimal_header, render_minimal_list, render_minimal_log_panel,
    render_minimal_preview_panel, render_minimal_search_bar, render_minimal_status,
    render_minimal_window_container, MinimalColors, MinimalConstants, MinimalRenderer,
    MinimalWindowConfig, MINIMAL_ITEM_HEIGHT,
};
pub use neon_cyberpunk::{
    render_neon_cyberpunk_header, render_neon_cyberpunk_log_panel,
    render_neon_cyberpunk_preview_panel, render_neon_cyberpunk_window_container,
    NeonCyberpunkRenderer,
};
pub use paper::{
    render_paper_header, render_paper_log_panel, render_paper_preview_panel,
    render_paper_window_container, PaperRenderer,
};
pub use playful::{
    render_playful_header, render_playful_log_panel, render_playful_preview_panel,
    render_playful_window_container, PlayfulColors, PlayfulRenderer,
};
pub use retro_terminal::{RetroTerminalRenderer, TerminalColors, TERMINAL_ITEM_HEIGHT};
pub use traits::{
    AppleHIGDesignTokens, BrutalistDesignTokens, CompactDesignTokens, DefaultDesignTokens,
    DesignColors, DesignSpacing, DesignTokens, DesignTokensBox, DesignTypography, DesignVisual,
    GlassmorphismDesignTokens, Material3DesignTokens, MinimalDesignTokens,
    NeonCyberpunkDesignTokens, PaperDesignTokens, PlayfulDesignTokens, RetroTerminalDesignTokens,
};
pub use traits::{DesignRenderer, DesignRendererBox};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants_count() {
        assert_eq!(DesignVariant::all().len(), 11);
    }

    #[test]
    fn test_keyboard_number_round_trip() {
        for num in 0..=9 {
            let variant = DesignVariant::from_keyboard_number(num);
            assert!(
                variant.is_some(),
                "Keyboard number {} should map to a variant",
                num
            );

            let v = variant.unwrap();
            let shortcut = v.shortcut_number();

            // All variants except Playful should have shortcuts
            if v != DesignVariant::Playful {
                assert!(shortcut.is_some(), "Variant {:?} should have a shortcut", v);
                assert_eq!(
                    shortcut.unwrap(),
                    num,
                    "Round-trip failed for number {}",
                    num
                );
            }
        }
    }

    #[test]
    fn test_playful_has_no_shortcut() {
        assert_eq!(DesignVariant::Playful.shortcut_number(), None);
    }

    #[test]
    fn test_variant_names_not_empty() {
        for variant in DesignVariant::all() {
            assert!(
                !variant.name().is_empty(),
                "Variant {:?} should have a name",
                variant
            );
            assert!(
                !variant.description().is_empty(),
                "Variant {:?} should have a description",
                variant
            );
        }
    }

    #[test]
    fn test_default_variant() {
        assert_eq!(DesignVariant::default(), DesignVariant::Default);
    }

    #[test]
    fn test_uses_default_renderer() {
        // Minimal and RetroTerminal now have custom renderers
        assert!(
            !uses_default_renderer(DesignVariant::Minimal),
            "Minimal should NOT use default renderer"
        );
        assert!(
            !uses_default_renderer(DesignVariant::RetroTerminal),
            "RetroTerminal should NOT use default renderer"
        );

        // Default still uses default renderer
        assert!(
            uses_default_renderer(DesignVariant::Default),
            "Default should use default renderer"
        );

        // Other variants still use default renderer (until implemented)
        assert!(uses_default_renderer(DesignVariant::Brutalist));
        assert!(uses_default_renderer(DesignVariant::NeonCyberpunk));
    }

    #[test]
    fn test_get_item_height() {
        // Minimal uses taller items (64px)
        assert_eq!(get_item_height(DesignVariant::Minimal), MINIMAL_ITEM_HEIGHT);
        assert_eq!(get_item_height(DesignVariant::Minimal), 64.0);

        // RetroTerminal uses denser items (28px)
        assert_eq!(
            get_item_height(DesignVariant::RetroTerminal),
            TERMINAL_ITEM_HEIGHT
        );
        assert_eq!(get_item_height(DesignVariant::RetroTerminal), 28.0);

        // Compact uses the smallest items (24px)
        assert_eq!(get_item_height(DesignVariant::Compact), COMPACT_ITEM_HEIGHT);
        assert_eq!(get_item_height(DesignVariant::Compact), 24.0);

        // Default and others use standard height (40px - from design tokens)
        // Note: This differs from LIST_ITEM_HEIGHT (48.0) which is used for actual rendering
        assert_eq!(get_item_height(DesignVariant::Default), 40.0);
        assert_eq!(get_item_height(DesignVariant::Brutalist), 40.0);
    }

    #[test]
    fn test_design_variant_dispatch_coverage() {
        // Ensure all variants are covered by the dispatch logic
        // This test verifies the match arms in render_design_item cover all cases
        for variant in DesignVariant::all() {
            let uses_default = uses_default_renderer(*variant);
            let height = get_item_height(*variant);

            // All variants should have a defined height
            assert!(
                height > 0.0,
                "Variant {:?} should have positive item height",
                variant
            );

            // Minimal and RetroTerminal should use custom renderers
            if *variant == DesignVariant::Minimal || *variant == DesignVariant::RetroTerminal {
                assert!(
                    !uses_default,
                    "Variant {:?} should use custom renderer",
                    variant
                );
            }
        }
    }

    #[test]
    fn test_design_keyboard_coverage() {
        // Verify all keyboard shortcuts 1-0 are mapped
        let mut mapped_variants = Vec::new();
        for num in 0..=9 {
            if let Some(variant) = DesignVariant::from_keyboard_number(num) {
                mapped_variants.push(variant);
            }
        }
        // Should have 10 mapped variants (Cmd+1 through Cmd+0)
        assert_eq!(
            mapped_variants.len(),
            10,
            "Expected 10 keyboard-mapped variants"
        );

        // All mapped variants should be unique
        let mut unique = mapped_variants.clone();
        unique.sort_by_key(|v| *v as u8);
        unique.dedup_by_key(|v| *v as u8);
        assert_eq!(unique.len(), 10, "All keyboard mappings should be unique");
    }

    #[test]
    fn test_design_cycling() {
        // Test that next() cycles through all designs
        let all = DesignVariant::all();
        let mut current = DesignVariant::Default;

        // Cycle through all designs
        for (i, expected) in all.iter().enumerate() {
            assert_eq!(
                current, *expected,
                "Cycle iteration {} should be {:?}",
                i, expected
            );
            current = current.next();
        }

        // After cycling through all, we should be back at Default
        assert_eq!(
            current,
            DesignVariant::Default,
            "Should cycle back to Default"
        );
    }

    #[test]
    fn test_design_prev() {
        // Test that prev() goes backwards
        let current = DesignVariant::Default;
        let prev = current.prev();

        // Default.prev() should be Playful (last in list)
        assert_eq!(prev, DesignVariant::Playful);

        // And prev of that should be Compact
        assert_eq!(prev.prev(), DesignVariant::Compact);
    }

    // =========================================================================
    // DesignTokens Tests
    // =========================================================================

    #[test]
    fn test_get_tokens_returns_correct_variant() {
        // Verify get_tokens returns tokens with matching variant
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            assert_eq!(
                tokens.variant(),
                *variant,
                "get_tokens({:?}) returned tokens for {:?}",
                variant,
                tokens.variant()
            );
        }
    }

    #[test]
    fn test_get_tokens_item_height_matches() {
        // Verify token item_height matches get_item_height function
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            let fn_height = get_item_height(*variant);
            let token_height = tokens.item_height();

            assert_eq!(
                fn_height, token_height,
                "Item height mismatch for {:?}: get_item_height={}, tokens.item_height={}",
                variant, fn_height, token_height
            );
        }
    }

    #[test]
    fn test_design_colors_defaults() {
        let colors = DesignColors::default();

        // Verify expected defaults
        assert_eq!(colors.background, 0x1e1e1e);
        assert_eq!(colors.text_primary, 0xffffff);
        assert_eq!(colors.accent, 0xfbbf24);
        assert_eq!(colors.border, 0x464647);
    }

    #[test]
    fn test_design_spacing_defaults() {
        let spacing = DesignSpacing::default();

        // Verify expected defaults
        assert_eq!(spacing.padding_xs, 4.0);
        assert_eq!(spacing.padding_md, 12.0);
        assert_eq!(spacing.gap_md, 8.0);
        assert_eq!(spacing.item_padding_x, 16.0);
    }

    #[test]
    fn test_design_typography_defaults() {
        let typography = DesignTypography::default();

        // Verify expected defaults
        assert_eq!(typography.font_family, ".AppleSystemUIFont");
        assert_eq!(typography.font_family_mono, "Menlo");
        assert_eq!(typography.font_size_md, 14.0);
    }

    #[test]
    fn test_design_visual_defaults() {
        let visual = DesignVisual::default();

        // Verify expected defaults
        assert_eq!(visual.radius_sm, 4.0);
        assert_eq!(visual.radius_md, 8.0);
        assert_eq!(visual.shadow_opacity, 0.25);
        assert_eq!(visual.border_thin, 1.0);
    }

    #[test]
    fn test_design_tokens_are_copy() {
        // Verify all token structs are Copy (needed for closure efficiency)
        fn assert_copy<T: Copy>() {}

        assert_copy::<DesignColors>();
        assert_copy::<DesignSpacing>();
        assert_copy::<DesignTypography>();
        assert_copy::<DesignVisual>();
    }

    #[test]
    fn test_minimal_tokens_distinctive() {
        let tokens = MinimalDesignTokens;

        // Minimal should have taller items and more generous padding
        assert_eq!(tokens.item_height(), 64.0);
        assert_eq!(tokens.spacing().item_padding_x, 80.0);
        assert_eq!(tokens.visual().radius_md, 0.0); // No borders
    }

    #[test]
    fn test_retro_terminal_tokens_distinctive() {
        let tokens = RetroTerminalDesignTokens;

        // Terminal should have dense items and phosphor green colors
        assert_eq!(tokens.item_height(), 28.0);
        assert_eq!(tokens.colors().text_primary, 0x00ff00); // Phosphor green
        assert_eq!(tokens.colors().background, 0x000000); // Pure black
        assert_eq!(tokens.typography().font_family, "Menlo");
    }

    #[test]
    fn test_compact_tokens_distinctive() {
        let tokens = CompactDesignTokens;

        // Compact should have smallest items
        assert_eq!(tokens.item_height(), 24.0);
        assert!(tokens.spacing().padding_md < DesignSpacing::default().padding_md);
    }

    #[test]
    fn test_all_variants_have_positive_item_height() {
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            assert!(
                tokens.item_height() > 0.0,
                "Variant {:?} has non-positive item height",
                variant
            );
        }
    }

    #[test]
    fn test_all_variants_have_valid_colors() {
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            let colors = tokens.colors();

            // Background should be different from text (for contrast)
            assert_ne!(
                colors.background, colors.text_primary,
                "Variant {:?} has no contrast between bg and text",
                variant
            );
        }
    }

    // =========================================================================
    // Auto-description tests
    // =========================================================================

    use crate::metadata_parser::TypedMetadata;
    use crate::scripts::{MatchIndices, Script, ScriptMatch, Scriptlet, ScriptletMatch};
    use std::path::PathBuf;
    use std::sync::Arc;

    fn make_test_script(name: &str) -> Script {
        Script {
            name: name.to_string(),
            path: PathBuf::from(format!(
                "/test/{}.ts",
                name.to_lowercase().replace(' ', "-")
            )),
            extension: "ts".to_string(),
            ..Default::default()
        }
    }

    fn make_script_search_result(script: Script) -> SearchResult {
        SearchResult::Script(ScriptMatch {
            filename: format!("{}.ts", script.name.to_lowercase().replace(' ', "-")),
            script: Arc::new(script),
            score: 100,
            match_indices: MatchIndices::default(),
        })
    }

    fn make_scriptlet_search_result(scriptlet: Scriptlet) -> SearchResult {
        SearchResult::Scriptlet(ScriptletMatch {
            scriptlet: Arc::new(scriptlet),
            score: 100,
            display_file_path: None,
            match_indices: MatchIndices::default(),
        })
    }

    #[test]
    fn test_search_accessories_hide_source_hint_during_filtering() {
        let mut script = make_test_script("Clipboard Variables");
        script.kit_name = Some("clipboard".to_string());
        script.shortcut = Some("cmd shift v".to_string());
        let result = make_script_search_result(script);

        let accessories = resolve_search_accessories(&result, "clip");
        assert!(
            accessories.type_tag.is_some(),
            "type label should stay visible"
        );
        assert_eq!(
            accessories.source_hint, None,
            "source/category metadata should be hidden during filtering"
        );
    }

    #[test]
    fn test_resolve_tool_badge_hidden_during_filtering_for_scriptlets() {
        let scriptlet = Scriptlet {
            name: "Paste Rich Link".to_string(),
            description: Some("Paste as markdown link".to_string()),
            code: "https://example.com".to_string(),
            tool: "paste".to_string(),
            shortcut: None,
            keyword: Some("!mdlink".to_string()),
            group: Some("Clipboard Transformations".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        let result = make_scriptlet_search_result(scriptlet);

        assert_eq!(resolve_tool_badge(&result, true), None);
    }

    #[test]
    fn test_resolve_tool_badge_kept_when_not_filtering_for_scriptlets() {
        let scriptlet = Scriptlet {
            name: "Paste Rich Link".to_string(),
            description: Some("Paste as markdown link".to_string()),
            code: "https://example.com".to_string(),
            tool: "paste".to_string(),
            shortcut: None,
            keyword: Some("!mdlink".to_string()),
            group: Some("Clipboard Transformations".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        let result = make_scriptlet_search_result(scriptlet);

        assert_eq!(
            resolve_tool_badge(&result, false),
            Some("paste".to_string())
        );
    }

    #[test]
    fn test_auto_description_preserves_explicit() {
        let mut s = make_test_script("My Script");
        s.description = Some("Explicit description".to_string());
        assert_eq!(
            auto_description_for_script(&s),
            Some("Explicit description".to_string())
        );
    }

    #[test]
    fn test_auto_description_cron() {
        let mut s = make_test_script("Daily Backup");
        s.typed_metadata = Some(TypedMetadata {
            cron: Some("0 0 * * *".to_string()),
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Cron: 0 0 * * *".to_string())
        );
    }

    #[test]
    fn test_auto_description_schedule_over_cron() {
        let mut s = make_test_script("Scheduled Task");
        s.typed_metadata = Some(TypedMetadata {
            schedule: Some("every weekday at 9am".to_string()),
            cron: Some("0 9 * * 1-5".to_string()),
            ..Default::default()
        });
        // schedule takes priority over cron
        assert_eq!(
            auto_description_for_script(&s),
            Some("Scheduled: every weekday at 9am".to_string())
        );
    }

    #[test]
    fn test_auto_description_watch() {
        let mut s = make_test_script("Config Watcher");
        s.typed_metadata = Some(TypedMetadata {
            watch: vec!["~/.config/**".to_string()],
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Watches: ~/.config/**".to_string())
        );
    }

    #[test]
    fn test_auto_description_watch_truncates_long_pattern() {
        let mut s = make_test_script("Long Watcher");
        let long_pattern =
            "/very/long/path/to/some/deeply/nested/directory/with/many/levels/**/*.json"
                .to_string();
        s.typed_metadata = Some(TypedMetadata {
            watch: vec![long_pattern],
            ..Default::default()
        });
        let desc = auto_description_for_script(&s).unwrap();
        assert!(desc.starts_with("Watches: "));
        assert!(desc.ends_with("..."));
    }

    #[test]
    fn test_auto_description_background() {
        let mut s = make_test_script("Bg Task");
        s.typed_metadata = Some(TypedMetadata {
            background: true,
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("Background process".to_string())
        );
    }

    #[test]
    fn test_auto_description_system() {
        let mut s = make_test_script("Sys Handler");
        s.typed_metadata = Some(TypedMetadata {
            system: true,
            ..Default::default()
        });
        assert_eq!(
            auto_description_for_script(&s),
            Some("System event handler".to_string())
        );
    }

    #[test]
    fn test_auto_description_filename_fallback() {
        // Script name differs from filename
        let s = make_test_script("My Script");
        // Path is /test/my-script.ts, filename is "my-script.ts", name is "My Script"
        let desc = auto_description_for_script(&s);
        assert_eq!(desc, Some("my-script.ts".to_string()));
    }

    #[test]
    fn test_auto_description_no_filename_when_same_as_name() {
        let mut s = make_test_script("exact");
        s.path = PathBuf::from("/test/exact");
        s.name = "exact".to_string();
        // filename == name → falls through to language label (extension is "ts")
        assert_eq!(
            auto_description_for_script(&s),
            Some("TypeScript".to_string())
        );
    }

    // =========================================================================
    // Grouped view hint tests
    // =========================================================================

    #[test]
    fn test_hint_shortcut_shows_alias() {
        let mut s = make_test_script("Git Commit");
        s.shortcut = Some("opt g".to_string());
        s.alias = Some("gc".to_string());
        assert_eq!(grouped_view_hint_for_script(&s), Some("/gc".to_string()));
    }

    #[test]
    fn test_hint_shortcut_falls_back_to_tags() {
        let mut s = make_test_script("Git Commit");
        s.shortcut = Some("opt g".to_string());
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["git".to_string(), "dev".to_string()],
            ..Default::default()
        });
        // No alias, so falls back to tags
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("git · dev".to_string())
        );
    }

    #[test]
    fn test_hint_alias_badge_shows_tags() {
        let mut s = make_test_script("Git Commit");
        s.alias = Some("gc".to_string());
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["git".to_string()],
            ..Default::default()
        });
        // Alias is badge, tags shown as hint
        assert_eq!(grouped_view_hint_for_script(&s), Some("git".to_string()));
    }

    #[test]
    fn test_hint_alias_badge_falls_back_to_kit() {
        let mut s = make_test_script("Capture Window");
        s.alias = Some("cw".to_string());
        s.kit_name = Some("cleanshot".to_string());
        // Alias is badge, no tags, so falls back to kit name
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("cleanshot".to_string())
        );
    }

    #[test]
    fn test_hint_no_badge_shows_tags() {
        let mut s = make_test_script("Notes");
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["productivity".to_string(), "notes".to_string()],
            ..Default::default()
        });
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("productivity · notes".to_string())
        );
    }

    #[test]
    fn test_hint_no_badge_falls_back_to_kit() {
        let mut s = make_test_script("Annotate");
        s.kit_name = Some("cleanshot".to_string());
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("cleanshot".to_string())
        );
    }

    #[test]
    fn test_hint_main_kit_not_shown() {
        let mut s = make_test_script("Notes");
        s.kit_name = Some("main".to_string());
        // "main" kit should not produce a hint
        assert_eq!(grouped_view_hint_for_script(&s), None);
    }

    #[test]
    fn test_scriptlet_hint_group_shown() {
        use crate::scripts::Scriptlet;
        let sl = Scriptlet {
            name: "Open GitHub".to_string(),
            description: None,
            code: "open https://github.com".to_string(),
            tool: "open".to_string(),
            shortcut: None,
            keyword: None,
            group: Some("Development".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        assert_eq!(
            grouped_view_hint_for_scriptlet(&sl),
            Some("Development".to_string())
        );
    }

    #[test]
    fn test_scriptlet_hint_main_group_hidden() {
        use crate::scripts::Scriptlet;
        let sl = Scriptlet {
            name: "Hello".to_string(),
            description: None,
            code: "echo hello".to_string(),
            tool: "bash".to_string(),
            shortcut: None,
            keyword: None,
            group: Some("main".to_string()),
            file_path: None,
            command: None,
            alias: None,
        };
        assert_eq!(grouped_view_hint_for_scriptlet(&sl), None);
    }

    // =========================================================================
    // Enter text hint tests
    // =========================================================================

    #[test]
    fn test_hint_enter_text_shown_as_fallback() {
        let mut s = make_test_script("Deploy");
        s.kit_name = Some("main".to_string());
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Deploy Now".to_string()),
            ..Default::default()
        });
        // No tags, main kit → falls back to enter text
        assert_eq!(
            grouped_view_hint_for_script(&s),
            Some("→ Deploy Now".to_string())
        );
    }

    #[test]
    fn test_hint_enter_text_not_shown_for_generic_run() {
        let mut s = make_test_script("Basic");
        s.kit_name = Some("main".to_string());
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Run".to_string()),
            ..Default::default()
        });
        // "Run" is generic, should not show
        assert_eq!(grouped_view_hint_for_script(&s), None);
    }

    #[test]
    fn test_hint_tags_preferred_over_enter_text() {
        let mut s = make_test_script("Deploy");
        s.typed_metadata = Some(TypedMetadata {
            enter: Some("Deploy Now".to_string()),
            tags: vec!["devops".to_string()],
            ..Default::default()
        });
        // Tags take priority over enter text
        assert_eq!(grouped_view_hint_for_script(&s), Some("devops".to_string()));
    }

    // =========================================================================
    // Code preview tests
    // =========================================================================

    fn make_test_scriptlet(name: &str, code: &str, tool: &str) -> crate::scripts::Scriptlet {
        crate::scripts::Scriptlet {
            name: name.to_string(),
            description: None,
            code: code.to_string(),
            tool: tool.to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }
    }

    #[test]
    fn test_code_preview_shows_first_line() {
        let sl = make_test_scriptlet("Hello", "echo hello world", "bash");
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("echo hello world".to_string())
        );
    }

    #[test]
    fn test_code_preview_skips_comments() {
        let sl = make_test_scriptlet(
            "Script",
            "#!/bin/bash\n# This is a comment\n// Another comment\nls -la",
            "bash",
        );
        assert_eq!(code_preview_for_scriptlet(&sl), Some("ls -la".to_string()));
    }

    #[test]
    fn test_code_preview_empty_code() {
        let sl = make_test_scriptlet("Empty", "", "bash");
        assert_eq!(code_preview_for_scriptlet(&sl), None);
    }

    #[test]
    fn test_code_preview_only_comments() {
        let sl = make_test_scriptlet("Comments", "# comment\n// another\n/* block */", "bash");
        assert_eq!(code_preview_for_scriptlet(&sl), None);
    }

    #[test]
    fn test_code_preview_truncates_long_lines() {
        let long_code =
            "const result = await fetchDataFromRemoteServerWithComplexAuthenticationAndRetryLogic(url, options)";
        let sl = make_test_scriptlet("Long", long_code, "ts");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        assert!(preview.ends_with("..."));
        assert!(preview.chars().count() <= 60);
    }

    #[test]
    fn test_code_preview_paste_shows_content() {
        let sl = make_test_scriptlet("Sig", "Best regards,\nJohn", "paste");
        // Short first line (< 20 chars) appends second line with arrow
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("Best regards, → John".to_string())
        );
    }

    #[test]
    fn test_code_preview_open_shows_url() {
        let sl = make_test_scriptlet("GitHub", "https://github.com", "open");
        assert_eq!(
            code_preview_for_scriptlet(&sl),
            Some("https://github.com".to_string())
        );
    }

    // =========================================================================
    // Match reason detection tests
    // =========================================================================

    #[test]
    fn test_match_reason_name_match_returns_none() {
        let s = make_test_script("Notes");
        // Query matches name → no reason indicator needed
        assert_eq!(detect_match_reason_for_script(&s, "notes"), None);
    }

    #[test]
    fn test_match_reason_short_query_returns_none() {
        let s = make_test_script("Notes");
        // Single char query → skip
        assert_eq!(detect_match_reason_for_script(&s, "n"), None);
    }

    #[test]
    fn test_match_reason_tag_match() {
        let mut s = make_test_script("Daily Backup");
        s.typed_metadata = Some(TypedMetadata {
            tags: vec!["productivity".to_string()],
            ..Default::default()
        });
        assert_eq!(
            detect_match_reason_for_script(&s, "productivity"),
            Some("tag: productivity".to_string())
        );
    }

    #[test]
    fn test_match_reason_author_match() {
        let mut s = make_test_script("My Tool");
        s.typed_metadata = Some(TypedMetadata {
            author: Some("John Lindquist".to_string()),
            ..Default::default()
        });
        assert_eq!(
            detect_match_reason_for_script(&s, "john"),
            Some("by John Lindquist".to_string())
        );
    }

    #[test]
    fn test_match_reason_shortcut_match() {
        let mut s = make_test_script("Quick Notes");
        s.shortcut = Some("opt n".to_string());
        assert_eq!(
            detect_match_reason_for_script(&s, "opt n"),
            Some("shortcut".to_string())
        );
    }

    #[test]
    fn test_match_reason_kit_match() {
        let mut s = make_test_script("Capture");
        s.kit_name = Some("cleanshot".to_string());
        assert_eq!(
            detect_match_reason_for_script(&s, "cleanshot"),
            Some("kit: cleanshot".to_string())
        );
    }

    #[test]
    fn test_match_reason_main_kit_not_shown() {
        let mut s = make_test_script("Capture");
        s.kit_name = Some("main".to_string());
        assert_eq!(detect_match_reason_for_script(&s, "main"), None);
    }

    #[test]
    fn test_scriptlet_match_reason_keyword() {
        let mut sl = make_test_scriptlet("Signature", "Best regards", "paste");
        sl.keyword = Some("!sig".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "!sig"),
            Some("keyword: !sig".to_string())
        );
    }

    #[test]
    fn test_scriptlet_match_reason_code_match() {
        let sl = make_test_scriptlet("Open URL", "https://github.com", "open");
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "github"),
            Some("code match".to_string())
        );
    }

    #[test]
    fn test_scriptlet_match_reason_name_match_returns_none() {
        let sl = make_test_scriptlet("Open GitHub", "https://github.com", "open");
        // Query matches name → no reason indicator
        assert_eq!(detect_match_reason_for_scriptlet(&sl, "github"), None);
    }

    #[test]
    fn test_scriptlet_match_reason_group() {
        let mut sl = make_test_scriptlet("Hello", "echo hello", "bash");
        sl.group = Some("Development".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "development"),
            Some("group: Development".to_string())
        );
    }

    // =========================================================================
    // Excerpt helper tests
    // =========================================================================

    #[test]
    fn test_excerpt_short_text_no_truncation() {
        let result = excerpt_around_match("short text", "short", 40);
        assert_eq!(result, "short text");
    }

    #[test]
    fn test_excerpt_long_text_shows_ellipsis() {
        let text = "This is a very long description that talks about managing clipboard history and other features";
        let result = excerpt_around_match(text, "clipboard", 30);
        assert!(
            result.contains("clipboard"),
            "Excerpt should contain the matched term"
        );
        assert!(
            result.contains("..."),
            "Long text should be truncated with ellipsis"
        );
    }

    #[test]
    fn test_excerpt_match_at_start() {
        let text = "clipboard manager that helps you organize your copy history across all apps";
        let result = excerpt_around_match(text, "clipboard", 30);
        // Match is at the start, so excerpt starts from beginning
        assert!(result.starts_with("clipboard"));
    }

    #[test]
    fn test_excerpt_match_at_end() {
        let text = "A tool that helps you organize and manage your clipboard";
        let result = excerpt_around_match(text, "clipboard", 30);
        assert!(result.contains("clipboard"));
    }

    // =========================================================================
    // Script match reason: description excerpt tests
    // =========================================================================

    #[test]
    fn test_match_reason_description_excerpt() {
        let mut s = make_test_script("My Tool");
        s.description = Some("Manages clipboard history across all your devices".to_string());
        let reason = detect_match_reason_for_script(&s, "clipboard");
        assert!(
            reason.is_some(),
            "Description match should produce a reason"
        );
        let reason = reason.unwrap();
        assert!(
            reason.starts_with("desc: "),
            "Should start with 'desc: ', got: {}",
            reason
        );
        assert!(
            reason.contains("clipboard"),
            "Excerpt should contain the match term"
        );
    }

    #[test]
    fn test_match_reason_description_not_shown_when_name_matches() {
        let mut s = make_test_script("Clipboard Manager");
        s.description = Some("Manages clipboard history".to_string());
        // Name matches "clipboard" so no reason needed
        assert_eq!(detect_match_reason_for_script(&s, "clipboard"), None);
    }

    // =========================================================================
    // Script match reason: alias tests
    // =========================================================================

    #[test]
    fn test_match_reason_alias_match() {
        let mut s = make_test_script("Git Commit Helper");
        s.alias = Some("gc".to_string());
        let reason = detect_match_reason_for_script(&s, "gc");
        assert_eq!(reason, Some("alias: /gc".to_string()));
    }

    #[test]
    fn test_match_reason_alias_not_shown_when_name_matches() {
        let mut s = make_test_script("GC Cleaner");
        s.alias = Some("gc".to_string());
        // Name contains "GC" so no reason needed
        assert_eq!(detect_match_reason_for_script(&s, "gc"), None);
    }

    // =========================================================================
    // Script match reason: path match tests
    // =========================================================================

    #[test]
    fn test_match_reason_path_match() {
        let mut s = make_test_script("My Tool");
        s.path = std::path::PathBuf::from("/Users/john/.kenv/scripts/secret-helper.ts");
        let reason = detect_match_reason_for_script(&s, "secret-helper");
        assert_eq!(reason, Some("path match".to_string()));
    }

    // =========================================================================
    // Scriptlet match reason: alias tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_alias() {
        let mut sl = make_test_scriptlet("Quick Paste", "Best regards", "paste");
        sl.alias = Some("qp".to_string());
        assert_eq!(
            detect_match_reason_for_scriptlet(&sl, "qp"),
            Some("alias: /qp".to_string())
        );
    }

    // =========================================================================
    // Scriptlet match reason: tool type tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_tool_type() {
        let sl = make_test_scriptlet("Run Server", "npm start", "bash");
        let reason = detect_match_reason_for_scriptlet(&sl, "bash");
        assert!(reason.is_some(), "Tool type match should produce a reason");
        let reason = reason.unwrap();
        assert!(
            reason.starts_with("tool: "),
            "Should start with 'tool: ', got: {}",
            reason
        );
    }

    #[test]
    fn test_scriptlet_match_reason_tool_not_shown_when_name_matches() {
        let sl = make_test_scriptlet("Bash Helper", "echo hi", "bash");
        // Name matches "bash" so no reason needed
        assert_eq!(detect_match_reason_for_scriptlet(&sl, "bash"), None);
    }

    // =========================================================================
    // Scriptlet match reason: description excerpt tests
    // =========================================================================

    #[test]
    fn test_scriptlet_match_reason_description_excerpt() {
        let mut sl = make_test_scriptlet("Quick Action", "echo done", "bash");
        sl.description = Some("Automates the deployment pipeline for staging".to_string());
        let reason = detect_match_reason_for_scriptlet(&sl, "deployment");
        assert!(reason.is_some());
        let reason = reason.unwrap();
        assert!(reason.starts_with("desc: "));
        assert!(reason.contains("deployment"));
    }

    // =========================================================================
    // Enhanced code preview tests (multi-line)
    // =========================================================================

    #[test]
    fn test_code_preview_short_first_line_appends_second() {
        let sl = make_test_scriptlet("Deploy", "cd ~/projects\nnpm run build", "bash");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        assert!(
            preview.contains("\u{2192}"),
            "Short first line should append second line with arrow: {}",
            preview
        );
        assert!(preview.contains("cd ~/projects"));
        assert!(preview.contains("npm run build"));
    }

    #[test]
    fn test_code_preview_long_first_line_no_append() {
        let sl = make_test_scriptlet(
            "Long",
            "const result = fetchData()\nconsole.log(result)",
            "ts",
        );
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // First line is > 20 chars, should NOT append second line
        assert!(
            !preview.contains("\u{2192}"),
            "Long first line should not append second: {}",
            preview
        );
    }

    #[test]
    fn test_code_preview_short_first_only_line() {
        let sl = make_test_scriptlet("Short", "ls -la", "bash");
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // Only one line, can't append second
        assert_eq!(preview, "ls -la");
    }

    #[test]
    fn test_code_preview_multi_line_truncates_combined() {
        let sl = make_test_scriptlet(
            "Deploy",
            "cd ~/projects\nexport NODE_ENV=production && npm run build && npm run deploy --target staging",
            "bash",
        );
        let preview = code_preview_for_scriptlet(&sl).unwrap();
        // Combined is long, should truncate
        assert!(preview.contains("\u{2192}"));
        assert!(
            preview.chars().count() <= 63,
            "Combined preview should be truncated, got {} chars: {}",
            preview.chars().count(),
            preview
        );
    }

    // =========================================================================
    // Extension default icon tests
    // =========================================================================

    #[test]
    fn test_extension_default_icon_shell() {
        assert_eq!(extension_default_icon("sh"), "Terminal");
        assert_eq!(extension_default_icon("bash"), "Terminal");
        assert_eq!(extension_default_icon("zsh"), "Terminal");
    }

    #[test]
    fn test_extension_default_icon_applescript() {
        assert_eq!(extension_default_icon("applescript"), "Terminal");
        assert_eq!(extension_default_icon("scpt"), "Terminal");
    }

    #[test]
    fn test_extension_default_icon_default_code() {
        assert_eq!(extension_default_icon("ts"), "Code");
        assert_eq!(extension_default_icon("js"), "Code");
        assert_eq!(extension_default_icon("py"), "Code");
        assert_eq!(extension_default_icon("rb"), "Code");
    }

    // =========================================================================
    // Extension language label tests
    // =========================================================================

    #[test]
    fn test_extension_language_label_typescript() {
        assert_eq!(extension_language_label("ts"), Some("TypeScript"));
        assert_eq!(extension_language_label("tsx"), Some("TypeScript"));
    }

    #[test]
    fn test_extension_language_label_javascript() {
        assert_eq!(extension_language_label("js"), Some("JavaScript"));
        assert_eq!(extension_language_label("mjs"), Some("JavaScript"));
    }

    #[test]
    fn test_extension_language_label_shell() {
        assert_eq!(extension_language_label("sh"), Some("Shell script"));
        assert_eq!(extension_language_label("bash"), Some("Shell script"));
        assert_eq!(extension_language_label("zsh"), Some("Zsh script"));
    }

    #[test]
    fn test_extension_language_label_python() {
        assert_eq!(extension_language_label("py"), Some("Python script"));
    }

    #[test]
    fn test_extension_language_label_unknown() {
        assert_eq!(extension_language_label("xyz"), None);
        assert_eq!(extension_language_label(""), None);
    }

    // =========================================================================
    // Auto-description with language label fallback tests
    // =========================================================================

    #[test]
    fn test_auto_description_language_label_fallback() {
        // Script with same-name filename and no metadata -> should get language label
        let script = crate::scripts::Script {
            name: "my-script".to_string(),
            path: std::path::PathBuf::from("/test/my-script.ts"),
            extension: "ts".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        // Filename "my-script.ts" differs from name "my-script", so filename wins
        assert_eq!(desc, Some("my-script.ts".to_string()));
    }

    #[test]
    fn test_auto_description_language_label_when_filename_matches() {
        // Script where filename equals name -> language label should appear
        // This happens when the name IS the filename (without extension somehow)
        let script = crate::scripts::Script {
            name: "my-script.ts".to_string(),
            path: std::path::PathBuf::from("/test/my-script.ts"),
            extension: "ts".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        // Filename "my-script.ts" == name "my-script.ts", so language label fallback
        assert_eq!(desc, Some("TypeScript".to_string()));
    }

    #[test]
    fn test_auto_description_shell_script_language_label() {
        let script = crate::scripts::Script {
            name: "backup.sh".to_string(),
            path: std::path::PathBuf::from("/test/backup.sh"),
            extension: "sh".to_string(),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        assert_eq!(desc, Some("Shell script".to_string()));
    }

    #[test]
    fn test_auto_description_explicit_description_unchanged() {
        let script = crate::scripts::Script {
            name: "test".to_string(),
            path: std::path::PathBuf::from("/test/test.ts"),
            extension: "ts".to_string(),
            description: Some("My custom description".to_string()),
            ..Default::default()
        };
        let desc = auto_description_for_script(&script);
        assert_eq!(desc, Some("My custom description".to_string()));
    }
}
