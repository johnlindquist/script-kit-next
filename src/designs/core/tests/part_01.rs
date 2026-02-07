    use super::*;

    #[test]
    fn test_all_variants_count() {
        assert_eq!(DesignVariant::all().len(), 11);
    }

    #[test]
    fn test_keyboard_number_round_trip() {
        for num in 0..=9 {
            let variant = DesignVariant::from_keyboard_number(num);
            assert!(
                variant.is_some(),
                "Keyboard number {} should map to a variant",
                num
            );

            let v = variant.unwrap();
            let shortcut = v.shortcut_number();

            // All variants except Playful should have shortcuts
            if v != DesignVariant::Playful {
                assert!(shortcut.is_some(), "Variant {:?} should have a shortcut", v);
                assert_eq!(
                    shortcut.unwrap(),
                    num,
                    "Round-trip failed for number {}",
                    num
                );
            }
        }
    }

    #[test]
    fn test_playful_has_no_shortcut() {
        assert_eq!(DesignVariant::Playful.shortcut_number(), None);
    }

    #[test]
    fn test_variant_names_not_empty() {
        for variant in DesignVariant::all() {
            assert!(
                !variant.name().is_empty(),
                "Variant {:?} should have a name",
                variant
            );
            assert!(
                !variant.description().is_empty(),
                "Variant {:?} should have a description",
                variant
            );
        }
    }

    #[test]
    fn test_default_variant() {
        assert_eq!(DesignVariant::default(), DesignVariant::Default);
    }

    #[test]
    fn test_uses_default_renderer() {
        // Minimal and RetroTerminal now have custom renderers
        assert!(
            !uses_default_renderer(DesignVariant::Minimal),
            "Minimal should NOT use default renderer"
        );
        assert!(
            !uses_default_renderer(DesignVariant::RetroTerminal),
            "RetroTerminal should NOT use default renderer"
        );

        // Default still uses default renderer
        assert!(
            uses_default_renderer(DesignVariant::Default),
            "Default should use default renderer"
        );

        // Other variants still use default renderer (until implemented)
        assert!(uses_default_renderer(DesignVariant::Brutalist));
        assert!(uses_default_renderer(DesignVariant::NeonCyberpunk));
    }

    #[test]
    fn test_get_item_height() {
        // Minimal uses taller items (64px)
        assert_eq!(get_item_height(DesignVariant::Minimal), MINIMAL_ITEM_HEIGHT);
        assert_eq!(get_item_height(DesignVariant::Minimal), 64.0);

        // RetroTerminal uses denser items (28px)
        assert_eq!(
            get_item_height(DesignVariant::RetroTerminal),
            TERMINAL_ITEM_HEIGHT
        );
        assert_eq!(get_item_height(DesignVariant::RetroTerminal), 28.0);

        // Compact uses the smallest items (24px)
        assert_eq!(get_item_height(DesignVariant::Compact), COMPACT_ITEM_HEIGHT);
        assert_eq!(get_item_height(DesignVariant::Compact), 24.0);

        // Default and others use standard height (40px - from design tokens)
        // Note: This differs from LIST_ITEM_HEIGHT (48.0) which is used for actual rendering
        assert_eq!(get_item_height(DesignVariant::Default), 40.0);
        assert_eq!(get_item_height(DesignVariant::Brutalist), 40.0);
    }

    #[test]
    fn test_design_variant_dispatch_coverage() {
        // Ensure all variants are covered by the dispatch logic
        // This test verifies the match arms in render_design_item cover all cases
        for variant in DesignVariant::all() {
            let uses_default = uses_default_renderer(*variant);
            let height = get_item_height(*variant);

            // All variants should have a defined height
            assert!(
                height > 0.0,
                "Variant {:?} should have positive item height",
                variant
            );

            // Minimal and RetroTerminal should use custom renderers
            if *variant == DesignVariant::Minimal || *variant == DesignVariant::RetroTerminal {
                assert!(
                    !uses_default,
                    "Variant {:?} should use custom renderer",
                    variant
                );
            }
        }
    }

    #[test]
    fn test_design_keyboard_coverage() {
        // Verify all keyboard shortcuts 1-0 are mapped
        let mut mapped_variants = Vec::new();
        for num in 0..=9 {
            if let Some(variant) = DesignVariant::from_keyboard_number(num) {
                mapped_variants.push(variant);
            }
        }
        // Should have 10 mapped variants (Cmd+1 through Cmd+0)
        assert_eq!(
            mapped_variants.len(),
            10,
            "Expected 10 keyboard-mapped variants"
        );

        // All mapped variants should be unique
        let mut unique = mapped_variants.clone();
        unique.sort_by_key(|v| *v as u8);
        unique.dedup_by_key(|v| *v as u8);
        assert_eq!(unique.len(), 10, "All keyboard mappings should be unique");
    }

    #[test]
    fn test_design_cycling() {
        // Test that next() cycles through all designs
        let all = DesignVariant::all();
        let mut current = DesignVariant::Default;

        // Cycle through all designs
        for (i, expected) in all.iter().enumerate() {
            assert_eq!(
                current, *expected,
                "Cycle iteration {} should be {:?}",
                i, expected
            );
            current = current.next();
        }

        // After cycling through all, we should be back at Default
        assert_eq!(
            current,
            DesignVariant::Default,
            "Should cycle back to Default"
        );
    }

    #[test]
    fn test_design_prev() {
        // Test that prev() goes backwards
        let current = DesignVariant::Default;
        let prev = current.prev();

        // Default.prev() should be Playful (last in list)
        assert_eq!(prev, DesignVariant::Playful);

        // And prev of that should be Compact
        assert_eq!(prev.prev(), DesignVariant::Compact);
    }

    // =========================================================================
    // DesignTokens Tests
    // =========================================================================

    #[test]
    fn test_get_tokens_returns_correct_variant() {
        // Verify get_tokens returns tokens with matching variant
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            assert_eq!(
                tokens.variant(),
                *variant,
                "get_tokens({:?}) returned tokens for {:?}",
                variant,
                tokens.variant()
            );
        }
    }

    #[test]
    fn test_get_tokens_item_height_matches() {
        // Verify token item_height matches get_item_height function
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            let fn_height = get_item_height(*variant);
            let token_height = tokens.item_height();

            assert_eq!(
                fn_height, token_height,
                "Item height mismatch for {:?}: get_item_height={}, tokens.item_height={}",
                variant, fn_height, token_height
            );
        }
    }

    #[test]
    fn test_design_colors_defaults() {
        let colors = DesignColors::default();

        // Verify expected defaults
        assert_eq!(colors.background, 0x1e1e1e);
        assert_eq!(colors.text_primary, 0xffffff);
        assert_eq!(colors.accent, 0xfbbf24);
        assert_eq!(colors.border, 0x464647);
    }

    #[test]
    fn test_design_spacing_defaults() {
        let spacing = DesignSpacing::default();

        // Verify expected defaults
        assert_eq!(spacing.padding_xs, 4.0);
        assert_eq!(spacing.padding_md, 12.0);
        assert_eq!(spacing.gap_md, 8.0);
        assert_eq!(spacing.item_padding_x, 16.0);
    }

    #[test]
    fn test_design_typography_defaults() {
        let typography = DesignTypography::default();

        // Verify expected defaults
        assert_eq!(typography.font_family, ".AppleSystemUIFont");
        assert_eq!(typography.font_family_mono, "Menlo");
        assert_eq!(typography.font_size_md, 14.0);
    }

    #[test]
    fn test_design_visual_defaults() {
        let visual = DesignVisual::default();

        // Verify expected defaults
        assert_eq!(visual.radius_sm, 4.0);
        assert_eq!(visual.radius_md, 8.0);
        assert_eq!(visual.shadow_opacity, 0.25);
        assert_eq!(visual.border_thin, 1.0);
    }

    #[test]
    fn test_design_tokens_are_copy() {
        // Verify all token structs are Copy (needed for closure efficiency)
        fn assert_copy<T: Copy>() {}

        assert_copy::<DesignColors>();
        assert_copy::<DesignSpacing>();
        assert_copy::<DesignTypography>();
        assert_copy::<DesignVisual>();
    }

    #[test]
    fn test_minimal_tokens_distinctive() {
        let tokens = MinimalDesignTokens;

        // Minimal should have taller items and more generous padding
        assert_eq!(tokens.item_height(), 64.0);
        assert_eq!(tokens.spacing().item_padding_x, 80.0);
        assert_eq!(tokens.visual().radius_md, 0.0); // No borders
    }

    #[test]
    fn test_retro_terminal_tokens_distinctive() {
        let tokens = RetroTerminalDesignTokens;

        // Terminal should have dense items and phosphor green colors
        assert_eq!(tokens.item_height(), 28.0);
        assert_eq!(tokens.colors().text_primary, 0x00ff00); // Phosphor green
        assert_eq!(tokens.colors().background, 0x000000); // Pure black
        assert_eq!(tokens.typography().font_family, "Menlo");
    }

    #[test]
    fn test_compact_tokens_distinctive() {
        let tokens = CompactDesignTokens;

        // Compact should have smallest items
        assert_eq!(tokens.item_height(), 24.0);
        assert!(tokens.spacing().padding_md < DesignSpacing::default().padding_md);
    }

    #[test]
    fn test_all_variants_have_positive_item_height() {
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            assert!(
                tokens.item_height() > 0.0,
                "Variant {:?} has non-positive item height",
                variant
            );
        }
    }

    #[test]
    fn test_all_variants_have_valid_colors() {
        for variant in DesignVariant::all() {
            let tokens = get_tokens(*variant);
            let colors = tokens.colors();

            // Background should be different from text (for contrast)
            assert_ne!(
                colors.background, colors.text_primary,
                "Variant {:?} has no contrast between bg and text",
                variant
            );
        }
    }

    // =========================================================================
    // Auto-description tests
    // =========================================================================

