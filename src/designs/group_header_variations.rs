//! Group Header Style Variations
//!
//! This module provides different visual styles for group headers/section labels
//! like "MAIN", "RECENT", "SCRIPTS", etc.
//!
//! Group headers serve to visually separate different sections in a list.
//! These variations explore different approaches to styling and positioning.

/// Categories of group header styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupHeaderCategory {
    /// Simple text-based headers
    TextOnly,
    /// Headers with lines/rules
    WithLines,
    /// Headers with background styling
    WithBackground,
    /// Minimal/subtle headers
    Minimal,
    /// Headers with decorative elements
    Decorative,
}

impl GroupHeaderCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TextOnly => "Text Only",
            Self::WithLines => "With Lines",
            Self::WithBackground => "With Background",
            Self::Minimal => "Minimal",
            Self::Decorative => "Decorative",
        }
    }
    
    pub fn all() -> &'static [GroupHeaderCategory] {
        &[
            Self::TextOnly,
            Self::WithLines,
            Self::WithBackground,
            Self::Minimal,
            Self::Decorative,
        ]
    }
    
    pub fn styles(&self) -> &[GroupHeaderStyle] {
        match self {
            Self::TextOnly => &[
                GroupHeaderStyle::UppercaseLeft,
                GroupHeaderStyle::UppercaseCenter,
                GroupHeaderStyle::SmallCapsLeft,
                GroupHeaderStyle::BoldLeft,
                GroupHeaderStyle::LightLeft,
                GroupHeaderStyle::MonospaceLeft,
            ],
            Self::WithLines => &[
                GroupHeaderStyle::LineLeft,
                GroupHeaderStyle::LineRight,
                GroupHeaderStyle::LineBothSides,
                GroupHeaderStyle::LineBelow,
                GroupHeaderStyle::LineAbove,
                GroupHeaderStyle::DoubleLine,
            ],
            Self::WithBackground => &[
                GroupHeaderStyle::PillBackground,
                GroupHeaderStyle::FullWidthBackground,
                GroupHeaderStyle::SubtleBackground,
                GroupHeaderStyle::GradientFade,
                GroupHeaderStyle::BorderedBox,
            ],
            Self::Minimal => &[
                GroupHeaderStyle::DotPrefix,
                GroupHeaderStyle::DashPrefix,
                GroupHeaderStyle::BulletPrefix,
                GroupHeaderStyle::ArrowPrefix,
                GroupHeaderStyle::ChevronPrefix,
                GroupHeaderStyle::Dimmed,
            ],
            Self::Decorative => &[
                GroupHeaderStyle::Bracketed,
                GroupHeaderStyle::Quoted,
                GroupHeaderStyle::Tagged,
                GroupHeaderStyle::Numbered,
                GroupHeaderStyle::IconPrefix,
            ],
        }
    }
}

/// Individual group header styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupHeaderStyle {
    // Text Only
    UppercaseLeft,
    UppercaseCenter,
    SmallCapsLeft,
    BoldLeft,
    LightLeft,
    MonospaceLeft,
    
    // With Lines
    LineLeft,
    LineRight,
    LineBothSides,
    LineBelow,
    LineAbove,
    DoubleLine,
    
    // With Background
    PillBackground,
    FullWidthBackground,
    SubtleBackground,
    GradientFade,
    BorderedBox,
    
    // Minimal
    DotPrefix,
    DashPrefix,
    BulletPrefix,
    ArrowPrefix,
    ChevronPrefix,
    Dimmed,
    
    // Decorative
    Bracketed,
    Quoted,
    Tagged,
    Numbered,
    IconPrefix,
}

impl GroupHeaderStyle {
    /// Get all styles
    pub fn all() -> &'static [GroupHeaderStyle] {
        &[
            // Text Only
            Self::UppercaseLeft,
            Self::UppercaseCenter,
            Self::SmallCapsLeft,
            Self::BoldLeft,
            Self::LightLeft,
            Self::MonospaceLeft,
            // With Lines
            Self::LineLeft,
            Self::LineRight,
            Self::LineBothSides,
            Self::LineBelow,
            Self::LineAbove,
            Self::DoubleLine,
            // With Background
            Self::PillBackground,
            Self::FullWidthBackground,
            Self::SubtleBackground,
            Self::GradientFade,
            Self::BorderedBox,
            // Minimal
            Self::DotPrefix,
            Self::DashPrefix,
            Self::BulletPrefix,
            Self::ArrowPrefix,
            Self::ChevronPrefix,
            Self::Dimmed,
            // Decorative
            Self::Bracketed,
            Self::Quoted,
            Self::Tagged,
            Self::Numbered,
            Self::IconPrefix,
        ]
    }
    
    /// Get the total count of styles
    pub fn count() -> usize {
        Self::all().len()
    }
    
    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::UppercaseLeft => "Uppercase Left",
            Self::UppercaseCenter => "Uppercase Center",
            Self::SmallCapsLeft => "Small Caps Left",
            Self::BoldLeft => "Bold Left",
            Self::LightLeft => "Light Left",
            Self::MonospaceLeft => "Monospace Left",
            Self::LineLeft => "Line Left",
            Self::LineRight => "Line Right",
            Self::LineBothSides => "Line Both Sides",
            Self::LineBelow => "Line Below",
            Self::LineAbove => "Line Above",
            Self::DoubleLine => "Double Line",
            Self::PillBackground => "Pill Background",
            Self::FullWidthBackground => "Full Width Background",
            Self::SubtleBackground => "Subtle Background",
            Self::GradientFade => "Gradient Fade",
            Self::BorderedBox => "Bordered Box",
            Self::DotPrefix => "Dot Prefix",
            Self::DashPrefix => "Dash Prefix",
            Self::BulletPrefix => "Bullet Prefix",
            Self::ArrowPrefix => "Arrow Prefix",
            Self::ChevronPrefix => "Chevron Prefix",
            Self::Dimmed => "Dimmed Text",
            Self::Bracketed => "Bracketed",
            Self::Quoted => "Quoted",
            Self::Tagged => "Tagged",
            Self::Numbered => "Numbered",
            Self::IconPrefix => "Icon Prefix",
        }
    }
    
    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::UppercaseLeft => "ALL CAPS text aligned left",
            Self::UppercaseCenter => "ALL CAPS text centered",
            Self::SmallCapsLeft => "Small caps styling aligned left",
            Self::BoldLeft => "Bold weight text aligned left",
            Self::LightLeft => "Light weight text aligned left",
            Self::MonospaceLeft => "Monospace font aligned left",
            Self::LineLeft => "Line extending from text to left edge",
            Self::LineRight => "Line extending from text to right edge",
            Self::LineBothSides => "Lines on both sides of text",
            Self::LineBelow => "Line below the text",
            Self::LineAbove => "Line above the text",
            Self::DoubleLine => "Double lines framing the text",
            Self::PillBackground => "Rounded pill-shaped background",
            Self::FullWidthBackground => "Background spanning full width",
            Self::SubtleBackground => "Very subtle tinted background",
            Self::GradientFade => "Background fading at edges",
            Self::BorderedBox => "Text inside a bordered box",
            Self::DotPrefix => "Small dot before text",
            Self::DashPrefix => "Dash before text",
            Self::BulletPrefix => "Bullet point before text",
            Self::ArrowPrefix => "Arrow before text",
            Self::ChevronPrefix => "Chevron before text",
            Self::Dimmed => "Lower opacity/muted text",
            Self::Bracketed => "Text inside [brackets]",
            Self::Quoted => "Text inside quotes",
            Self::Tagged => "Text styled like a tag/badge",
            Self::Numbered => "Prefixed with section number",
            Self::IconPrefix => "Icon before the text",
        }
    }
    
    /// Get a sample rendering of this style (as formatted text)
    #[allow(dead_code)]
    pub fn sample(&self, label: &str) -> String {
        match self {
            Self::UppercaseLeft => label.to_uppercase(),
            Self::UppercaseCenter => label.to_uppercase(),
            Self::SmallCapsLeft => label.to_uppercase(),
            Self::BoldLeft => label.to_string(),
            Self::LightLeft => label.to_string(),
            Self::MonospaceLeft => label.to_uppercase(),
            Self::LineLeft => format!("---- {}", label.to_uppercase()),
            Self::LineRight => format!("{} ----", label.to_uppercase()),
            Self::LineBothSides => format!("-- {} --", label.to_uppercase()),
            Self::LineBelow => format!("{}\n----", label.to_uppercase()),
            Self::LineAbove => format!("----\n{}", label.to_uppercase()),
            Self::DoubleLine => format!("====\n{}\n====", label.to_uppercase()),
            Self::PillBackground => format!("( {} )", label.to_uppercase()),
            Self::FullWidthBackground => label.to_uppercase(),
            Self::SubtleBackground => label.to_uppercase(),
            Self::GradientFade => label.to_uppercase(),
            Self::BorderedBox => format!("[ {} ]", label.to_uppercase()),
            Self::DotPrefix => format!("\u{2022} {}", label.to_uppercase()),
            Self::DashPrefix => format!("- {}", label.to_uppercase()),
            Self::BulletPrefix => format!("\u{25CF} {}", label.to_uppercase()),
            Self::ArrowPrefix => format!("\u{25B8} {}", label.to_uppercase()),
            Self::ChevronPrefix => format!("\u{203A} {}", label.to_uppercase()),
            Self::Dimmed => label.to_uppercase(),
            Self::Bracketed => format!("[{}]", label.to_uppercase()),
            Self::Quoted => format!("\"{}\"", label.to_uppercase()),
            Self::Tagged => format!("<{}>", label.to_uppercase()),
            Self::Numbered => format!("01. {}", label.to_uppercase()),
            Self::IconPrefix => format!("\u{25A0} {}", label.to_uppercase()),
        }
    }
    
    /// Get the category this style belongs to
    #[allow(dead_code)]
    pub fn category(&self) -> GroupHeaderCategory {
        match self {
            Self::UppercaseLeft | Self::UppercaseCenter | Self::SmallCapsLeft |
            Self::BoldLeft | Self::LightLeft | Self::MonospaceLeft => GroupHeaderCategory::TextOnly,
            
            Self::LineLeft | Self::LineRight | Self::LineBothSides |
            Self::LineBelow | Self::LineAbove | Self::DoubleLine => GroupHeaderCategory::WithLines,
            
            Self::PillBackground | Self::FullWidthBackground | Self::SubtleBackground |
            Self::GradientFade | Self::BorderedBox => GroupHeaderCategory::WithBackground,
            
            Self::DotPrefix | Self::DashPrefix | Self::BulletPrefix |
            Self::ArrowPrefix | Self::ChevronPrefix | Self::Dimmed => GroupHeaderCategory::Minimal,
            
            Self::Bracketed | Self::Quoted | Self::Tagged |
            Self::Numbered | Self::IconPrefix => GroupHeaderCategory::Decorative,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_style_count() {
        assert_eq!(GroupHeaderStyle::count(), 28);
    }
    
    #[test]
    fn test_all_styles_have_names() {
        for style in GroupHeaderStyle::all() {
            assert!(!style.name().is_empty());
            assert!(!style.description().is_empty());
        }
    }
    
    #[test]
    fn test_category_contains_all_styles() {
        let mut total = 0;
        for cat in GroupHeaderCategory::all() {
            total += cat.styles().len();
        }
        assert_eq!(total, GroupHeaderStyle::count());
    }
    
    #[test]
    fn test_sample_generation() {
        let sample = GroupHeaderStyle::LineBothSides.sample("MAIN");
        assert!(sample.contains("MAIN"));
        assert!(sample.contains("--"));
    }
}
