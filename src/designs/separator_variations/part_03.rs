impl SeparatorStyle {
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
}
