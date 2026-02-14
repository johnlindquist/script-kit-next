use super::SeparatorStyle;

/// Categories for grouping separator styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn styles(&self) -> Vec<SeparatorStyle> {
        SeparatorStyle::all()
            .iter()
            .filter(|style| style.category() == *self)
            .copied()
            .collect()
    }
}
