use super::*;

#[test]
fn unordered_list_produces_separate_items() {
    let md = "- First item\n- Second item\n- Third item\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("\u{2022}".into(), "First item".into()),
            TestBlock::ListItem("\u{2022}".into(), "Second item".into()),
            TestBlock::ListItem("\u{2022}".into(), "Third item".into()),
        ]
    );
}

#[test]
fn ordered_list_produces_numbered_items() {
    let md = "1. Alpha\n2. Beta\n3. Gamma\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("1.".into(), "Alpha".into()),
            TestBlock::ListItem("2.".into(), "Beta".into()),
            TestBlock::ListItem("3.".into(), "Gamma".into()),
        ]
    );
}

#[test]
fn nested_lists_preserve_parent_child_structure() {
    let md = "1. Parent\n   - Child A\n   - Child B\n2. Next\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("1.".into(), "Parent".into()),
            TestBlock::ListItem("  \u{2022}".into(), "Child A".into()),
            TestBlock::ListItem("  \u{2022}".into(), "Child B".into()),
            TestBlock::ListItem("2.".into(), "Next".into()),
        ]
    );
}

#[test]
fn paragraph_after_list_is_separate_block() {
    let md = "- Item one\n- Item two\n\nParagraph after the list.\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("\u{2022}".into(), "Item one".into()),
            TestBlock::ListItem("\u{2022}".into(), "Item two".into()),
            TestBlock::Paragraph("Paragraph after the list.".into()),
        ]
    );
}

#[test]
fn heading_then_list_then_paragraph() {
    let md = "## My Heading\n\n- Item A\n- Item B\n\nSome text.\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::Heading(2, "My Heading".into()),
            TestBlock::ListItem("\u{2022}".into(), "Item A".into()),
            TestBlock::ListItem("\u{2022}".into(), "Item B".into()),
            TestBlock::Paragraph("Some text.".into()),
        ]
    );
}

#[test]
fn list_with_bold_and_inline_code() {
    let md = "- **Bold** item\n- Item with `code`\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("\u{2022}".into(), "Bold item".into()),
            TestBlock::ListItem("\u{2022}".into(), "Item with code".into()),
        ]
    );
}

#[test]
fn code_block_after_list() {
    let md = "- Item\n\n```rust\nfn main() {}\n```\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("\u{2022}".into(), "Item".into()),
            TestBlock::CodeBlock(Some("rust".into()), "fn main() {}\n".into()),
        ]
    );
}

/// Simulates progressive reveal of a markdown string containing a list.
/// Each intermediate revealed substring should parse without panic and
/// the final full string should produce the expected block structure.
#[test]
fn progressive_reveal_of_list_parses_at_every_boundary() {
    use crate::prompts::chat::chat_tests::next_reveal_boundary_pub;

    let content = "Here's a list:\n\n- First item\n- Second item\n- Third item\n\nDone!\n";
    let mut offset = 0;

    // Reveal word-by-word / line-by-line
    while let Some(new_offset) = next_reveal_boundary_pub(content, offset) {
        if new_offset <= offset {
            break;
        }
        let partial = &content[..new_offset];
        // Should not panic
        let _ = parse_markdown_blocks(partial);
        offset = new_offset;
    }

    // Final flush
    let blocks = parse_markdown_blocks(content);
    assert_eq!(
        blocks,
        vec![
            TestBlock::Paragraph("Here\u{2019}s a list:".into()),
            TestBlock::ListItem("\u{2022}".into(), "First item".into()),
            TestBlock::ListItem("\u{2022}".into(), "Second item".into()),
            TestBlock::ListItem("\u{2022}".into(), "Third item".into()),
            TestBlock::Paragraph("Done!".into()),
        ]
    );
}

#[test]
fn horizontal_rule_between_sections() {
    let md = "Before\n\n---\n\nAfter\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::Paragraph("Before".into()),
            TestBlock::Hr,
            TestBlock::Paragraph("After".into()),
        ]
    );
}

#[test]
fn empty_string_produces_no_blocks() {
    assert_eq!(parse_markdown_blocks(""), vec![]);
}

#[test]
fn single_paragraph() {
    let blocks = parse_markdown_blocks("Hello world.\n");
    assert_eq!(blocks, vec![TestBlock::Paragraph("Hello world.".into())]);
}

#[test]
fn task_list_renders_checkboxes() {
    let md = "- [x] Done task\n- [ ] Pending task\n- Regular item\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![
            TestBlock::ListItem("\u{2611}".into(), "Done task".into()),
            TestBlock::ListItem("\u{2610}".into(), "Pending task".into()),
            TestBlock::ListItem("\u{2022}".into(), "Regular item".into()),
        ]
    );
}

#[test]
fn simple_table_parses_headers_and_rows() {
    let md = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![TestBlock::Table(
            vec!["Name".into(), "Age".into()],
            vec![
                vec!["Alice".into(), "30".into()],
                vec!["Bob".into(), "25".into()],
            ]
        )]
    );
}

#[test]
fn hard_break_preserves_line_break() {
    let md = "line one  \nline two\n";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(
        blocks,
        vec![TestBlock::Paragraph("line one\nline two".into())]
    );
}

#[test]
fn markdown_image_preserves_alt_text_and_url() {
    let md = "![diagram](https://example.com/diagram.png)\n";
    let blocks = parse_markdown(md, true);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        ParsedBlock::Paragraph { spans, .. } => {
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].text, "[Image: diagram]");
            assert!(spans[0].style.link);
            assert_eq!(
                spans[0].link_url.as_deref(),
                Some("https://example.com/diagram.png")
            );
        }
        other => panic!("expected paragraph block, got: {other:?}"),
    }
}

#[test]
fn markdown_link_url_allowlist_rejects_unsafe_schemes() {
    assert!(is_allowed_markdown_url("https://example.com"));
    assert!(is_allowed_markdown_url("http://example.com"));
    assert!(is_allowed_markdown_url("mailto:test@example.com"));
    assert!(is_allowed_markdown_url("relative/path"));
    assert!(!is_allowed_markdown_url("file:///tmp/secrets.txt"));
    assert!(!is_allowed_markdown_url("javascript:alert(1)"));
    assert!(!is_allowed_markdown_url("data:text/html,hello"));
}

#[test]
fn markdown_scope_hash_is_deterministic_for_same_scope() {
    let first = stable_markdown_scope_hash(Some("assistant-msg-123"));
    let second = stable_markdown_scope_hash(Some("assistant-msg-123"));
    let other = stable_markdown_scope_hash(Some("assistant-msg-456"));

    assert_eq!(first, second);
    assert_ne!(first, other);
}

#[test]
fn scoped_markdown_element_id_is_stable_and_indexed() {
    let scope_hash = stable_markdown_scope_hash(Some("assistant-msg-123"));
    let block_a = scoped_markdown_element_id(scope_hash, "block", 7, 0);
    let block_a_again = scoped_markdown_element_id(scope_hash, "block", 7, 0);
    let block_b = scoped_markdown_element_id(scope_hash, "block", 8, 0);

    assert_eq!(block_a, block_a_again);
    assert_ne!(block_a, block_b);
}

#[test]
fn inferred_scope_hash_stays_stable_for_appended_content_after_prefix_window() {
    let stable_prefix = "a".repeat(INFERRED_SCOPE_PREFIX_CHARS + 32);
    let baseline = format!("{stable_prefix}\n\n- item 1");
    let appended = format!("{baseline}\n- item 2\n- item 3");

    assert_eq!(
        inferred_markdown_scope_hash(&baseline),
        inferred_markdown_scope_hash(&appended),
        "Appended tail content should not change inferred scope hash once prefix window is filled",
    );
}
