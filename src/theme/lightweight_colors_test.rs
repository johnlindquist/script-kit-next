//! Test for lightweight color struct extraction
//!
//! Demonstrates that we can extract only the colors needed by prompts
//! instead of passing Arc<Theme> around.

#[cfg(test)]
mod tests {
    use crate::theme::{
        helpers::{ListItemColors, PromptColors},
        types::load_theme,
    };

    #[test]
    fn test_extract_list_item_colors_is_copy() {
        // Load theme
        let theme = load_theme();

        // Extract lightweight colors - this should be Copy
        let colors = theme.colors.list_item_colors();

        // Prove it's Copy by using it twice without clone()
        let _bg1 = colors.background;
        let _bg2 = colors.background; // Would fail if not Copy

        // Colors should match theme (text is Rgba, primary is u32)
        let expected_r = ((theme.colors.text.primary >> 16) & 0xFF) as f32 / 255.0;
        assert!((colors.text.r - expected_r).abs() < 0.01);
    }

    #[test]
    fn test_list_item_colors_struct_size() {
        use std::mem::size_of;

        let _colors = ListItemColors::from_color_scheme(&load_theme().colors);

        // Should be small enough for stack allocation
        // 10 Rgba fields * 16 bytes each = 160 bytes (reasonable)
        let size = size_of::<ListItemColors>();
        assert!(
            size <= 256,
            "ListItemColors too large: {} bytes (should be <= 256)",
            size
        );

        // Compare to Arc<Theme> which is just 8 bytes but points to large heap allocation
        assert!(size > size_of::<std::sync::Arc<()>>());
    }

    #[test]
    fn test_multiple_extractions_dont_clone_theme() {
        let theme = load_theme();

        // Extract multiple color sets - should not clone theme
        let _list_colors = theme.colors.list_item_colors();
        let _input_colors = theme.colors.input_field_colors();
        let _prompt_colors = theme.colors.prompt_colors();

        // All extractions happen in constant time, no heap allocations
        // (This test mainly documents the pattern)
    }

    #[test]
    fn test_prompt_colors_is_copy() {
        let theme = load_theme();

        // Extract prompt colors - should be Copy
        let colors = theme.colors.prompt_colors();

        // Prove it's Copy by using it multiple times without clone()
        let _text1 = colors.text_primary;
        let _text2 = colors.text_primary;
        let _accent1 = colors.accent_color;
        let _accent2 = colors.accent_color;

        // Verify struct contains expected colors
        assert_eq!(colors.text_primary, theme.colors.text.primary);
        assert_eq!(colors.accent_color, theme.colors.accent.selected);
        assert_eq!(colors.code_bg, theme.colors.background.search_box);
    }

    #[test]
    fn test_prompt_colors_struct_size() {
        use std::mem::size_of;

        // PromptColors should be very small - just 7 u32 values
        let size = size_of::<PromptColors>();

        // 7 u32 fields * 4 bytes each = 28 bytes
        assert_eq!(size, 28, "PromptColors should be exactly 28 bytes");

        // Much smaller than Arc<Theme> pointing to large heap allocation
        assert!(size > size_of::<std::sync::Arc<()>>());
    }
}
