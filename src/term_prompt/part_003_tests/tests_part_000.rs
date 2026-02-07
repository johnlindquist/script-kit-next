    use super::*;
    // ========================================================================
    // Ctrl+Key Handling Tests (TDD)
    // ========================================================================

    #[test]
    fn test_ctrl_c_returns_sigint() {
        // Ctrl+C should return 0x03 (ETX - End of Text / SIGINT)
        assert_eq!(TermPrompt::ctrl_key_to_byte("c"), Some(0x03));
        assert_eq!(TermPrompt::ctrl_key_to_byte("C"), Some(0x03));
    }
    #[test]
    fn test_ctrl_d_returns_eof() {
        // Ctrl+D should return 0x04 (EOT - End of Transmission / EOF)
        assert_eq!(TermPrompt::ctrl_key_to_byte("d"), Some(0x04));
        assert_eq!(TermPrompt::ctrl_key_to_byte("D"), Some(0x04));
    }
    #[test]
    fn test_ctrl_z_returns_sigtstp() {
        // Ctrl+Z should return 0x1A (SUB - Substitute / SIGTSTP)
        assert_eq!(TermPrompt::ctrl_key_to_byte("z"), Some(0x1A));
        assert_eq!(TermPrompt::ctrl_key_to_byte("Z"), Some(0x1A));
    }
    #[test]
    fn test_ctrl_l_returns_clear() {
        // Ctrl+L should return 0x0C (FF - Form Feed / clear screen)
        assert_eq!(TermPrompt::ctrl_key_to_byte("l"), Some(0x0C));
        assert_eq!(TermPrompt::ctrl_key_to_byte("L"), Some(0x0C));
    }
    #[test]
    fn test_ctrl_a_through_z() {
        // Test all Ctrl+letter combinations
        let expected: [(char, u8); 26] = [
            ('a', 0x01),
            ('b', 0x02),
            ('c', 0x03),
            ('d', 0x04),
            ('e', 0x05),
            ('f', 0x06),
            ('g', 0x07),
            ('h', 0x08),
            ('i', 0x09),
            ('j', 0x0A),
            ('k', 0x0B),
            ('l', 0x0C),
            ('m', 0x0D),
            ('n', 0x0E),
            ('o', 0x0F),
            ('p', 0x10),
            ('q', 0x11),
            ('r', 0x12),
            ('s', 0x13),
            ('t', 0x14),
            ('u', 0x15),
            ('v', 0x16),
            ('w', 0x17),
            ('x', 0x18),
            ('y', 0x19),
            ('z', 0x1A),
        ];

        for (ch, expected_byte) in expected {
            let result = TermPrompt::ctrl_key_to_byte(&ch.to_string());
            assert_eq!(
                result,
                Some(expected_byte),
                "Ctrl+{} should return 0x{:02X}",
                ch,
                expected_byte
            );
        }
    }
    #[test]
    fn test_ctrl_bracket_returns_esc() {
        // Ctrl+[ should return 0x1B (ESC)
        assert_eq!(TermPrompt::ctrl_key_to_byte("["), Some(0x1B));
    }
    #[test]
    fn test_ctrl_backslash_returns_sigquit() {
        // Ctrl+\ should return 0x1C (SIGQUIT)
        assert_eq!(TermPrompt::ctrl_key_to_byte("\\"), Some(0x1C));
    }
    #[test]
    fn test_ctrl_special_chars() {
        // Test other special control characters
        assert_eq!(TermPrompt::ctrl_key_to_byte("]"), Some(0x1D));
        assert_eq!(TermPrompt::ctrl_key_to_byte("^"), Some(0x1E));
        assert_eq!(TermPrompt::ctrl_key_to_byte("_"), Some(0x1F));
    }
    #[test]
    fn test_ctrl_invalid_key_returns_none() {
        // Non-control keys should return None
        assert_eq!(TermPrompt::ctrl_key_to_byte("1"), None);
        assert_eq!(TermPrompt::ctrl_key_to_byte("!"), None);
        assert_eq!(TermPrompt::ctrl_key_to_byte("@"), None);
        assert_eq!(TermPrompt::ctrl_key_to_byte(" "), None);
        assert_eq!(TermPrompt::ctrl_key_to_byte("enter"), None);
        assert_eq!(TermPrompt::ctrl_key_to_byte("escape"), None);
    }
    // ========================================================================
    // Cell Dimension Tests
    // ========================================================================

    #[test]
    fn test_cell_dimensions_are_reasonable() {
        // Menlo 14pt should have reasonable cell dimensions
        const _: () = assert!(CELL_WIDTH > 5.0 && CELL_WIDTH < 15.0);
        const _: () = assert!(CELL_HEIGHT > 10.0 && CELL_HEIGHT < 25.0);
    }
    #[test]
    fn test_refresh_interval_is_reasonable() {
        // Refresh can be up to 120fps (8ms) for smoother terminal output
        const _: () = assert!(REFRESH_INTERVAL_MS >= 4);
        const _: () = assert!(REFRESH_INTERVAL_MS <= 100);
    }
    // ========================================================================
    // Terminal Size Calculation Tests
    // ========================================================================

    #[test]
    fn test_calculate_terminal_size_basic() {
        use gpui::px;

        // Window of 750x500 pixels with default padding (12 left, 12 right, 8 top, 8 bottom)
        // Available width: 750 - 12 - 12 = 726
        // Available height: 500 - 8 - 8 = 484
        let (cols, rows) =
            TermPrompt::calculate_terminal_size(px(750.0), px(500.0), 12.0, 12.0, 8.0, 8.0);

        // Expected: 726 / 8.5 = 85.4 -> 85 cols
        // Expected: 484 / 18.2 = 26.6 -> 26 rows
        assert!(
            (80..=90).contains(&cols),
            "Cols should be around 85, got {}",
            cols
        );
        assert!(
            (24..=28).contains(&rows),
            "Rows should be around 26, got {}",
            rows
        );
    }
    #[test]
    fn test_calculate_terminal_size_minimum() {
        use gpui::px;

        // Very small window should return minimum size
        let (cols, rows) =
            TermPrompt::calculate_terminal_size(px(50.0), px(50.0), 0.0, 0.0, 0.0, 0.0);

        assert_eq!(cols, MIN_COLS, "Should use minimum cols");
        assert_eq!(rows, MIN_ROWS, "Should use minimum rows");
    }
    #[test]
    fn test_calculate_terminal_size_large() {
        use gpui::px;

        // Large window (1920x1080) with no padding
        let (cols, rows) =
            TermPrompt::calculate_terminal_size(px(1920.0), px(1080.0), 0.0, 0.0, 0.0, 0.0);

        // Should be roughly 225 cols x 59 rows
        assert!(
            cols > 200,
            "Large window should have many cols, got {}",
            cols
        );
        assert!(
            rows > 50,
            "Large window should have many rows, got {}",
            rows
        );
    }
    #[test]
    fn test_calculate_terminal_size_conservative() {
        use gpui::px;

        // Test that we use conservative column calculation to prevent wrapping.
        // CELL_WIDTH is 8.5px (slightly larger than actual 8.4287px Menlo width)
        // to ensure we never tell PTY we have more columns than can render.

        // With no padding: 680px / 8.5 = 80.0 -> exactly 80 cols
        let (cols, _rows) =
            TermPrompt::calculate_terminal_size(px(680.0), px(500.0), 0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            cols, 80,
            "680px width should give 80 cols (680/8.5=80), got {}",
            cols
        );

        // 679px / 8.5 = 79.88 -> floors to 79 cols (conservative)
        let (cols2, _) =
            TermPrompt::calculate_terminal_size(px(679.0), px(500.0), 0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            cols2, 79,
            "679px width should give 79 cols (679/8.5=79.88 floors to 79), got {}",
            cols2
        );

        // 500px / 8.5 = 58.82 -> floors to 58 cols
        let (cols3, _) =
            TermPrompt::calculate_terminal_size(px(500.0), px(500.0), 0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            cols3, 58,
            "500px width should give 58 cols (500/8.5=58.82 floors to 58), got {}",
            cols3
        );
    }
    #[test]
    fn test_calculate_terminal_size_with_padding() {
        use gpui::px;

        // Test that padding is properly subtracted from available space
        // 500px width with 12px left and 12px right padding = 476px available
        // 476 / 8.5 = 56.0 -> 56 cols
        let (cols, _) =
            TermPrompt::calculate_terminal_size(px(500.0), px(500.0), 12.0, 12.0, 0.0, 0.0);
        assert_eq!(
            cols, 56,
            "500px with 24px total horizontal padding should give 56 cols, got {}",
            cols
        );

        // 500px height with 8px top padding only = 492px available
        // 492 / 18.2 = 27.0 -> 27 rows
        let (_, rows) =
            TermPrompt::calculate_terminal_size(px(500.0), px(500.0), 0.0, 0.0, 8.0, 0.0);
        assert_eq!(
            rows, 27,
            "500px with 8px top padding only should give 27 rows, got {}",
            rows
        );

        // 500px height with 8px top AND 8px bottom padding = 484px available
        // 484 / 18.2 = 26.6 -> 26 rows
        let (_, rows2) =
            TermPrompt::calculate_terminal_size(px(500.0), px(500.0), 0.0, 0.0, 8.0, 8.0);
        assert_eq!(
            rows2, 26,
            "500px with 8px top+bottom padding should give 26 rows, got {}",
            rows2
        );
    }
    // ========================================================================
    // Padding Symmetry Regression Tests
    //
    // BUG FIXED: calculate_terminal_size only subtracted padding_top from height,
    // but render() applied BOTH top AND bottom padding, causing content cutoff.
    // These tests ensure the fix is never regressed.
    // ========================================================================

    #[test]
    fn test_padding_symmetry_regression_top_and_bottom_must_both_be_subtracted() {
        use gpui::px;

        // REGRESSION TEST: This test would FAIL if padding_bottom is not subtracted.
        //
        // Scenario: 700px window height with 8px top and 8px bottom padding
        // render() applies: pt(8) + pb(8) = 16px total vertical padding
        // calculate_terminal_size MUST subtract BOTH:
        //   available_height = 700 - 8 - 8 = 684px
        //   rows = floor(684 / 18.2) = 37 rows
        //
        // If only padding_top was subtracted (the bug):
        //   available_height = 700 - 8 = 692px
        //   rows = floor(692 / 18.2) = 38 rows
        //   Then 38 * 18.2 = 691.6px > 684px available = CONTENT CUTOFF!

        let padding_top = 8.0;
        let padding_bottom = 8.0;
        let total_height = 700.0;

        let (_, rows) = TermPrompt::calculate_terminal_size(
            px(500.0),
            px(total_height),
            0.0,
            0.0,
            padding_top,
            padding_bottom,
        );

        // Verify the row count accounts for BOTH paddings
        let expected_available_height = total_height - padding_top - padding_bottom;
        let expected_rows = (expected_available_height / CELL_HEIGHT).floor() as u16;

        assert_eq!(
            rows, expected_rows,
            "REGRESSION: padding_bottom not being subtracted! \
            Expected {} rows (684px / 18.2), got {} rows. \
            This means content will be cut off!",
            expected_rows, rows
        );

        // Additional invariant: rendered content must fit within available space
        let content_height = rows as f32 * CELL_HEIGHT;
        let available_height = total_height - padding_top - padding_bottom;
        assert!(
            content_height <= available_height,
            "REGRESSION: Content ({:.1}px = {} rows × {:.1}px) exceeds available height ({:.1}px)!",
            content_height,
            rows,
            CELL_HEIGHT,
            available_height
        );
    }
    #[test]
    fn test_padding_symmetry_invariant_content_plus_padding_never_exceeds_total() {
        use gpui::px;

        // INVARIANT TEST: rows * CELL_HEIGHT + padding_top + padding_bottom <= total_height
        // This must hold for ANY valid padding values.

        let test_cases: Vec<(f32, f32, f32)> = vec![
            // (total_height, padding_top, padding_bottom)
            (700.0, 8.0, 8.0),   // Default case
            (500.0, 8.0, 8.0),   // Smaller window
            (700.0, 16.0, 16.0), // Larger padding
            (700.0, 0.0, 0.0),   // No padding
            (700.0, 20.0, 20.0), // Very large padding
            (400.0, 50.0, 50.0), // Extreme padding ratio
        ];

        for (total_height, padding_top, padding_bottom) in test_cases {
            let (_, rows) = TermPrompt::calculate_terminal_size(
                px(500.0),
                px(total_height),
                0.0,
                0.0,
                padding_top,
                padding_bottom,
            );

            let content_height = rows as f32 * CELL_HEIGHT;
            let total_used = content_height + padding_top + padding_bottom;

            assert!(
                total_used <= total_height,
                "INVARIANT VIOLATED for height={}, top={}, bottom={}: \
                content ({} rows × {:.1}px = {:.1}px) + padding ({:.1}+{:.1}={:.1}px) = {:.1}px > {:.1}px!",
                total_height, padding_top, padding_bottom,
                rows, CELL_HEIGHT, content_height,
                padding_top, padding_bottom, padding_top + padding_bottom,
                total_used, total_height
            );
        }
    }
    #[test]
    fn test_padding_edge_case_padding_exceeds_available_height() {
        use gpui::px;

        // EDGE CASE: What happens when padding is larger than available height?
        // Should return MIN_ROWS to prevent panic/negative values.

        let total_height = 50.0;
        let padding_top = 30.0;
        let padding_bottom = 30.0;
        // Available height = 50 - 30 - 30 = -10px (negative!)

        let (_, rows) = TermPrompt::calculate_terminal_size(
            px(500.0),
            px(total_height),
            0.0,
            0.0,
            padding_top,
            padding_bottom,
        );

        // Should return minimum rows, not crash or return 0
        assert_eq!(
            rows, MIN_ROWS,
            "When padding exceeds height, should return MIN_ROWS ({}), got {}",
            MIN_ROWS, rows
        );
    }
    #[test]
    fn test_padding_symmetry_max_height_scenario() {
        use gpui::px;

        // This test uses the actual MAX_HEIGHT (700px) from window_resize.rs
        // to verify the exact scenario that was causing content cutoff.
        const MAX_HEIGHT: f32 = 700.0;
        const DEFAULT_PADDING: f32 = 8.0; // From config defaults

        let (_, rows) = TermPrompt::calculate_terminal_size(
            px(500.0),
            px(MAX_HEIGHT),
            12.0,
            12.0,            // left/right padding
            DEFAULT_PADDING, // top
            DEFAULT_PADDING, // bottom
        );

        // With 700px and 8+8=16px vertical padding:
        // Available = 684px, rows = floor(684/18.2) = 37
        assert_eq!(
            rows, 37,
            "MAX_HEIGHT (700px) with 8px symmetric padding should give 37 rows, got {}. \
            This was the exact bug scenario!",
            rows
        );

        // Verify no cutoff
        let content_height = rows as f32 * CELL_HEIGHT;
        let available_height = MAX_HEIGHT - DEFAULT_PADDING - DEFAULT_PADDING;

        assert!(
            content_height <= available_height,
            "Content ({:.1}px) exceeds available space ({:.1}px) - cutoff will occur!",
            content_height,
            available_height
        );
    }
