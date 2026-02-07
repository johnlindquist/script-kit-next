impl ListItemColors {
    /// Create from theme reference
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let opacity = theme.get_opacity();
        Self {
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent_selected: theme.colors.accent.selected,
            accent_selected_subtle: theme.colors.accent.selected_subtle,
            background: theme.colors.background.main,
            background_selected: theme.colors.accent.selected_subtle,
            selected_opacity: opacity.selected,
            hover_opacity: opacity.hover,
            warning_bg: theme.colors.ui.warning,
            // White text provides good contrast on warning/accent backgrounds in dark themes
            text_on_accent: theme.colors.text.primary,
        }
    }

    /// Create from design colors for GLOBAL theming support
    /// Uses same opacity values as from_theme() for consistent vibrancy-compatible styling
    ///
    /// NOTE: This defaults to dark mode opacity values. For light mode support,
    /// use `from_design_with_dark_mode()` instead.
    pub fn from_design(colors: &crate::designs::DesignColors) -> Self {
        // Default to dark mode
        Self::from_design_with_dark_mode(colors, true)
    }

    /// Create from design colors with explicit dark/light mode
    ///
    /// Light mode needs higher opacity values because low opacity on light backgrounds
    /// (e.g., white at 7-12%) is too subtle to be visible. Dark mode uses lower opacity
    /// because white overlays are more visible on dark backgrounds.
    ///
    /// # Arguments
    /// * `colors` - Design colors to use
    /// * `is_dark` - True for dark mode (lower opacity), false for light mode (higher opacity)
    pub fn from_design_with_dark_mode(
        colors: &crate::designs::DesignColors,
        is_dark: bool,
    ) -> Self {
        // Dark mode: low opacity works well (white at 7-12% visible on dark bg)
        // Light mode: needs higher opacity for visibility (black overlay on light bg)
        // Values aligned with Material Design elevation overlay model (~4-6dp)
        let (selected_opacity, hover_opacity) = if is_dark {
            (0.14, 0.08) // Dark mode: improved selection/hover visibility
        } else {
            (0.20, 0.12) // Light mode: stronger overlay for visibility on vibrancy
        };

        Self {
            text_primary: colors.text_primary,
            text_secondary: colors.text_secondary,
            text_muted: colors.text_muted,
            text_dimmed: colors.text_dimmed,
            accent_selected: colors.accent,
            accent_selected_subtle: colors.background_selected,
            background: colors.background,
            background_selected: colors.background_selected,
            selected_opacity,
            hover_opacity,
            warning_bg: colors.warning,
            text_on_accent: colors.text_on_accent,
        }
    }
}
/// Format a keyboard shortcut string using macOS-native modifier symbols.
///
/// Converts common shortcut formats to native macOS symbols:
/// - "cmd+shift+k" → "⌘⇧K"
/// - "ctrl+c" → "⌃C"
/// - "alt+enter" → "⌥↩"
///
/// If the input already contains macOS symbols (⌘, ⇧, ⌥, ⌃), returns as-is.
pub fn format_shortcut_display(shortcut: &str) -> String {
    // If already contains macOS modifier symbols, return as-is
    if shortcut.contains('⌘')
        || shortcut.contains('⇧')
        || shortcut.contains('⌥')
        || shortcut.contains('⌃')
    {
        return shortcut.to_string();
    }

    // Normalize: replace '+' with space, then split on whitespace.
    // This handles both space-delimited ("opt i", "cmd shift k") and
    // plus-delimited ("cmd+shift+k") shortcut formats from Script Kit metadata.
    let normalized = shortcut.replace('+', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    let mut result = String::new();

    for part in &parts {
        match part.to_lowercase().as_str() {
            "cmd" | "command" | "meta" | "super" => result.push('⌘'),
            "shift" => result.push('⇧'),
            "alt" | "option" | "opt" => result.push('⌥'),
            "ctrl" | "control" => result.push('⌃'),
            "enter" | "return" => result.push('↩'),
            "escape" | "esc" => result.push('⎋'),
            "tab" => result.push('⇥'),
            "space" => result.push('␣'),
            "backspace" | "delete" => result.push('⌫'),
            "up" | "arrowup" => result.push('↑'),
            "down" | "arrowdown" => result.push('↓'),
            "left" | "arrowleft" => result.push('←'),
            "right" | "arrowright" => result.push('→'),
            key => {
                // Uppercase single-character keys, preserve multi-char keys as-is
                if key.len() == 1 {
                    result.push_str(&key.to_uppercase());
                } else {
                    result.push_str(key);
                }
            }
        }
    }

    result
}
/// Search rows keep shortcuts only on the actively focused row to avoid right-side noise.
pub(crate) fn should_show_search_shortcut(
    is_filtering: bool,
    selected: bool,
    hovered: bool,
) -> bool {
    if !is_filtering {
        return true;
    }
    selected || hovered
}
/// Search rows keep descriptions only when they add context for the current focus or match.
pub(crate) fn should_show_search_description(
    selected: bool,
    hovered: bool,
    has_description_match: bool,
) -> bool {
    selected || hovered || has_description_match
}
/// Callback type for hover events on list items.
/// The callback receives the item index and a boolean indicating hover state (true = entered, false = left).
pub type OnHoverCallback = Box<dyn Fn(usize, bool) + 'static>;
/// A reusable list item component for displaying selectable items
///
/// Supports:
/// - Name (required)
/// - Description (optional, shown below name)
/// - Icon (optional, emoji or PNG image displayed left of name)
/// - Shortcut badge (optional, right-aligned)
/// - Selection state with themed colors (full focus styling)
/// - Hover state with subtle visual feedback (separate from selection)
/// - Hover callback for mouse interaction (optional)
/// - Semantic ID for AI-driven targeting (optional)
///
#[derive(IntoElement)]
pub struct ListItem {
    name: SharedString,
    description: Option<String>,
    shortcut: Option<String>,
    icon: Option<IconKind>,
    selected: bool,
    /// Whether this item is being hovered (subtle visual feedback, separate from selected)
    hovered: bool,
    colors: ListItemColors,
    /// Index of this item in the list (needed for hover callback)
    index: Option<usize>,
    /// Optional callback triggered when mouse enters/leaves this item
    on_hover: Option<OnHoverCallback>,
    /// Semantic ID for AI-driven UX targeting. Format: {type}:{index}:{value}
    semantic_id: Option<String>,
    /// Show left accent bar when selected (3px colored bar on left edge)
    show_accent_bar: bool,
    /// Whether to enable instant hover effects (via GPUI .hover() pseudo-class)
    /// When false, the .hover() modifier is not applied, preventing visual feedback
    /// Used to disable hover when user is navigating with keyboard
    enable_hover_effect: bool,
    /// Character indices in the name that match the search query (for fuzzy highlight)
    /// When present, matched characters are rendered with accent color for visual emphasis
    highlight_indices: Option<Vec<usize>>,
    /// Character indices in the description that match the search query (for fuzzy highlight)
    /// When present, matched characters are rendered with accent color for visual emphasis
    description_highlight_indices: Option<Vec<usize>>,
    /// Type tag shown as subtle colored text (e.g., "Script", "Snippet", "App")
    /// Only shown during search mode to help distinguish mixed result types
    type_tag: Option<TypeTag>,
    /// Source/kit name (e.g., "main", "cleanshot") shown as subtle text during search
    source_hint: Option<String>,
    /// Tool/language badge for scriptlets (e.g., "ts", "bash", "paste")
    /// Shown as a subtle monospace badge in the accessories area
    tool_badge: Option<String>,
}
/// Type tag displayed as subtle colored text on list items during search
#[derive(Clone, Debug)]
pub struct TypeTag {
    /// Display label (e.g., "Script", "Snippet", "App")
    pub label: &'static str,
    /// Color for the tag (u32 hex, e.g., 0x3B82F6 for blue)
    pub color: u32,
}
/// Width of the left accent bar for selected items
pub const ACCENT_BAR_WIDTH: f32 = 3.0;
impl ListItem {
    /// Create a new list item with the given name and pre-computed colors
    pub fn new(name: impl Into<SharedString>, colors: ListItemColors) -> Self {
        Self {
            name: name.into(),
            description: None,
            shortcut: None,
            icon: None,
            selected: false,
            hovered: false,
            colors,
            index: None,
            on_hover: None,
            semantic_id: None,
            show_accent_bar: false,
            enable_hover_effect: true, // Default to enabled
            highlight_indices: None,
            description_highlight_indices: None,
            type_tag: None,
            source_hint: None,
            tool_badge: None,
        }
    }

    /// Enable the left accent bar (3px colored bar shown when selected)
    pub fn with_accent_bar(mut self, show: bool) -> Self {
        self.show_accent_bar = show;
        self
    }

    /// Enable or disable instant hover effects (GPUI .hover() pseudo-class)
    /// When disabled, no visual feedback is shown on mouse hover
    /// Used to prevent hover effects during keyboard navigation
    pub fn with_hover_effect(mut self, enable: bool) -> Self {
        self.enable_hover_effect = enable;
        self
    }

    /// Set the index of this item in the list (required for hover callback to work)
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Set a callback to be triggered when mouse enters or leaves this item.
    /// The callback receives (index, is_hovered) where is_hovered is true when entering.
    pub fn on_hover(mut self, callback: OnHoverCallback) -> Self {
        self.on_hover = Some(callback);
        self
    }

    /// Set the semantic ID for AI-driven UX targeting.
    /// Format: {type}:{index}:{value} (e.g., "choice:0:apple")
    pub fn semantic_id(mut self, id: impl Into<String>) -> Self {
        self.semantic_id = Some(id.into());
        self
    }

    /// Set an optional semantic ID (convenience for Option<String>)
    pub fn semantic_id_opt(mut self, id: Option<String>) -> Self {
        self.semantic_id = id;
        self
    }

    /// Set the description text (shown below the name)
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Set an optional description (convenience for Option<String>)
    pub fn description_opt(mut self, d: Option<String>) -> Self {
        self.description = d;
        self
    }

    /// Set the shortcut badge text (shown right-aligned)
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }

    /// Set an optional shortcut (convenience for Option<String>)
    pub fn shortcut_opt(mut self, s: Option<String>) -> Self {
        self.shortcut = s;
        self
    }

    /// Set the icon (emoji) to display on the left side
    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(IconKind::Emoji(i.into()));
        self
    }

    /// Set an optional emoji icon (convenience for Option<String>)
    pub fn icon_opt(mut self, i: Option<String>) -> Self {
        self.icon = i.map(IconKind::Emoji);
        self
    }

    /// Set a pre-decoded RenderImage icon
    pub fn icon_image(mut self, image: Arc<RenderImage>) -> Self {
        self.icon = Some(IconKind::Image(image));
        self
    }

    /// Set an optional pre-decoded image icon
    pub fn icon_image_opt(mut self, image: Option<Arc<RenderImage>>) -> Self {
        self.icon = image.map(IconKind::Image);
        self
    }

    /// Set icon from IconKind enum (for mixed icon types)
    pub fn icon_kind(mut self, kind: IconKind) -> Self {
        self.icon = Some(kind);
        self
    }

    /// Set an optional icon from IconKind
    pub fn icon_kind_opt(mut self, kind: Option<IconKind>) -> Self {
        self.icon = kind;
        self
    }

    /// Set whether this item is selected
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set whether this item is hovered (visual feedback)
    ///
    /// Hovered items show a visible background tint (25% opacity).
    /// This is separate from `selected` which shows full focus styling
    /// (35% opacity background + accent bar).
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Set character indices for fuzzy match highlighting
    /// When set, matched characters in the name are rendered with accent color
    pub fn highlight_indices(mut self, indices: Vec<usize>) -> Self {
        if !indices.is_empty() {
            self.highlight_indices = Some(indices);
        }
        self
    }

    /// Set optional highlight indices (convenience for Option<Vec<usize>>)
    pub fn highlight_indices_opt(mut self, indices: Option<Vec<usize>>) -> Self {
        self.highlight_indices = indices.filter(|v| !v.is_empty());
        self
    }

    /// Set character indices for fuzzy match highlighting in description
    /// When set, matched characters in the description are rendered with accent color
    pub fn description_highlight_indices(mut self, indices: Vec<usize>) -> Self {
        if !indices.is_empty() {
            self.description_highlight_indices = Some(indices);
        }
        self
    }

    /// Set optional description highlight indices (convenience for Option<Vec<usize>>)
    pub fn description_highlight_indices_opt(mut self, indices: Option<Vec<usize>>) -> Self {
        self.description_highlight_indices = indices.filter(|v| !v.is_empty());
        self
    }

    /// Set a type tag to show as subtle colored text (e.g., "Script", "Snippet")
    /// Only used during search mode to distinguish mixed result types
    pub fn type_tag(mut self, tag: TypeTag) -> Self {
        self.type_tag = Some(tag);
        self
    }

    /// Set an optional type tag
    pub fn type_tag_opt(mut self, tag: Option<TypeTag>) -> Self {
        self.type_tag = tag;
        self
    }

    /// Set the source/kit name hint (shown during search to indicate origin)
    pub fn source_hint(mut self, hint: impl Into<String>) -> Self {
        self.source_hint = Some(hint.into());
        self
    }

    /// Set an optional source hint
    pub fn source_hint_opt(mut self, hint: Option<String>) -> Self {
        self.source_hint = hint;
        self
    }

    /// Set the tool/language badge (e.g., "ts", "bash", "paste")
    /// Displayed as a subtle monospace badge for scriptlets
    pub fn tool_badge(mut self, badge: impl Into<String>) -> Self {
        self.tool_badge = Some(badge.into());
        self
    }

    /// Set the tool/language badge from an option
    pub fn tool_badge_opt(mut self, badge: Option<String>) -> Self {
        self.tool_badge = badge;
        self
    }
}
