#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tabstop() {
        let snippet = ParsedSnippet::parse("$1");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 1),
            _ => panic!("Expected Tabstop"),
        }
        assert_eq!(snippet.text, "");
    }

    #[test]
    fn test_parse_tabstop_with_placeholder() {
        let snippet = ParsedSnippet::parse("${1:name}");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Tabstop {
                index, placeholder, ..
            } => {
                assert_eq!(*index, 1);
                assert_eq!(placeholder.as_deref(), Some("name"));
            }
            _ => panic!("Expected Tabstop"),
        }
        assert_eq!(snippet.text, "name");
    }

    #[test]
    fn test_parse_tabstop_with_choices() {
        let snippet = ParsedSnippet::parse("${1|a,b,c|}");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Tabstop { index, choices, .. } => {
                assert_eq!(*index, 1);
                assert_eq!(
                    choices.as_ref().unwrap(),
                    &vec!["a".to_string(), "b".to_string(), "c".to_string()]
                );
            }
            _ => panic!("Expected Tabstop"),
        }
        // First choice is used as expanded text
        assert_eq!(snippet.text, "a");
    }

    #[test]
    fn test_parse_text_and_tabstop() {
        let snippet = ParsedSnippet::parse("Hello $1!");
        assert_eq!(snippet.parts.len(), 3);

        match &snippet.parts[0] {
            SnippetPart::Text(t) => assert_eq!(t, "Hello "),
            _ => panic!("Expected Text"),
        }
        match &snippet.parts[1] {
            SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 1),
            _ => panic!("Expected Tabstop"),
        }
        match &snippet.parts[2] {
            SnippetPart::Text(t) => assert_eq!(t, "!"),
            _ => panic!("Expected Text"),
        }

        assert_eq!(snippet.text, "Hello !");
    }

    #[test]
    fn test_parse_escaped_dollar() {
        let snippet = ParsedSnippet::parse("$$100");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Text(t) => assert_eq!(t, "$100"),
            _ => panic!("Expected Text"),
        }
        assert_eq!(snippet.text, "$100");
    }

    #[test]
    fn test_parse_linked_tabstops() {
        let snippet = ParsedSnippet::parse("${1:foo} and ${1:bar}");

        // Should have 3 parts: tabstop, text, tabstop
        assert_eq!(snippet.parts.len(), 3);

        // Both tabstops should have index 1
        let tabstop1 = &snippet.parts[0];
        let tabstop2 = &snippet.parts[2];

        match (tabstop1, tabstop2) {
            (
                SnippetPart::Tabstop {
                    index: idx1,
                    placeholder: p1,
                    ..
                },
                SnippetPart::Tabstop {
                    index: idx2,
                    placeholder: p2,
                    ..
                },
            ) => {
                assert_eq!(*idx1, 1);
                assert_eq!(*idx2, 1);
                assert_eq!(p1.as_deref(), Some("foo"));
                assert_eq!(p2.as_deref(), Some("bar"));
            }
            _ => panic!("Expected two Tabstops"),
        }

        // Should only have one TabstopInfo with two ranges
        assert_eq!(snippet.tabstops.len(), 1);
        assert_eq!(snippet.tabstops[0].index, 1);
        assert_eq!(snippet.tabstops[0].ranges.len(), 2);
        // First placeholder should be kept
        assert_eq!(snippet.tabstops[0].placeholder.as_deref(), Some("foo"));
    }

    #[test]
    fn test_parse_final_cursor() {
        let snippet = ParsedSnippet::parse("$0");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Tabstop { index, .. } => assert_eq!(*index, 0),
            _ => panic!("Expected Tabstop"),
        }
    }

    #[test]
    fn test_parse_empty_string() {
        let snippet = ParsedSnippet::parse("");
        assert_eq!(snippet.parts.len(), 0);
        assert_eq!(snippet.text, "");
        assert_eq!(snippet.tabstops.len(), 0);
    }

    #[test]
    fn test_tabstop_order() {
        let snippet = ParsedSnippet::parse("$3 $1 $2 $0");
        let order = snippet.tabstop_order();
        // Should be sorted: 1, 2, 3, then 0 at end
        assert_eq!(order, vec![1, 2, 3, 0]);
    }

    #[test]
    fn test_get_tabstop() {
        let snippet = ParsedSnippet::parse("${1:hello} ${2:world}");

        let t1 = snippet.get_tabstop(1).unwrap();
        assert_eq!(t1.index, 1);
        assert_eq!(t1.placeholder.as_deref(), Some("hello"));

        let t2 = snippet.get_tabstop(2).unwrap();
        assert_eq!(t2.index, 2);
        assert_eq!(t2.placeholder.as_deref(), Some("world"));

        assert!(snippet.get_tabstop(3).is_none());
    }

    #[test]
    fn test_tabstop_ranges() {
        let snippet = ParsedSnippet::parse("Hello ${1:world}!");

        // "Hello " is 6 chars, "world" is 5 chars
        // Range should be (6, 11)
        let t1 = snippet.get_tabstop(1).unwrap();
        assert_eq!(t1.ranges, vec![(6, 11)]);
    }

    #[test]
    fn test_multiple_tabstops_with_text() {
        let snippet = ParsedSnippet::parse("function ${1:name}(${2:args}) { $0 }");

        assert_eq!(snippet.text, "function name(args) {  }");

        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn test_simple_braced_tabstop() {
        let snippet = ParsedSnippet::parse("${1}");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Tabstop {
                index, placeholder, ..
            } => {
                assert_eq!(*index, 1);
                assert!(placeholder.is_none());
            }
            _ => panic!("Expected Tabstop"),
        }
    }

    #[test]
    fn test_lone_dollar_preserved() {
        let snippet = ParsedSnippet::parse("$x");
        // $ followed by non-digit/non-brace should be preserved
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Text(t) => assert_eq!(t, "$x"),
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_dollar_at_end() {
        let snippet = ParsedSnippet::parse("test$");
        assert_eq!(snippet.parts.len(), 1);
        match &snippet.parts[0] {
            SnippetPart::Text(t) => assert_eq!(t, "test$"),
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_parse_placeholder_keeps_escaped_closing_brace_literal() {
        let snippet = ParsedSnippet::parse("${1:foo\\}}");

        assert_eq!(snippet.text, "foo}");

        let t1 = snippet.get_tabstop(1).unwrap();
        assert_eq!(t1.placeholder.as_deref(), Some("foo}"));
        assert_eq!(t1.ranges, vec![(0, 4)]);
    }

    // --- Tests for update_tabstops_after_edit ---

    #[test]
    fn test_update_tabstops_after_insert_first_tabstop() {
        // Template: "${1:hello} ${2:world}"
        // Initial text: "hello world" (char indices)
        // Tabstop 1 at (0, 5), Tabstop 2 at (6, 11)
        //
        // If we type "XX" at position 0 (replacing "hello" with "XXhello"):
        // - Tabstop 1 should expand from (0, 5) to (0, 7)
        // - Tabstop 2 should shift from (6, 11) to (8, 13)
        let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");

        // Verify initial state
        assert_eq!(snippet.tabstops.len(), 2);
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 5)]);
        assert_eq!(snippet.tabstops[1].ranges, vec![(6, 11)]);

        // Simulate inserting "XX" at position 0, which replaces nothing (old_len=0)
        // edit_start=0, old_len=0, new_len=2
        snippet.update_tabstops_after_edit(0, 0, 0, 2);

        // Tabstop 1 was being edited (contains edit point), should expand
        // Original: (0, 5), +2 chars inserted at start -> still (0, 5+2) = (0, 7)
        // But the current tabstop (0) is the one being edited, so its end expands
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 7)]);
        // Tabstop 2 should shift right by 2
        assert_eq!(snippet.tabstops[1].ranges, vec![(8, 13)]);
    }

    #[test]
    fn test_update_tabstops_after_delete_in_first_tabstop() {
        // Template: "${1:hello} ${2:world}"
        // Initial: Tabstop 1 at (0, 5), Tabstop 2 at (6, 11)
        //
        // If we delete "hel" (positions 0-3), leaving "lo":
        // - Tabstop 1 shrinks from (0, 5) to (0, 2)
        // - Tabstop 2 shifts from (6, 11) to (3, 8)
        let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");

        // Delete 3 chars at position 0 (old_len=3, new_len=0)
        snippet.update_tabstops_after_edit(0, 0, 3, 0);

        // Tabstop 1 shrinks by 3 chars
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 2)]);
        // Tabstop 2 shifts left by 3 chars
        assert_eq!(snippet.tabstops[1].ranges, vec![(3, 8)]);
    }

    #[test]
    fn test_update_tabstops_after_replace_in_first_tabstop() {
        // Template: "${1:hello} ${2:world}"
        // Initial: Tabstop 1 at (0, 5), Tabstop 2 at (6, 11)
        //
        // If we replace "hello" (0-5) with "hi" (delta = 2 - 5 = -3):
        // - Tabstop 1 shrinks from (0, 5) to (0, 2)
        // - Tabstop 2 shifts from (6, 11) to (3, 8)
        let mut snippet = ParsedSnippet::parse("${1:hello} ${2:world}");

        // Replace 5 chars with 2 chars at position 0
        snippet.update_tabstops_after_edit(0, 0, 5, 2);

        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 2)]);
        assert_eq!(snippet.tabstops[1].ranges, vec![(3, 8)]);
    }

    #[test]
    fn test_update_tabstops_no_change_before_edit() {
        // Edits before a tabstop should shift it
        // Template: "prefix ${1:hello}"
        // Initial: Tabstop 1 at (7, 12)
        //
        // If we add "XX" at position 2 (in "prefix"):
        // - Tabstop 1 shifts from (7, 12) to (9, 14)
        let mut snippet = ParsedSnippet::parse("prefix ${1:hello}");

        assert_eq!(snippet.tabstops[0].ranges, vec![(7, 12)]);

        // Insert 2 chars at position 2 (inside "prefix")
        // current_tabstop_idx is irrelevant here since edit is in text, not tabstop
        // But we need to pass it - use a value that won't affect the tabstop
        snippet.update_tabstops_after_edit(usize::MAX, 2, 0, 2);

        // Tabstop 1 shifts right by 2
        assert_eq!(snippet.tabstops[0].ranges, vec![(9, 14)]);
    }

    #[test]
    fn test_update_tabstops_linked_tabstops() {
        // Template: "${1:foo} and ${1:bar}"
        // This creates a single TabstopInfo with multiple ranges
        // Initial ranges: [(0, 3), (8, 11)]
        //
        // If we edit the first occurrence, both should update appropriately
        let mut snippet = ParsedSnippet::parse("${1:foo} and ${1:bar}");

        assert_eq!(snippet.tabstops.len(), 1);
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 3), (8, 11)]);

        // Insert 2 chars at position 0 (start of first range)
        // Current tabstop is 0 (the only one)
        snippet.update_tabstops_after_edit(0, 0, 0, 2);

        // First range expands, second range shifts
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 5), (10, 13)]);
    }

    #[test]
    fn test_update_tabstops_clamps_range_when_delete_exceeds_current_tabstop() {
        // Deleting beyond the edited tabstop should not underflow indices.
        let mut snippet = ParsedSnippet::parse("${1:abc}${2:def}");
        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 3)]);
        assert_eq!(snippet.tabstops[1].ranges, vec![(3, 6)]);

        // Delete five chars from position 0 while editing the first tabstop.
        // This fully removes tabstop 1 and consumes two chars from tabstop 2.
        snippet.update_tabstops_after_edit(0, 0, 5, 0);

        assert_eq!(snippet.tabstops[0].ranges, vec![(0, 0)]);
        assert_eq!(snippet.tabstops[1].ranges, vec![(0, 1)]);
    }

    #[test]
    fn test_choices_with_commas() {
        let snippet = ParsedSnippet::parse("${1|apple,banana,cherry|}");
        match &snippet.parts[0] {
            SnippetPart::Tabstop { choices, .. } => {
                let c = choices.as_ref().unwrap();
                assert_eq!(c.len(), 3);
                assert_eq!(c[0], "apple");
                assert_eq!(c[1], "banana");
                assert_eq!(c[2], "cherry");
            }
            _ => panic!("Expected Tabstop"),
        }
    }

    #[test]
    fn test_complex_template() {
        let template = r#"import { ${1:Component} } from '${2:react}';

export default function ${1:Component}() {
    return (
        <div>$0</div>
    );
}"#;

        let snippet = ParsedSnippet::parse(template);

        // Should have Component tabstop (index 1) twice
        let t1 = snippet.get_tabstop(1).unwrap();
        assert_eq!(t1.ranges.len(), 2);
        assert_eq!(t1.placeholder.as_deref(), Some("Component"));

        // Should have react tabstop (index 2) once
        let t2 = snippet.get_tabstop(2).unwrap();
        assert_eq!(t2.ranges.len(), 1);
        assert_eq!(t2.placeholder.as_deref(), Some("react"));

        // Order should be 1, 2, 0
        assert_eq!(snippet.tabstop_order(), vec![1, 2, 0]);
    }

    // =========================================================================
    // $0 (final cursor) edge case tests - CRITICAL for correct positioning
    // =========================================================================

    #[test]
    fn test_final_cursor_at_end_of_text() {
        // Common pattern: $0 at the very end
        let snippet = ParsedSnippet::parse("Hello ${1:name}!$0");

        // The expanded text should be "Hello name!"
        assert_eq!(snippet.text, "Hello name!");
        assert_eq!(snippet.text.chars().count(), 11);

        // $0 should be at position (11, 11) - a zero-length range at the end
        let t0 = snippet.get_tabstop(0).unwrap();
        assert_eq!(t0.ranges, vec![(11, 11)], "$0 should be at end of text");

        // Verify $0 is last in navigation order
        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 0]);
        assert_eq!(order.last(), Some(&0), "$0 must be last");
    }

    #[test]
    fn test_final_cursor_empty_range() {
        // $0 without placeholder has zero-length range
        let snippet = ParsedSnippet::parse("$0");

        let t0 = snippet.get_tabstop(0).unwrap();
        assert_eq!(t0.ranges, vec![(0, 0)], "$0 should have zero-length range");
        assert!(t0.placeholder.is_none());
    }

    #[test]
    fn test_final_cursor_with_placeholder() {
        // ${0:done} - $0 with placeholder text
        let snippet = ParsedSnippet::parse("${1:hello} ${0:cursor here}");

        assert_eq!(snippet.text, "hello cursor here");

        let t0 = snippet.get_tabstop(0).unwrap();
        assert_eq!(t0.placeholder.as_deref(), Some("cursor here"));
        // Range should span "cursor here" (6, 17)
        assert_eq!(t0.ranges, vec![(6, 17)]);
    }

    #[test]
    fn test_multiple_tabstops_then_final_cursor() {
        // Real-world pattern: function template
        let snippet = ParsedSnippet::parse("fn ${1:name}(${2:args}) { $0 }");

        assert_eq!(snippet.text, "fn name(args) {  }");

        // Verify tabstop order: 1, 2, then 0
        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 2, 0]);

        // $0 should be at position between "{" and "}"
        let t0 = snippet.get_tabstop(0).unwrap();
        // "fn name(args) { " = 16 chars, then $0 is at (16, 16)
        assert_eq!(t0.ranges, vec![(16, 16)]);
    }

    #[test]
    fn test_only_final_cursor() {
        // Edge case: only $0 in template
        let snippet = ParsedSnippet::parse("$0");

        assert_eq!(snippet.tabstops.len(), 1);
        assert_eq!(snippet.tabstop_order(), vec![0]);
    }

    #[test]
    fn test_parse_merges_all_ranges_when_template_contains_multiple_final_cursors() {
        let snippet = ParsedSnippet::parse("$0 and $0");

        assert_eq!(snippet.text, " and ");
        assert_eq!(snippet.tabstop_order(), vec![0]);

        let t0 = snippet.get_tabstop(0).unwrap();
        assert_eq!(t0.ranges, vec![(0, 0), (5, 5)]);
    }

    #[test]
    fn test_final_cursor_unicode() {
        // $0 after unicode text - uses char count, not byte count
        let snippet = ParsedSnippet::parse("你好$0");

        assert_eq!(snippet.text, "你好");
        assert_eq!(snippet.text.chars().count(), 2); // 2 chars
        assert_eq!(snippet.text.len(), 6); // 6 bytes

        let t0 = snippet.get_tabstop(0).unwrap();
        // IMPORTANT: ranges use CHAR indices (2), not byte indices (6)
        assert_eq!(t0.ranges, vec![(2, 2)]);
    }

    #[test]
    fn test_tabstop_navigation_order_with_gaps() {
        // Tabstop numbers can have gaps: $1, $3, $5, $0
        let snippet = ParsedSnippet::parse("$5 $1 $3 $0");

        // Order should sort numerically: 1, 3, 5, then 0 at end
        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 3, 5, 0]);
    }

    #[test]
    fn test_tabstop_navigation_order_reverse_in_template() {
        // Template has tabstops in reverse order
        let snippet = ParsedSnippet::parse("$3 $2 $1 $0");

        // Navigation order should still be 1, 2, 3, 0
        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 2, 3, 0]);
    }

    #[test]
    fn test_tabstop_without_zero() {
        // Template without $0 - navigation ends after last numbered tabstop
        let snippet = ParsedSnippet::parse("${1:first} ${2:second}");

        let order = snippet.tabstop_order();
        assert_eq!(order, vec![1, 2]); // No 0
        assert_eq!(snippet.get_tabstop(0), None);
    }
}
