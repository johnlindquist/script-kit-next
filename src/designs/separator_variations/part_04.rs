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
