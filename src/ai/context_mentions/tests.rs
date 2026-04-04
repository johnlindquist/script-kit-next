use super::*;

#[test]
fn parse_context_mentions_extracts_resource_directives_and_keeps_body() {
    let parsed = parse_context_mentions("@selection\n@browser\nCompare these.");

    assert_eq!(parsed.cleaned_content, "Compare these.");
    assert_eq!(parsed.parts.len(), 2);
    assert_eq!(
        parsed.parts[0],
        AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
                    .to_string(),
            label: "Selection".to_string(),
        }
    );
    assert_eq!(
        parsed.parts[1],
        AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                    .to_string(),
            label: "Browser URL".to_string(),
        }
    );
}

#[test]
fn parse_context_mentions_extracts_file_directive() {
    let parsed = parse_context_mentions("@file /tmp/demo.rs\nRefactor this.");

    assert_eq!(parsed.cleaned_content, "Refactor this.");
    assert_eq!(
        parsed.parts,
        vec![AiContextPart::FilePath {
            path: "/tmp/demo.rs".to_string(),
            label: "demo.rs".to_string(),
        }]
    );
}

#[test]
fn parse_context_mentions_keeps_unknown_at_lines_as_content() {
    let parsed = parse_context_mentions("@unknown\nKeep this.");

    assert_eq!(parsed.cleaned_content, "@unknown\nKeep this.");
    assert!(parsed.parts.is_empty());
}

#[test]
fn parse_context_mentions_allows_directive_only_messages() {
    let parsed = parse_context_mentions("@context\n@selection");

    assert_eq!(parsed.cleaned_content, "");
    assert_eq!(parsed.parts.len(), 2);
}

#[test]
fn parse_context_mentions_handles_all_resource_directives() {
    let input = "@context\n@context-full\n@selection\n@browser\n@window\n@diagnostics";
    let parsed = parse_context_mentions(input);

    assert_eq!(parsed.cleaned_content, "");
    assert_eq!(parsed.parts.len(), 6);
    assert_eq!(parsed.parts[0].label(), "Current Context");
    assert_eq!(parsed.parts[1].label(), "Current Context (Full)");
    assert_eq!(parsed.parts[2].label(), "Selection");
    assert_eq!(parsed.parts[3].label(), "Browser URL");
    assert_eq!(parsed.parts[4].label(), "Focused Window");
    assert_eq!(parsed.parts[5].label(), "Context Diagnostics");
}

#[test]
fn parse_context_mentions_preserves_body_ordering() {
    let parsed = parse_context_mentions("Line one.\n@context\nLine two.\n@selection\nLine three.");

    assert_eq!(parsed.cleaned_content, "Line one.\nLine two.\nLine three.");
    assert_eq!(parsed.parts.len(), 2);
}

#[test]
fn parse_context_mentions_handles_file_with_tab_separator() {
    let parsed = parse_context_mentions("@file\t/tmp/test.txt");

    assert_eq!(
        parsed.parts,
        vec![AiContextPart::FilePath {
            path: "/tmp/test.txt".to_string(),
            label: "test.txt".to_string(),
        }]
    );
}

#[test]
fn parse_context_mentions_ignores_empty_file_path() {
    let parsed = parse_context_mentions("@file ");

    assert!(parsed.parts.is_empty());
    assert_eq!(parsed.cleaned_content, "@file ");
}

#[test]
fn parse_context_mentions_has_parts_helper() {
    let empty = parse_context_mentions("Just text.");
    assert!(!empty.has_parts());

    let with_parts = parse_context_mentions("@context\nText.");
    assert!(with_parts.has_parts());
}

// ── @file: colon-prefix parsing ────────────────────────────────

#[test]
fn parse_context_mentions_extracts_file_colon_directive() {
    let parsed = parse_context_mentions("@file:/tmp/demo.rs\nRefactor this.");

    assert_eq!(parsed.cleaned_content, "Refactor this.");
    assert_eq!(
        parsed.parts,
        vec![AiContextPart::FilePath {
            path: "/tmp/demo.rs".to_string(),
            label: "demo.rs".to_string(),
        }]
    );
}

// ── Inline mention parsing ─────────────────────────────────────

#[test]
fn parse_inline_context_mentions_finds_builtins() {
    let mentions = parse_inline_context_mentions("Fix @browser and @git-status");
    assert_eq!(mentions.len(), 2);
    assert_eq!(mentions[0].token, "@browser");
    assert_eq!(mentions[0].range, 4..12);
    assert_eq!(mentions[1].token, "@git-status");
}

#[test]
fn parse_inline_context_mentions_finds_file_colon() {
    let mentions = parse_inline_context_mentions("Check @file:/tmp/demo.rs please");
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].token, "@file:/tmp/demo.rs");
    assert_eq!(
        mentions[0].part,
        AiContextPart::FilePath {
            path: "/tmp/demo.rs".to_string(),
            label: "demo.rs".to_string(),
        }
    );
}

#[test]
fn parse_inline_context_mentions_ignores_unknown() {
    let mentions = parse_inline_context_mentions("Hello @unknown world");
    assert!(mentions.is_empty());
}

#[test]
fn parse_inline_context_mentions_requires_word_boundary() {
    let mentions = parse_inline_context_mentions("email@browser.com");
    assert!(mentions.is_empty());
}

// ── part_to_inline_token round-trip ────────────────────────────

#[test]
fn part_to_inline_token_roundtrips_builtin() {
    let part = crate::ai::context_contract::ContextAttachmentKind::Browser.part();
    let token = part_to_inline_token(&part);
    assert_eq!(token, Some("@browser".to_string()));
}

#[test]
fn part_to_inline_token_roundtrips_file() {
    let part = AiContextPart::FilePath {
        path: "/tmp/demo.rs".to_string(),
        label: "demo.rs".to_string(),
    };
    let token = part_to_inline_token(&part);
    assert_eq!(token, Some("@file:/tmp/demo.rs".to_string()));
}

#[test]
fn part_to_inline_token_returns_none_for_ambient() {
    let part = AiContextPart::AmbientContext {
        label: "test".to_string(),
    };
    assert_eq!(part_to_inline_token(&part), None);
}
