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
