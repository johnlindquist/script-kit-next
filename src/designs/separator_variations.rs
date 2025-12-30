#![allow(dead_code)]
//! Group Separator Style Variations
//!
//! This module provides 25+ named separator styles for group headers in the main menu list.
//! Each separator defines how to visually separate group sections like "RECENT" and "MAIN".
//!
//! # Visual Guide
//!
//! ## Line-Based Separators
//! ```text
//! SolidLine:       ─────────────────────────────
//! DottedLine:      · · · · · · · · · · · · · · ·
//! DashedLine:      ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
//! DoubleLine:      ═══════════════════════════════
//! HairlineSingle:  ──────────────────────────────  (1px)
//! ThickBar:        ████████████████████████████   (4px)
//! FadeEdges:       ░░░▓▓▓███████████████▓▓▓░░░
//! ```
//!
//! ## Typographic Separators
//! ```text
//! UppercaseLabel:    ── RECENT ──
//! SmallCapsLabel:    ⸺ Recent ⸺
//! ItalicLabel:       — Recent —
//! BoldLabel:         ▌RECENT▐
//! UnderlinedLabel:   RECENT
//!                    ‾‾‾‾‾‾
//! ```
//!
//! ## Decorative Separators
//! ```text
//! ChevronArrow:      ›› RECENT
//! DotsCenter:        ••• RECENT •••
//! DiamondDivider:    ◆─────────◆
//! BracketWrap:       [ RECENT ]
//! ArrowPointer:      ▶ RECENT
//! StarDivider:       ★ ─────── ★
//! ```
//!
//! ## Spacing-Based Separators
//! ```text
//! LargeGap:          (24px vertical space)
//! TightGap:          (8px vertical space)
//! IndentedLabel:         RECENT
//! HangingIndent:     RECENT
//!                        ↳ items below
//! ```
//!
//! ## Background Separators
//! ```text
//! SubtleFill:        ▒▒▒▒▒ RECENT ▒▒▒▒▒  (light bg)
//! GradientFade:      ░▒▓ RECENT ▓▒░      (faded edges)
//! FrostedPanel:      ╔═══ RECENT ═══╗    (glass effect)
//! PillBadge:         ╭──────────╮
//!                    │  RECENT  │
//!                    ╰──────────╯
//! ```
//!
//! ## Minimalist Separators
//! ```text
//! Invisible:         (no visual, just spacing)
//! SingleDot:         •
//! PipeChar:          │
//! ColonPrefix:       : RECENT
//! SlashPrefix:       / RECENT
//! ```
//!
//! ## Retro Separators
//! ```text
//! AsciiBox:          +--[ RECENT ]--+
//! BoxDrawing:        ├── RECENT ────┤
//! TerminalPrompt:    ~/recent $
//! DosStyle:          ══[ RECENT ]══
//! TypewriterRule:    ────────────────
//!                    ═══════════════
//! ```
//!
//! ## Modern Separators
//! ```text
//! AnimatedFade:      (opacity transition)
//! BlurOverlay:       (backdrop blur effect)
//! NeonGlow:          ═══ RECENT ═══  (with glow)
//! GlassCard:         Frosted card with shadow
//! FloatingLabel:     RECENT (elevated with shadow)
//! ```

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

impl SeparatorStyle {
    /// Get all available separator styles.
    pub fn all() -> &'static [SeparatorStyle] {
        &[
            // Line-Based
            SeparatorStyle::SolidLine,
            SeparatorStyle::DottedLine,
            SeparatorStyle::DashedLine,
            SeparatorStyle::DoubleLine,
            SeparatorStyle::HairlineSingle,
            SeparatorStyle::ThickBar,
            SeparatorStyle::FadeEdges,
            // Typographic
            SeparatorStyle::UppercaseLabel,
            SeparatorStyle::SmallCapsLabel,
            SeparatorStyle::ItalicLabel,
            SeparatorStyle::BoldLabel,
            SeparatorStyle::UnderlinedLabel,
            // Decorative
            SeparatorStyle::ChevronArrow,
            SeparatorStyle::DotsCenter,
            SeparatorStyle::DiamondDivider,
            SeparatorStyle::BracketWrap,
            SeparatorStyle::ArrowPointer,
            SeparatorStyle::StarDivider,
            // Spacing-Based
            SeparatorStyle::LargeGap,
            SeparatorStyle::TightGap,
            SeparatorStyle::IndentedLabel,
            SeparatorStyle::HangingIndent,
            // Background
            SeparatorStyle::SubtleFill,
            SeparatorStyle::GradientFade,
            SeparatorStyle::FrostedPanel,
            SeparatorStyle::PillBadge,
            // Minimalist
            SeparatorStyle::Invisible,
            SeparatorStyle::SingleDot,
            SeparatorStyle::PipeChar,
            SeparatorStyle::ColonPrefix,
            SeparatorStyle::SlashPrefix,
            // Retro
            SeparatorStyle::AsciiBox,
            SeparatorStyle::BoxDrawing,
            SeparatorStyle::TerminalPrompt,
            SeparatorStyle::DosStyle,
            SeparatorStyle::TypewriterRule,
            // Modern
            SeparatorStyle::AnimatedFade,
            SeparatorStyle::BlurOverlay,
            SeparatorStyle::NeonGlow,
            SeparatorStyle::GlassCard,
            SeparatorStyle::FloatingLabel,
        ]
    }

    /// Get the count of all separator styles.
    pub fn count() -> usize {
        Self::all().len()
    }

    /// Get the display name for this separator style.
    pub fn name(&self) -> &'static str {
        match self {
            // Line-Based
            SeparatorStyle::SolidLine => "Solid Line",
            SeparatorStyle::DottedLine => "Dotted Line",
            SeparatorStyle::DashedLine => "Dashed Line",
            SeparatorStyle::DoubleLine => "Double Line",
            SeparatorStyle::HairlineSingle => "Hairline Single",
            SeparatorStyle::ThickBar => "Thick Bar",
            SeparatorStyle::FadeEdges => "Fade Edges",
            // Typographic
            SeparatorStyle::UppercaseLabel => "Uppercase Label",
            SeparatorStyle::SmallCapsLabel => "Small Caps Label",
            SeparatorStyle::ItalicLabel => "Italic Label",
            SeparatorStyle::BoldLabel => "Bold Label",
            SeparatorStyle::UnderlinedLabel => "Underlined Label",
            // Decorative
            SeparatorStyle::ChevronArrow => "Chevron Arrow",
            SeparatorStyle::DotsCenter => "Dots Center",
            SeparatorStyle::DiamondDivider => "Diamond Divider",
            SeparatorStyle::BracketWrap => "Bracket Wrap",
            SeparatorStyle::ArrowPointer => "Arrow Pointer",
            SeparatorStyle::StarDivider => "Star Divider",
            // Spacing-Based
            SeparatorStyle::LargeGap => "Large Gap",
            SeparatorStyle::TightGap => "Tight Gap",
            SeparatorStyle::IndentedLabel => "Indented Label",
            SeparatorStyle::HangingIndent => "Hanging Indent",
            // Background
            SeparatorStyle::SubtleFill => "Subtle Fill",
            SeparatorStyle::GradientFade => "Gradient Fade",
            SeparatorStyle::FrostedPanel => "Frosted Panel",
            SeparatorStyle::PillBadge => "Pill Badge",
            // Minimalist
            SeparatorStyle::Invisible => "Invisible",
            SeparatorStyle::SingleDot => "Single Dot",
            SeparatorStyle::PipeChar => "Pipe Character",
            SeparatorStyle::ColonPrefix => "Colon Prefix",
            SeparatorStyle::SlashPrefix => "Slash Prefix",
            // Retro
            SeparatorStyle::AsciiBox => "ASCII Box",
            SeparatorStyle::BoxDrawing => "Box Drawing",
            SeparatorStyle::TerminalPrompt => "Terminal Prompt",
            SeparatorStyle::DosStyle => "DOS Style",
            SeparatorStyle::TypewriterRule => "Typewriter Rule",
            // Modern
            SeparatorStyle::AnimatedFade => "Animated Fade",
            SeparatorStyle::BlurOverlay => "Blur Overlay",
            SeparatorStyle::NeonGlow => "Neon Glow",
            SeparatorStyle::GlassCard => "Glass Card",
            SeparatorStyle::FloatingLabel => "Floating Label",
        }
    }

    /// Get a description of the visual appearance for this separator style.
    pub fn description(&self) -> &'static str {
        match self {
            // Line-Based
            SeparatorStyle::SolidLine => "A clean, solid horizontal line spanning the full width",
            SeparatorStyle::DottedLine => {
                "A series of evenly spaced dots forming a horizontal line"
            }
            SeparatorStyle::DashedLine => "A line made of short dashes with gaps between them",
            SeparatorStyle::DoubleLine => {
                "Two parallel horizontal lines creating a stronger visual break"
            }
            SeparatorStyle::HairlineSingle => "An ultra-thin 1-pixel line for subtle separation",
            SeparatorStyle::ThickBar => "A bold 4-pixel thick bar for prominent division",
            SeparatorStyle::FadeEdges => "A line that gradually fades to transparent at both edges",

            // Typographic
            SeparatorStyle::UppercaseLabel => {
                "ALL CAPS label with horizontal lines extending from both sides"
            }
            SeparatorStyle::SmallCapsLabel => {
                "Label styled in small caps with subtle side decorations"
            }
            SeparatorStyle::ItalicLabel => "Label rendered in italics with em-dash decorations",
            SeparatorStyle::BoldLabel => "Heavy bold label with vertical bar accents",
            SeparatorStyle::UnderlinedLabel => {
                "Label with an underline decoration directly beneath the text"
            }

            // Decorative
            SeparatorStyle::ChevronArrow => "Double chevron arrows pointing right before the label",
            SeparatorStyle::DotsCenter => "Label surrounded by bullet point decorations",
            SeparatorStyle::DiamondDivider => "Diamond shapes at each end of a horizontal line",
            SeparatorStyle::BracketWrap => "Label enclosed in square brackets",
            SeparatorStyle::ArrowPointer => "Filled arrow/triangle pointing right before the label",
            SeparatorStyle::StarDivider => "Star symbols at each end of the separator line",

            // Spacing-Based
            SeparatorStyle::LargeGap => {
                "Extra vertical whitespace (24px) for strong visual grouping"
            }
            SeparatorStyle::TightGap => "Minimal vertical spacing (8px) for compact layouts",
            SeparatorStyle::IndentedLabel => {
                "Label offset from the left edge with increased left margin"
            }
            SeparatorStyle::HangingIndent => {
                "Label flush left with subsequent content indented below"
            }

            // Background
            SeparatorStyle::SubtleFill => {
                "Full-width background fill in a muted color behind the label"
            }
            SeparatorStyle::GradientFade => {
                "Background that fades from solid center to transparent edges"
            }
            SeparatorStyle::FrostedPanel => "Frosted glass effect panel containing the label",
            SeparatorStyle::PillBadge => "Label inside a rounded pill/capsule shape",

            // Minimalist
            SeparatorStyle::Invisible => "No visible separator, only standard vertical spacing",
            SeparatorStyle::SingleDot => "A single centered bullet point as a minimal divider",
            SeparatorStyle::PipeChar => "A vertical pipe character as a minimal marker",
            SeparatorStyle::ColonPrefix => "A colon before the label for namespace-like appearance",
            SeparatorStyle::SlashPrefix => {
                "A forward slash before the label for path-like appearance"
            }

            // Retro
            SeparatorStyle::AsciiBox => "Classic ASCII art box using + and - characters",
            SeparatorStyle::BoxDrawing => "Unicode box drawing characters for a technical look",
            SeparatorStyle::TerminalPrompt => "Styled like a terminal/shell prompt: ~/path $",
            SeparatorStyle::DosStyle => "Double-line DOS/BIOS style with box characters",
            SeparatorStyle::TypewriterRule => "Stacked single and double rules like a typewriter",

            // Modern
            SeparatorStyle::AnimatedFade => "Label that fades in with a smooth opacity animation",
            SeparatorStyle::BlurOverlay => "Backdrop blur effect behind the separator region",
            SeparatorStyle::NeonGlow => "Glowing neon effect around text and lines",
            SeparatorStyle::GlassCard => "Glassmorphism card with blur, border, and shadow",
            SeparatorStyle::FloatingLabel => "Label elevated with a drop shadow for depth",
        }
    }

    /// Get the category this separator belongs to.
    pub fn category(&self) -> SeparatorCategory {
        match self {
            SeparatorStyle::SolidLine
            | SeparatorStyle::DottedLine
            | SeparatorStyle::DashedLine
            | SeparatorStyle::DoubleLine
            | SeparatorStyle::HairlineSingle
            | SeparatorStyle::ThickBar
            | SeparatorStyle::FadeEdges => SeparatorCategory::LineBased,

            SeparatorStyle::UppercaseLabel
            | SeparatorStyle::SmallCapsLabel
            | SeparatorStyle::ItalicLabel
            | SeparatorStyle::BoldLabel
            | SeparatorStyle::UnderlinedLabel => SeparatorCategory::Typographic,

            SeparatorStyle::ChevronArrow
            | SeparatorStyle::DotsCenter
            | SeparatorStyle::DiamondDivider
            | SeparatorStyle::BracketWrap
            | SeparatorStyle::ArrowPointer
            | SeparatorStyle::StarDivider => SeparatorCategory::Decorative,

            SeparatorStyle::LargeGap
            | SeparatorStyle::TightGap
            | SeparatorStyle::IndentedLabel
            | SeparatorStyle::HangingIndent => SeparatorCategory::SpacingBased,

            SeparatorStyle::SubtleFill
            | SeparatorStyle::GradientFade
            | SeparatorStyle::FrostedPanel
            | SeparatorStyle::PillBadge => SeparatorCategory::Background,

            SeparatorStyle::Invisible
            | SeparatorStyle::SingleDot
            | SeparatorStyle::PipeChar
            | SeparatorStyle::ColonPrefix
            | SeparatorStyle::SlashPrefix => SeparatorCategory::Minimalist,

            SeparatorStyle::AsciiBox
            | SeparatorStyle::BoxDrawing
            | SeparatorStyle::TerminalPrompt
            | SeparatorStyle::DosStyle
            | SeparatorStyle::TypewriterRule => SeparatorCategory::Retro,

            SeparatorStyle::AnimatedFade
            | SeparatorStyle::BlurOverlay
            | SeparatorStyle::NeonGlow
            | SeparatorStyle::GlassCard
            | SeparatorStyle::FloatingLabel => SeparatorCategory::Modern,
        }
    }

    /// Get the default configuration for this separator style.
    pub fn default_config(&self) -> SeparatorConfig {
        let base = SeparatorConfig::default();

        match self {
            // Line-Based configurations
            SeparatorStyle::SolidLine => base,

            SeparatorStyle::DottedLine => SeparatorConfig {
                line_thickness: 2.0,
                ..base
            },

            SeparatorStyle::DashedLine => SeparatorConfig {
                line_thickness: 2.0,
                ..base
            },

            SeparatorStyle::DoubleLine => SeparatorConfig {
                height: 28.0,
                line_thickness: 1.0,
                ..base
            },

            SeparatorStyle::HairlineSingle => SeparatorConfig {
                line_thickness: 0.5,
                opacity: 0.5,
                ..base
            },

            SeparatorStyle::ThickBar => SeparatorConfig {
                line_thickness: 4.0,
                ..base
            },

            SeparatorStyle::FadeEdges => SeparatorConfig {
                line_thickness: 2.0,
                opacity: 0.8,
                ..base
            },

            // Typographic configurations
            SeparatorStyle::UppercaseLabel => SeparatorConfig {
                uppercase: true,
                letter_spacing: 1.5,
                show_decorations: true,
                ..base
            },

            SeparatorStyle::SmallCapsLabel => SeparatorConfig {
                uppercase: false,
                font_size: 10.0,
                letter_spacing: 0.5,
                ..base
            },

            SeparatorStyle::ItalicLabel => SeparatorConfig {
                italic: true,
                uppercase: false,
                ..base
            },

            SeparatorStyle::BoldLabel => SeparatorConfig {
                bold: true,
                uppercase: true,
                letter_spacing: 2.0,
                color_text: 0xa0a0a0,
                ..base
            },

            SeparatorStyle::UnderlinedLabel => SeparatorConfig {
                line_thickness: 1.0,
                padding_bottom: 6.0,
                ..base
            },

            // Decorative configurations
            SeparatorStyle::ChevronArrow => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 6.0,
                ..base
            },

            SeparatorStyle::DotsCenter => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 8.0,
                ..base
            },

            SeparatorStyle::DiamondDivider => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 12.0,
                ..base
            },

            SeparatorStyle::BracketWrap => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 4.0,
                ..base
            },

            SeparatorStyle::ArrowPointer => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 8.0,
                color_primary: 0xfbbf24, // Accent color
                ..base
            },

            SeparatorStyle::StarDivider => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 12.0,
                color_primary: 0xfbbf24, // Accent color
                ..base
            },

            // Spacing-Based configurations
            SeparatorStyle::LargeGap => SeparatorConfig {
                height: 32.0,
                padding_top: 16.0,
                padding_bottom: 8.0,
                ..base
            },

            SeparatorStyle::TightGap => SeparatorConfig {
                height: 16.0,
                padding_top: 4.0,
                padding_bottom: 2.0,
                font_size: 10.0,
                ..base
            },

            SeparatorStyle::IndentedLabel => SeparatorConfig {
                indent: 24.0,
                ..base
            },

            SeparatorStyle::HangingIndent => SeparatorConfig {
                indent: 0.0,
                padding_bottom: 2.0,
                ..base
            },

            // Background configurations
            SeparatorStyle::SubtleFill => SeparatorConfig {
                color_background: 0x2a2a2a,
                padding_x: 12.0,
                border_radius: 0.0,
                ..base
            },

            SeparatorStyle::GradientFade => SeparatorConfig {
                color_background: 0x2a2a2a,
                opacity: 0.6,
                ..base
            },

            SeparatorStyle::FrostedPanel => SeparatorConfig {
                color_background: 0x3a3a3a,
                border_radius: 6.0,
                padding_x: 12.0,
                shadow_blur: 4.0,
                shadow_offset_y: 2.0,
                ..base
            },

            SeparatorStyle::PillBadge => SeparatorConfig {
                color_background: 0x3a3a3a,
                border_radius: 12.0,
                padding_x: 16.0,
                font_size: 10.0,
                ..base
            },

            // Minimalist configurations
            SeparatorStyle::Invisible => SeparatorConfig {
                height: 16.0,
                show_decorations: false,
                opacity: 0.0,
                ..base
            },

            SeparatorStyle::SingleDot => SeparatorConfig {
                show_decorations: true,
                font_size: 8.0,
                opacity: 0.5,
                ..base
            },

            SeparatorStyle::PipeChar => SeparatorConfig {
                show_decorations: true,
                opacity: 0.4,
                ..base
            },

            SeparatorStyle::ColonPrefix => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 4.0,
                opacity: 0.6,
                ..base
            },

            SeparatorStyle::SlashPrefix => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 4.0,
                opacity: 0.6,
                ..base
            },

            // Retro configurations
            SeparatorStyle::AsciiBox => SeparatorConfig {
                height: 28.0,
                font_size: 12.0,
                color_text: 0x00ff00, // Terminal green
                color_primary: 0x00ff00,
                ..base
            },

            SeparatorStyle::BoxDrawing => SeparatorConfig {
                height: 24.0,
                font_size: 12.0,
                color_text: 0xcccccc,
                color_primary: 0x808080,
                ..base
            },

            SeparatorStyle::TerminalPrompt => SeparatorConfig {
                uppercase: false,
                font_size: 12.0,
                color_text: 0x00ff00,
                color_primary: 0x00aa00,
                ..base
            },

            SeparatorStyle::DosStyle => SeparatorConfig {
                height: 28.0,
                font_size: 12.0,
                color_text: 0xaaaaaa,
                color_primary: 0x555555,
                ..base
            },

            SeparatorStyle::TypewriterRule => SeparatorConfig {
                height: 32.0,
                line_thickness: 1.0,
                ..base
            },

            // Modern configurations
            SeparatorStyle::AnimatedFade => SeparatorConfig {
                opacity: 0.8,
                ..base
            },

            SeparatorStyle::BlurOverlay => SeparatorConfig {
                color_background: 0x1e1e1e,
                opacity: 0.7,
                border_radius: 4.0,
                ..base
            },

            SeparatorStyle::NeonGlow => SeparatorConfig {
                color_primary: 0x00ffff, // Cyan glow
                color_text: 0x00ffff,
                shadow_blur: 8.0,
                ..base
            },

            SeparatorStyle::GlassCard => SeparatorConfig {
                color_background: 0x3a3a3a,
                border_radius: 8.0,
                shadow_blur: 12.0,
                shadow_offset_y: 4.0,
                opacity: 0.9,
                ..base
            },

            SeparatorStyle::FloatingLabel => SeparatorConfig {
                shadow_blur: 6.0,
                shadow_offset_y: 2.0,
                color_background: 0x2a2a2a,
                border_radius: 4.0,
                ..base
            },
        }
    }

    /// Get the text prefix/decoration for this separator style (if any).
    ///
    /// Returns an optional tuple of (prefix, suffix) strings.
    pub fn decorations(&self) -> Option<(&'static str, &'static str)> {
        match self {
            SeparatorStyle::ChevronArrow => Some(("›› ", "")),
            SeparatorStyle::DotsCenter => Some(("••• ", " •••")),
            SeparatorStyle::DiamondDivider => Some(("◆ ", " ◆")),
            SeparatorStyle::BracketWrap => Some(("[ ", " ]")),
            SeparatorStyle::ArrowPointer => Some(("▶ ", "")),
            SeparatorStyle::StarDivider => Some(("★ ", " ★")),
            SeparatorStyle::ColonPrefix => Some((": ", "")),
            SeparatorStyle::SlashPrefix => Some(("/ ", "")),
            SeparatorStyle::AsciiBox => Some(("+--[ ", " ]--+")),
            SeparatorStyle::BoxDrawing => Some(("├── ", " ──┤")),
            SeparatorStyle::TerminalPrompt => Some(("~/", " $")),
            SeparatorStyle::DosStyle => Some(("══[ ", " ]══")),
            SeparatorStyle::BoldLabel => Some(("▌", "▐")),
            _ => None,
        }
    }

    /// Check if this separator style is compatible with a given design variant.
    ///
    /// Some separator styles work better with certain design systems.
    pub fn is_compatible_with(&self, variant: DesignVariant) -> bool {
        match self.category() {
            // Retro styles work best with RetroTerminal design
            SeparatorCategory::Retro => matches!(
                variant,
                DesignVariant::RetroTerminal | DesignVariant::Default
            ),

            // Modern styles work with most modern designs
            SeparatorCategory::Modern => {
                !matches!(variant, DesignVariant::RetroTerminal | DesignVariant::Paper)
            }

            // All other categories are universally compatible
            _ => true,
        }
    }

    /// Get recommended separator styles for a given design variant.
    pub fn recommended_for(variant: DesignVariant) -> Vec<SeparatorStyle> {
        match variant {
            DesignVariant::Default => vec![
                SeparatorStyle::UppercaseLabel,
                SeparatorStyle::SolidLine,
                SeparatorStyle::SubtleFill,
            ],
            DesignVariant::Minimal => vec![
                SeparatorStyle::Invisible,
                SeparatorStyle::HairlineSingle,
                SeparatorStyle::LargeGap,
            ],
            DesignVariant::RetroTerminal => vec![
                SeparatorStyle::TerminalPrompt,
                SeparatorStyle::BoxDrawing,
                SeparatorStyle::AsciiBox,
            ],
            DesignVariant::Glassmorphism => vec![
                SeparatorStyle::GlassCard,
                SeparatorStyle::FrostedPanel,
                SeparatorStyle::BlurOverlay,
            ],
            DesignVariant::Brutalist => vec![
                SeparatorStyle::ThickBar,
                SeparatorStyle::BoldLabel,
                SeparatorStyle::DoubleLine,
            ],
            DesignVariant::NeonCyberpunk => vec![
                SeparatorStyle::NeonGlow,
                SeparatorStyle::ChevronArrow,
                SeparatorStyle::DiamondDivider,
            ],
            DesignVariant::Paper => vec![
                SeparatorStyle::UnderlinedLabel,
                SeparatorStyle::ItalicLabel,
                SeparatorStyle::DottedLine,
            ],
            DesignVariant::AppleHIG => vec![
                SeparatorStyle::UppercaseLabel,
                SeparatorStyle::HairlineSingle,
                SeparatorStyle::SubtleFill,
            ],
            DesignVariant::Material3 => vec![
                SeparatorStyle::FloatingLabel,
                SeparatorStyle::PillBadge,
                SeparatorStyle::LargeGap,
            ],
            DesignVariant::Compact => vec![
                SeparatorStyle::TightGap,
                SeparatorStyle::HairlineSingle,
                SeparatorStyle::SmallCapsLabel,
            ],
            DesignVariant::Playful => vec![
                SeparatorStyle::PillBadge,
                SeparatorStyle::StarDivider,
                SeparatorStyle::DotsCenter,
            ],
        }
    }
}

// ============================================================================
// Separator Category
// ============================================================================

/// Categories for grouping separator styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeparatorCategory {
    /// Line-based separators using horizontal rules
    LineBased,
    /// Typographic separators focusing on text styling
    Typographic,
    /// Decorative separators with symbols and ornaments
    Decorative,
    /// Spacing-based separators using whitespace
    SpacingBased,
    /// Background-based separators with fills and panels
    Background,
    /// Minimalist separators with minimal visual weight
    Minimalist,
    /// Retro separators with ASCII/terminal aesthetics
    Retro,
    /// Modern separators with effects and animations
    Modern,
}

impl SeparatorCategory {
    /// Get the display name for this category.
    pub fn name(&self) -> &'static str {
        match self {
            SeparatorCategory::LineBased => "Line-Based",
            SeparatorCategory::Typographic => "Typographic",
            SeparatorCategory::Decorative => "Decorative",
            SeparatorCategory::SpacingBased => "Spacing-Based",
            SeparatorCategory::Background => "Background",
            SeparatorCategory::Minimalist => "Minimalist",
            SeparatorCategory::Retro => "Retro",
            SeparatorCategory::Modern => "Modern",
        }
    }

    /// Get all separator styles in this category.
    pub fn styles(&self) -> Vec<SeparatorStyle> {
        SeparatorStyle::all()
            .iter()
            .filter(|s| s.category() == *self)
            .copied()
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separator_count() {
        // Verify we have 25+ separator styles
        assert!(
            SeparatorStyle::count() >= 25,
            "Expected at least 25 separator styles, got {}",
            SeparatorStyle::count()
        );
    }

    #[test]
    fn test_all_styles_have_names() {
        for style in SeparatorStyle::all() {
            let name = style.name();
            assert!(!name.is_empty(), "Style {:?} has empty name", style);
        }
    }

    #[test]
    fn test_all_styles_have_descriptions() {
        for style in SeparatorStyle::all() {
            let desc = style.description();
            assert!(!desc.is_empty(), "Style {:?} has empty description", style);
            assert!(
                desc.len() > 20,
                "Style {:?} description too short: {}",
                style,
                desc
            );
        }
    }

    #[test]
    fn test_all_styles_have_categories() {
        for style in SeparatorStyle::all() {
            // This should not panic
            let _ = style.category();
        }
    }

    #[test]
    fn test_all_styles_have_default_configs() {
        for style in SeparatorStyle::all() {
            let config = style.default_config();
            assert!(config.height > 0.0, "Style {:?} has invalid height", style);
        }
    }

    #[test]
    fn test_category_coverage() {
        // Ensure all categories have at least one style
        let categories = [
            SeparatorCategory::LineBased,
            SeparatorCategory::Typographic,
            SeparatorCategory::Decorative,
            SeparatorCategory::SpacingBased,
            SeparatorCategory::Background,
            SeparatorCategory::Minimalist,
            SeparatorCategory::Retro,
            SeparatorCategory::Modern,
        ];

        for category in categories {
            let styles = category.styles();
            assert!(!styles.is_empty(), "Category {:?} has no styles", category);
        }
    }

    #[test]
    fn test_recommended_styles_exist() {
        for variant in DesignVariant::all() {
            let recommended = SeparatorStyle::recommended_for(*variant);
            assert!(
                !recommended.is_empty(),
                "No recommended styles for {:?}",
                variant
            );
        }
    }

    #[test]
    fn test_config_defaults_are_reasonable() {
        let config = SeparatorConfig::default();

        assert_eq!(
            config.height, 24.0,
            "Default height should match SECTION_HEADER_HEIGHT"
        );
        assert!(config.line_thickness >= 0.5 && config.line_thickness <= 4.0);
        assert!(config.padding_x > 0.0);
        assert!(config.opacity >= 0.0 && config.opacity <= 1.0);
    }

    #[test]
    fn test_style_default_is_solid_line() {
        assert_eq!(SeparatorStyle::default(), SeparatorStyle::SolidLine);
    }

    #[test]
    fn test_decorations_exist_for_decorated_styles() {
        // Styles that should have decorations
        let decorated = [
            SeparatorStyle::ChevronArrow,
            SeparatorStyle::DotsCenter,
            SeparatorStyle::BracketWrap,
            SeparatorStyle::AsciiBox,
            SeparatorStyle::BoxDrawing,
        ];

        for style in decorated {
            assert!(
                style.decorations().is_some(),
                "Style {:?} should have decorations",
                style
            );
        }
    }

    #[test]
    fn test_category_names_not_empty() {
        let categories = [
            SeparatorCategory::LineBased,
            SeparatorCategory::Typographic,
            SeparatorCategory::Decorative,
            SeparatorCategory::SpacingBased,
            SeparatorCategory::Background,
            SeparatorCategory::Minimalist,
            SeparatorCategory::Retro,
            SeparatorCategory::Modern,
        ];

        for category in categories {
            assert!(!category.name().is_empty());
        }
    }
}
