use super::DesignVariant;

// ============================================================================
// Separator Style Enum
// ============================================================================

/// Enumeration of all available separator styles.
///
/// Each variant represents a distinct visual approach to separating
/// group headers in the script list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeparatorStyle {
    // ── Line-Based (7 styles) ──────────────────────────────────────────────
    /// Solid horizontal line across the full width
    #[default]
    SolidLine,

    /// Dotted line pattern: · · · · ·
    DottedLine,

    /// Dashed line pattern: ─ ─ ─ ─ ─
    DashedLine,

    /// Double parallel lines: ═══════════
    DoubleLine,

    /// Ultra-thin 1px line (hairline)
    HairlineSingle,

    /// Thick bar (4px height)
    ThickBar,

    /// Line that fades at the edges (gradient effect)
    FadeEdges,

    // ── Typographic (5 styles) ─────────────────────────────────────────────
    /// ALL CAPS label with lines: ── RECENT ──
    UppercaseLabel,

    /// Small caps styled label
    SmallCapsLabel,

    /// Italic text style for labels
    ItalicLabel,

    /// Bold heavy label with side marks: ▌RECENT▐
    BoldLabel,

    /// Label with underline decoration below
    UnderlinedLabel,

    // ── Decorative (6 styles) ──────────────────────────────────────────────
    /// Chevron arrows before label: ›› RECENT
    ChevronArrow,

    /// Centered dots around label: ••• RECENT •••
    DotsCenter,

    /// Diamond shapes at ends: ◆─────◆
    DiamondDivider,

    /// Square brackets around label: [ RECENT ]
    BracketWrap,

    /// Arrow pointer before label: ▶ RECENT
    ArrowPointer,

    /// Star decorations: ★ ─── ★
    StarDivider,

    // ── Spacing-Based (4 styles) ───────────────────────────────────────────
    /// Extra large vertical gap (24px)
    LargeGap,

    /// Minimal vertical gap (8px)
    TightGap,

    /// Label indented from left edge
    IndentedLabel,

    /// Label at left with indented content below
    HangingIndent,

    // ── Background (4 styles) ──────────────────────────────────────────────
    /// Subtle filled background behind label
    SubtleFill,

    /// Gradient background that fades at edges
    GradientFade,

    /// Frosted glass panel effect
    FrostedPanel,

    /// Rounded pill/badge containing label
    PillBadge,

    // ── Minimalist (5 styles) ──────────────────────────────────────────────
    /// No visual separator, only vertical spacing
    Invisible,

    /// Single centered dot: •
    SingleDot,

    /// Vertical pipe character: │
    PipeChar,

    /// Colon prefix: : RECENT
    ColonPrefix,

    /// Slash prefix: / RECENT
    SlashPrefix,

    // ── Retro (5 styles) ───────────────────────────────────────────────────
    /// ASCII art box: +--[ LABEL ]--+
    AsciiBox,

    /// Unicode box drawing: ├── LABEL ──┤
    BoxDrawing,

    /// Terminal prompt style: ~/recent $
    TerminalPrompt,

    /// DOS-style double lines: ══[ LABEL ]══
    DosStyle,

    /// Typewriter double-rule effect
    TypewriterRule,

    // ── Modern (5 styles) ──────────────────────────────────────────────────
    /// Animated opacity fade effect
    AnimatedFade,

    /// Blur overlay effect description
    BlurOverlay,

    /// Neon glow effect around text/lines
    NeonGlow,

    /// Glass card with backdrop blur
    GlassCard,

    /// Floating label with elevation shadow
    FloatingLabel,
}

// ============================================================================
// Separator Configuration
// ============================================================================

/// Configuration parameters for rendering a separator.
///
/// Each separator style uses these parameters differently based on its
/// visual approach. Some fields may be ignored by simpler styles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeparatorConfig {
    // ── Dimensions ─────────────────────────────────────────────────────────
    /// Total height of the separator (including padding)
    pub height: f32,

    /// Thickness of line elements (for line-based styles)
    pub line_thickness: f32,

    /// Horizontal padding from container edges
    pub padding_x: f32,

    /// Vertical padding above the separator
    pub padding_top: f32,

    /// Vertical padding below the separator
    pub padding_bottom: f32,

    /// Indent from left edge (for indented styles)
    pub indent: f32,

    // ── Colors (as 0xRRGGBB hex values) ────────────────────────────────────
    /// Primary color for lines and decorations
    pub color_primary: u32,

    /// Secondary/muted color for subtle elements
    pub color_secondary: u32,

    /// Background color (for filled styles)
    pub color_background: u32,

    /// Text color for labels
    pub color_text: u32,

    // ── Typography ─────────────────────────────────────────────────────────
    /// Font size for label text
    pub font_size: f32,

    /// Whether label should be uppercase
    pub uppercase: bool,

    /// Whether label should be bold
    pub bold: bool,

    /// Whether label should be italic
    pub italic: bool,

    /// Letter spacing adjustment (0.0 = normal)
    pub letter_spacing: f32,

    // ── Visual Effects ─────────────────────────────────────────────────────
    /// Corner radius for rounded elements
    pub border_radius: f32,

    /// Opacity (0.0 - 1.0)
    pub opacity: f32,

    /// Shadow blur radius (0.0 = no shadow)
    pub shadow_blur: f32,

    /// Shadow offset Y
    pub shadow_offset_y: f32,

    /// Whether to show decorative elements
    pub show_decorations: bool,

    /// Gap between decorations and label
    pub decoration_gap: f32,
}

impl Default for SeparatorConfig {
    fn default() -> Self {
        Self {
            // Dimensions
            height: 24.0, // Standard section header height
            line_thickness: 1.0,
            padding_x: 16.0,
            padding_top: 8.0,
            padding_bottom: 4.0,
            indent: 0.0,

            // Colors (default dark theme)
            color_primary: 0x464647,    // Border color
            color_secondary: 0x3a3a3a,  // Subtle border
            color_background: 0x2a2a2a, // Selected bg
            color_text: 0x808080,       // Muted text

            // Typography
            font_size: 11.0,
            uppercase: true,
            bold: false,
            italic: false,
            letter_spacing: 1.0, // Slight spacing for caps

            // Visual effects
            border_radius: 0.0,
            opacity: 1.0,
            shadow_blur: 0.0,
            shadow_offset_y: 0.0,
            show_decorations: true,
            decoration_gap: 8.0,
        }
    }
}

// ============================================================================
// SeparatorStyle Implementation
// ============================================================================

