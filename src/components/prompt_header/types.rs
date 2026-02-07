use crate::designs::DesignColors;
use crate::theme::Theme;

/// Path prefix text is clipped/truncated beyond this width to preserve query visibility.
pub const HEADER_PATH_PREFIX_MAX_WIDTH_PX: f32 = 320.0;
/// Reserved action slot width when running in compact density mode.
pub const HEADER_ACTIONS_MIN_WIDTH_COMPACT_PX: f32 = 168.0;
/// Reserved action slot width for the default header density.
pub const HEADER_ACTIONS_MIN_WIDTH_NORMAL_PX: f32 = 200.0;
/// Reserved action slot width for expanded action labels/shortcuts.
pub const HEADER_ACTIONS_MIN_WIDTH_EXPANDED_PX: f32 = 236.0;

/// Horizontal density policy for the right-side actions slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeaderActionsDensity {
    /// Tight layout for narrower prompt widths.
    Compact,
    /// Default layout used by existing prompts.
    #[default]
    Normal,
    /// Wider layout when action labels require more room.
    Expanded,
}

impl HeaderActionsDensity {
    /// Reserved minimum width for actions/search area.
    pub fn reserved_min_width_px(self) -> f32 {
        match self {
            Self::Compact => HEADER_ACTIONS_MIN_WIDTH_COMPACT_PX,
            Self::Normal => HEADER_ACTIONS_MIN_WIDTH_NORMAL_PX,
            Self::Expanded => HEADER_ACTIONS_MIN_WIDTH_EXPANDED_PX,
        }
    }

    /// Width of the inline actions-mode search field.
    pub fn actions_search_width_px(self) -> f32 {
        match self {
            Self::Compact => 116.0,
            Self::Normal => 130.0,
            Self::Expanded => 144.0,
        }
    }
}

/// Pre-computed colors for PromptHeader rendering
///
/// This struct holds the primitive color values needed for header rendering,
/// allowing efficient use in closures without cloning the full theme.
#[derive(Clone, Copy, Debug)]
pub struct PromptHeaderColors {
    /// Main text color (for typed input)
    pub text_primary: u32,
    /// Placeholder/muted text color
    pub text_muted: u32,
    /// Separator and dimmed text color
    pub text_dimmed: u32,
    /// Accent color for logo and buttons
    pub accent: u32,
    /// Background color (usually transparent for header)
    pub background: u32,
    /// Search box background color
    pub search_box_bg: u32,
    /// Border color
    pub border: u32,
    /// Logo icon color (for icons on accent background)
    pub logo_icon: u32,
    /// Hover overlay color with alpha (Format: 0xRRGGBBAA)
    pub hover_overlay: u32,
    /// Primary input text size (header search text)
    pub input_font_size: f32,
    /// Supporting text size (hints, search labels, helper text)
    pub supporting_font_size: f32,
    /// Caption text size (badges and compact key hints)
    pub caption_font_size: f32,
}

impl PromptHeaderColors {
    fn overlay_with_alpha(base_color: u32, alpha: u8) -> u32 {
        ((base_color & 0x00ff_ffff) << 8) | (alpha as u32)
    }

    /// Create PromptHeaderColors from theme reference
    pub fn from_theme(theme: &Theme) -> Self {
        let ui_font_size = theme.get_fonts().ui_size;
        Self {
            text_primary: theme.colors.text.primary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent: theme.colors.accent.selected,
            background: theme.colors.background.main,
            search_box_bg: theme.colors.background.search_box,
            border: theme.colors.ui.border,
            logo_icon: theme.colors.text.on_accent,
            hover_overlay: Self::overlay_with_alpha(theme.colors.accent.selected_subtle, 0x26),
            input_font_size: (ui_font_size + 2.0).max(12.0),
            supporting_font_size: (ui_font_size - 2.0).max(10.0),
            caption_font_size: (ui_font_size - 4.0).max(9.0),
        }
    }

    /// Create PromptHeaderColors from design colors for design system support
    pub fn from_design(colors: &DesignColors) -> Self {
        let typography = crate::designs::DesignTypography::default();
        Self {
            text_primary: colors.text_primary,
            text_muted: colors.text_muted,
            text_dimmed: colors.text_dimmed,
            accent: colors.accent,
            background: colors.background,
            search_box_bg: colors.background_secondary,
            border: colors.border,
            logo_icon: colors.text_on_accent,
            hover_overlay: Self::overlay_with_alpha(colors.background_selected, 0x26),
            input_font_size: typography.font_size_lg,
            supporting_font_size: typography.font_size_sm,
            caption_font_size: typography.font_size_xs,
        }
    }
}

impl Default for PromptHeaderColors {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

/// Configuration for PromptHeader display
#[derive(Clone, Debug)]
pub struct PromptHeaderConfig {
    /// Current input text
    pub filter_text: String,
    /// Placeholder shown when input is empty
    pub placeholder: String,
    /// Optional path prefix displayed before filter (e.g., "/Users/john/")
    pub path_prefix: Option<String>,
    /// Label for the primary button (e.g., "Run", "Select")
    pub primary_button_label: String,
    /// Shortcut hint for primary button (e.g., "↵")
    pub primary_button_shortcut: String,
    /// Whether to show the Actions button
    pub show_actions_button: bool,
    /// Whether the cursor is currently visible (for blinking)
    pub cursor_visible: bool,
    /// When true, show actions search input instead of buttons
    pub actions_mode: bool,
    /// Actions search text (when in actions_mode)
    pub actions_search_text: String,
    /// Whether the input is focused
    pub is_focused: bool,
    /// Whether to show the "Ask AI" hint with Tab badge
    pub show_ask_ai_hint: bool,
    /// Width reservation policy for the right-side actions area.
    pub actions_density: HeaderActionsDensity,
}

impl Default for PromptHeaderConfig {
    fn default() -> Self {
        Self {
            filter_text: String::new(),
            placeholder: "Type to search...".to_string(),
            path_prefix: None,
            primary_button_label: "Run".to_string(),
            primary_button_shortcut: "↵".to_string(),
            show_actions_button: true,
            cursor_visible: true,
            actions_mode: false,
            actions_search_text: String::new(),
            is_focused: true,
            show_ask_ai_hint: false,
            actions_density: HeaderActionsDensity::Normal,
        }
    }
}

impl PromptHeaderConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the filter text
    pub fn filter_text(mut self, text: impl Into<String>) -> Self {
        self.filter_text = text.into();
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the path prefix
    pub fn path_prefix(mut self, prefix: Option<String>) -> Self {
        self.path_prefix = prefix;
        self
    }

    /// Set the primary button label
    pub fn primary_button_label(mut self, label: impl Into<String>) -> Self {
        self.primary_button_label = label.into();
        self
    }

    /// Set the primary button shortcut
    pub fn primary_button_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.primary_button_shortcut = shortcut.into();
        self
    }

    /// Set whether to show the actions button
    pub fn show_actions_button(mut self, show: bool) -> Self {
        self.show_actions_button = show;
        self
    }

    /// Set cursor visibility
    pub fn cursor_visible(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    /// Set actions mode
    pub fn actions_mode(mut self, mode: bool) -> Self {
        self.actions_mode = mode;
        self
    }

    /// Set actions search text
    pub fn actions_search_text(mut self, text: impl Into<String>) -> Self {
        self.actions_search_text = text.into();
        self
    }

    /// Set whether the input is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    /// Set whether to show the "Ask AI" hint with Tab badge
    pub fn show_ask_ai_hint(mut self, show: bool) -> Self {
        self.show_ask_ai_hint = show;
        self
    }

    /// Set the action-slot density policy
    pub fn actions_density(mut self, density: HeaderActionsDensity) -> Self {
        self.actions_density = density;
        self
    }
}
