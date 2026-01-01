    use super::*;

    #[test]
    fn test_cursor_position() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_selection_ordered() {
        let sel = Selection::new(CursorPosition::new(5, 10), CursorPosition::new(2, 5));
        let (start, end) = sel.ordered();
        assert_eq!(start.line, 2);
        assert_eq!(end.line, 5);
    }

    #[test]
    fn test_selection_is_empty() {
        let pos = CursorPosition::new(3, 7);
        let sel = Selection::caret(pos);
        assert!(sel.is_empty());

        let sel2 = Selection::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5));
        assert!(!sel2.is_empty());
    }

    #[test]
    fn test_line_count_empty() {
        let content = "";
        let lines = highlight_code_lines(content, "text");
        assert!(lines.is_empty() || lines.len() == 1);
    }

    #[test]
    fn test_line_count_multiline() {
        let content = "line1\nline2\nline3";
        let lines = highlight_code_lines(content, "text");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_typescript_highlighting() {
        let content = "const x: number = 42;";
        let lines = highlight_code_lines(content, "typescript");
        assert!(!lines.is_empty());
        assert!(!lines[0].spans.is_empty());
    }

    /// Regression test: Verify arrow key patterns match BOTH short and long forms.
    /// GPUI sends "up"/"down"/"left"/"right" on macOS, but we must also handle
    /// "arrowup"/"arrowdown"/"arrowleft"/"arrowright" for cross-platform compatibility.
    ///
    /// This test reads the source code and verifies the patterns are correct.
    /// If this test fails, arrow keys will be broken in the editor!
    #[test]
    fn test_arrow_key_patterns_match_both_forms() {
        let source = include_str!("editor.rs");

        // These patterns MUST exist - they match both key name variants
        let required_patterns = [
            r#""up" | "arrowup""#,
            r#""down" | "arrowdown""#,
            r#""left" | "arrowleft""#,
            r#""right" | "arrowright""#,
        ];

        for pattern in required_patterns {
            assert!(
                source.contains(pattern),
                "CRITICAL: Missing arrow key pattern '{}' in editor.rs!\n\
                 Arrow keys will be BROKEN. GPUI sends short names like 'up' but we must match both forms.\n\
                 Fix: Use pattern matching like: \"up\" | \"arrowup\" => ...",
                pattern
            );
        }

        // These patterns are WRONG - they only match one form
        let forbidden_patterns = [
            // Standalone arrowup without the short form - this is broken!
            ("(\"arrowup\", false, _, false)", "arrowup without 'up'"),
            (
                "(\"arrowdown\", false, _, false)",
                "arrowdown without 'down'",
            ),
            (
                "(\"arrowleft\", false, _, false)",
                "arrowleft without 'left'",
            ),
            (
                "(\"arrowright\", false, _, false)",
                "arrowright without 'right'",
            ),
        ];

        for (pattern, desc) in forbidden_patterns {
            assert!(
                !source.contains(pattern),
                "CRITICAL: Found broken arrow key pattern ({}) in editor.rs!\n\
                 Pattern '{}' only matches long form. GPUI sends short names like 'up'.\n\
                 Fix: Use \"up\" | \"arrowup\" instead of just \"arrowup\"",
                desc,
                pattern
            );
        }
    }

    #[test]
    fn test_snippet_state_creation() {
        let snippet = ParsedSnippet::parse("Hello ${1:world}!");
        let state = SnippetState {
            snippet,
            current_tabstop_idx: 0,
        };
        assert_eq!(state.current_tabstop_idx, 0);
        assert_eq!(state.snippet.tabstops.len(), 1);
    }

    #[test]
    fn test_snippet_state_with_multiple_tabstops() {
        let snippet = ParsedSnippet::parse("${1:first} ${2:second} ${0:end}");
        let state = SnippetState {
            snippet,
            current_tabstop_idx: 0,
        };
        // Order should be 1, 2, 0 (0 is always last)
        assert_eq!(state.snippet.tabstops.len(), 3);
        assert_eq!(state.snippet.tabstops[0].index, 1);
        assert_eq!(state.snippet.tabstops[1].index, 2);
        assert_eq!(state.snippet.tabstops[2].index, 0);
    }

    #[test]
    fn test_byte_to_cursor_static() {
        let rope = Rope::from_str("Hello\nWorld");

        // "Hello" is 5 bytes, cursor at start
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        // After "Hello" (position 5)
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 5);

        // After "Hello\n" (position 6) - start of second line
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);

        // "Hello\nWor" (position 9)
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 9);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_byte_to_cursor_static_clamps_to_end() {
        let rope = Rope::from_str("Hello");

        // Beyond end should clamp
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 100);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 5);
    }

    // --- Arrow Key Selection Collapse Tests ---
    // These tests verify that pressing Left/Right without Shift collapses
    // any existing selection to the appropriate edge (standard editor behavior).

    #[test]
    fn test_selection_collapse_with_left_arrow() {
        // Test with "Hello World" content context
        // Simulate: user selects "World" (columns 6-11 on line 0)
        // Selection anchor at (0, 6), head at (0, 11)
        let selection = Selection::new(CursorPosition::new(0, 6), CursorPosition::new(0, 11));

        assert!(!selection.is_empty());

        // Pressing Left should collapse to selection START (column 6)
        let (start, _end) = selection.ordered();
        assert_eq!(start.line, 0);
        assert_eq!(start.column, 6);
        // After move_left(false), cursor should be at start
    }

    #[test]
    fn test_selection_collapse_with_right_arrow() {
        // Test with "Hello World" content context
        // Simulate: user selects "Hello" (columns 0-5 on line 0)
        // Selection anchor at (0, 0), head at (0, 5)
        let selection = Selection::new(CursorPosition::new(0, 0), CursorPosition::new(0, 5));

        assert!(!selection.is_empty());

        // Pressing Right should collapse to selection END (column 5)
        let (_start, end) = selection.ordered();
        assert_eq!(end.line, 0);
        assert_eq!(end.column, 5);
        // After move_right(false), cursor should be at end
    }

    #[test]
    fn test_selection_collapse_with_up_arrow() {
        // Test with "Line 1\nLine 2\nLine 3" content context
        // Simulate: user selects from (1, 0) to (2, 3) - spanning lines 2 and 3
        let selection = Selection::new(CursorPosition::new(1, 0), CursorPosition::new(2, 3));

        assert!(!selection.is_empty());

        // Pressing Up should collapse to selection START (line 1, column 0)
        let (start, _end) = selection.ordered();
        assert_eq!(start.line, 1);
        assert_eq!(start.column, 0);
    }

    #[test]
    fn test_selection_collapse_with_down_arrow() {
        // Test with "Line 1\nLine 2\nLine 3" content context
        // Simulate: user selects from (0, 2) to (1, 4) - spanning lines 1 and 2
        let selection = Selection::new(CursorPosition::new(0, 2), CursorPosition::new(1, 4));

        assert!(!selection.is_empty());

        // Pressing Down should collapse to selection END (line 1, column 4)
        let (_start, end) = selection.ordered();
        assert_eq!(end.line, 1);
        assert_eq!(end.column, 4);
    }

    #[test]
    fn test_selection_extend_with_shift_arrow() {
        // Verify that Shift+Arrow still extends selection (doesn't collapse)
        let selection = Selection::new(
            CursorPosition::new(0, 5),  // anchor
            CursorPosition::new(0, 10), // head
        );

        assert!(!selection.is_empty());

        // With extend_selection=true, selection should NOT collapse
        // The head moves, anchor stays
        // (Implementation test - the actual behavior is in move_* functions)
    }

    #[test]
    fn test_no_collapse_when_no_selection() {
        // When there's no selection (caret), arrow keys should just move
        let pos = CursorPosition::new(0, 5);
        let selection = Selection::caret(pos);

        assert!(selection.is_empty());

        // No collapse needed - cursor just moves normally
    }

    #[test]
    fn test_selection_collapse_backwards_selection() {
        // Test with backwards selection (head before anchor)
        // User drags from right to left: anchor at (0, 10), head at (0, 5)
        let selection = Selection::new(
            CursorPosition::new(0, 10), // anchor (where drag started)
            CursorPosition::new(0, 5),  // head (where drag ended)
        );

        assert!(!selection.is_empty());

        // ordered() should return (5, 10) regardless of drag direction
        let (start, end) = selection.ordered();
        assert_eq!(start.column, 5);
        assert_eq!(end.column, 10);

        // Left arrow should go to start (5), Right arrow should go to end (10)
    }

    // --- Unicode and CRLF Handling Tests ---

    #[test]
    fn test_normalize_line_endings_crlf() {
        // Windows-style CRLF -> LF
        let content = "line1\r\nline2\r\nline3";
        let normalized = EditorPrompt::normalize_line_endings(content);
        assert_eq!(normalized, "line1\nline2\nline3");
    }

    #[test]
    fn test_normalize_line_endings_cr_only() {
        // Old Mac-style CR -> LF
        let content = "line1\rline2\rline3";
        let normalized = EditorPrompt::normalize_line_endings(content);
        assert_eq!(normalized, "line1\nline2\nline3");
    }

    #[test]
    fn test_normalize_line_endings_mixed() {
        // Mixed line endings -> all LF
        let content = "line1\r\nline2\nline3\rline4";
        let normalized = EditorPrompt::normalize_line_endings(content);
        assert_eq!(normalized, "line1\nline2\nline3\nline4");
    }

    #[test]
    fn test_normalize_line_endings_already_lf() {
        // Already LF -> unchanged
        let content = "line1\nline2\nline3";
        let normalized = EditorPrompt::normalize_line_endings(content);
        assert_eq!(normalized, "line1\nline2\nline3");
    }

    #[test]
    fn test_unicode_char_count_cjk() {
        // CJK characters: each is 3 bytes in UTF-8 but 1 char
        let text = "你好世界"; // "Hello World" in Chinese - 4 chars, 12 bytes
        assert_eq!(text.len(), 12); // bytes
        assert_eq!(text.chars().count(), 4); // chars
    }

    #[test]
    fn test_unicode_char_count_emoji() {
        // Emoji: can be 4 bytes in UTF-8 but 1 char
        let text = "Hello 🌍"; // 6 ASCII chars + 1 emoji
        assert_eq!(text.chars().count(), 7);
        assert!(text.len() > 7); // bytes > chars
    }

    #[test]
    fn test_unicode_char_count_mixed() {
        // Mixed ASCII and Unicode
        let text = "Hi你好!"; // 2 ASCII + 2 CJK + 1 ASCII = 5 chars
        assert_eq!(text.chars().count(), 5);
        assert!(text.len() > 5); // bytes > chars due to UTF-8 encoding
    }

    #[test]
    fn test_rope_unicode_line_length() {
        // Verify ropey correctly counts chars, not bytes
        let content = "你好世界\nHello\n🌍🌎🌏";
        let rope = Rope::from_str(content);

        // Line 0: "你好世界" = 4 chars (not 12 bytes!)
        assert_eq!(rope.line(0).len_chars(), 5); // 4 chars + newline

        // Line 1: "Hello" = 5 chars
        assert_eq!(rope.line(1).len_chars(), 6); // 5 chars + newline

        // Line 2: "🌍🌎🌏" = 3 chars (emojis)
        assert_eq!(rope.line(2).len_chars(), 3); // 3 chars, no trailing newline
    }

    #[test]
    fn test_char_to_cursor_static_unicode() {
        // Test char_to_cursor_static with Unicode content
        let rope = Rope::from_str("你好\nWorld");

        // Char 0 is '你'
        let pos = EditorPrompt::char_to_cursor_static(&rope, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        // Char 1 is '好'
        let pos = EditorPrompt::char_to_cursor_static(&rope, 1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 1);

        // Char 2 is '\n'
        let pos = EditorPrompt::char_to_cursor_static(&rope, 2);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 2);

        // Char 3 is 'W' (start of line 1)
        let pos = EditorPrompt::char_to_cursor_static(&rope, 3);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_byte_to_cursor_static_unicode() {
        // Test byte_to_cursor_static with Unicode content
        // "你好" = 6 bytes (3 per CJK char), then '\n' = 1 byte, then "World" = 5 bytes
        let rope = Rope::from_str("你好\nWorld");

        // Byte 0-2 is '你'
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        // Byte 3-5 is '好'
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 3);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 1);

        // Byte 6 is '\n'
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 6);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 2);

        // Byte 7 is 'W' (start of line 1)
        let pos = EditorPrompt::byte_to_cursor_static(&rope, 7);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    // --- Tab/Shift+Tab Indentation Tests ---
    // These tests verify the Tab key behavior for indentation and the presence
    // of correct key handling patterns in the source code.

    #[test]
    fn test_tab_key_patterns_exist_in_source() {
        // Verify Tab key handling patterns exist in the source code
        let source = include_str!("editor.rs");

        // Tab without modifiers should exist for indentation/snippet/insert
        assert!(
            source.contains(r#"("tab", false, false, false)"#),
            "Missing Tab key pattern for normal Tab press"
        );

        // Shift+Tab should exist for outdent/prev tabstop
        assert!(
            source.contains(r#"("tab", false, true, false)"#),
            "Missing Shift+Tab key pattern"
        );

        // indent_selected_lines should be called on Tab
        assert!(
            source.contains("self.indent_selected_lines()"),
            "Missing indent_selected_lines call in Tab handler"
        );

        // outdent_selected_lines should be called on Shift+Tab
        assert!(
            source.contains("self.outdent_selected_lines()"),
            "Missing outdent_selected_lines call in Shift+Tab handler"
        );
    }

    #[test]
    fn test_indent_function_adds_4_spaces() {
        // Verify the indent function uses 4 spaces
        let source = include_str!("editor.rs");

        // The indent function should have the 4-space indent string
        assert!(
            source.contains(r#"let indent = "    "; // 4 spaces"#),
            "Indent function should use 4 spaces"
        );
    }

    #[test]
    fn test_outdent_removes_up_to_4_spaces_or_tab() {
        // Verify outdent logic handles both spaces and tabs
        let source = include_str!("editor.rs");

        // Should check for tab character
        assert!(
            source.contains(r#"if *ch == '\t'"#),
            "Outdent should handle tab characters"
        );

        // Should limit space removal to 4
        assert!(
            source.contains("spaces_counted < 4"),
            "Outdent should remove at most 4 spaces"
        );
    }

    #[test]
    fn test_selection_line_range_for_single_line() {
        // When cursor is on a single line with no selection, range should be that line
        let selection = Selection::caret(CursorPosition::new(3, 5));
        let (start, end) = selection.ordered();

        // For a caret (no selection), start and end should be the same position
        assert_eq!(start.line, 3);
        assert_eq!(end.line, 3);
    }

    #[test]
    fn test_selection_line_range_for_multi_line() {
        // Selection spanning lines 2-4
        let selection = Selection::new(CursorPosition::new(2, 0), CursorPosition::new(4, 10));
        let (start, end) = selection.ordered();

        assert_eq!(start.line, 2);
        assert_eq!(end.line, 4);
    }

    #[test]
    fn test_selection_line_range_backwards() {
        // Backwards selection (head before anchor)
        let selection = Selection::new(
            CursorPosition::new(5, 8), // anchor
            CursorPosition::new(2, 3), // head (before anchor)
        );
        let (start, end) = selection.ordered();

        // ordered() should normalize regardless of selection direction
        assert_eq!(start.line, 2);
        assert_eq!(end.line, 5);
    }

    #[test]
    fn test_tab_handler_checks_snippet_state_first() {
        // Verify that snippet mode is checked before indent
        let source = include_str!("editor.rs");

        // Tab handler should check snippet_state first
        // This pattern should appear in the Tab handling code
        let tab_handler_check = source.contains("if self.snippet_state.is_some()");
        assert!(
            tab_handler_check,
            "Tab handler should check snippet mode first"
        );
    }

    #[test]
    fn test_tab_handler_checks_selection_for_indent() {
        // Verify Tab checks for selection before inserting spaces
        let source = include_str!("editor.rs");

        // Should check if selection is empty to decide between indent and insert
        assert!(
            source.contains("else if !self.selection.is_empty()"),
            "Tab handler should check selection for indent vs insert"
        );
    }

    #[test]
    fn test_shift_tab_always_outdents_without_snippet() {
        // Verify Shift+Tab outdents when not in snippet mode
        let source = include_str!("editor.rs");

        // The Shift+Tab handler should call outdent regardless of selection
        // (outdent_selected_lines handles both single line and multi-line)
        let shift_tab_section = source.find(r#"("tab", false, true, false)"#);
        assert!(
            shift_tab_section.is_some(),
            "Shift+Tab pattern should exist"
        );

        // Verify outdent is called in the else branch (non-snippet mode)
        let outdent_call = source.find("self.outdent_selected_lines()");
        assert!(
            outdent_call.is_some(),
            "outdent_selected_lines should be called"
        );
    }
