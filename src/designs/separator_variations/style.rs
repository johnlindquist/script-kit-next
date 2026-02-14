use crate::designs::DesignVariant;

use super::{SeparatorCategory, SeparatorColorRole, SeparatorConfig};

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

#[allow(dead_code)]
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
}

impl SeparatorStyle {
    /// Get the category this separator belongs to.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
                color_text: SeparatorColorRole::TextSecondary,
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
                color_primary: SeparatorColorRole::AccentWarning,
                ..base
            },

            SeparatorStyle::StarDivider => SeparatorConfig {
                show_decorations: true,
                decoration_gap: 12.0,
                color_primary: SeparatorColorRole::AccentWarning,
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
                color_background: SeparatorColorRole::UiSurface,
                padding_x: 12.0,
                border_radius: 0.0,
                ..base
            },

            SeparatorStyle::GradientFade => SeparatorConfig {
                color_background: SeparatorColorRole::UiSurface,
                opacity: 0.6,
                ..base
            },

            SeparatorStyle::FrostedPanel => SeparatorConfig {
                color_background: SeparatorColorRole::UiSurfaceElevated,
                border_radius: 6.0,
                padding_x: 12.0,
                shadow_blur: 4.0,
                shadow_offset_y: 2.0,
                ..base
            },

            SeparatorStyle::PillBadge => SeparatorConfig {
                color_background: SeparatorColorRole::UiSurfaceElevated,
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
                color_text: SeparatorColorRole::AccentTerminal,
                color_primary: SeparatorColorRole::AccentTerminal,
                ..base
            },

            SeparatorStyle::BoxDrawing => SeparatorConfig {
                height: 24.0,
                font_size: 12.0,
                color_text: SeparatorColorRole::TextHighContrast,
                color_primary: SeparatorColorRole::TextMuted,
                ..base
            },

            SeparatorStyle::TerminalPrompt => SeparatorConfig {
                uppercase: false,
                font_size: 12.0,
                color_text: SeparatorColorRole::AccentTerminal,
                color_primary: SeparatorColorRole::AccentTerminalMuted,
                ..base
            },

            SeparatorStyle::DosStyle => SeparatorConfig {
                height: 28.0,
                font_size: 12.0,
                color_text: SeparatorColorRole::TextPrimary,
                color_primary: SeparatorColorRole::UiBorderMuted,
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
                color_background: SeparatorColorRole::UiSurfaceOverlay,
                opacity: 0.7,
                border_radius: 4.0,
                ..base
            },

            SeparatorStyle::NeonGlow => SeparatorConfig {
                color_primary: SeparatorColorRole::AccentNeon,
                color_text: SeparatorColorRole::AccentNeon,
                shadow_blur: 8.0,
                ..base
            },

            SeparatorStyle::GlassCard => SeparatorConfig {
                color_background: SeparatorColorRole::UiSurfaceElevated,
                border_radius: 8.0,
                shadow_blur: 12.0,
                shadow_offset_y: 4.0,
                opacity: 0.9,
                ..base
            },

            SeparatorStyle::FloatingLabel => SeparatorConfig {
                shadow_blur: 6.0,
                shadow_offset_y: 2.0,
                color_background: SeparatorColorRole::UiSurface,
                border_radius: 4.0,
                ..base
            },
        }
    }
}

impl SeparatorStyle {
    /// Get the text prefix/decoration for this separator style (if any).
    ///
    /// Returns an optional tuple of (prefix, suffix) strings.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Return styles that are not recommended by any current design variant.
    #[allow(dead_code)]
    pub fn unreferenced_in_recommendations() -> Vec<SeparatorStyle> {
        SeparatorStyle::all()
            .iter()
            .copied()
            .filter(|style| {
                !DesignVariant::all()
                    .iter()
                    .any(|variant| SeparatorStyle::recommended_for(*variant).contains(style))
            })
            .collect()
    }

    /// Return style pairs that share identical default config values.
    ///
    /// Matching configs are not always a bug (decorations may differ), but this
    /// list is useful during audits to spot potentially redundant variants.
    #[allow(dead_code)]
    pub fn shared_default_config_pairs() -> Vec<(SeparatorStyle, SeparatorStyle)> {
        let all = SeparatorStyle::all();
        let mut pairs = Vec::new();

        for (idx, left) in all.iter().enumerate() {
            for right in all.iter().skip(idx + 1) {
                if left.default_config() == right.default_config() {
                    pairs.push((*left, *right));
                }
            }
        }

        pairs
    }
}
