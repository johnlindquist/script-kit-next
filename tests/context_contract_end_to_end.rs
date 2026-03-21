//! End-to-end regression test for the public message-preparation pipeline.
//!
//! The canonical context contract itself is locked by crate-local tests where
//! internal parsing and action routing are available. This integration test
//! intentionally stays on the public API surface and uses fixed fixtures to
//! verify that mixed context parts still prepare consistently end to end.

use script_kit_gpui::ai::message_parts::{
    merge_context_parts, prepare_user_message_with_receipt, AiContextPart, PreparedMessageDecision,
};
use std::sync::Arc;

/// Parse fixture `@mention` directives into the public `AiContextPart` shape.
fn parse_mentions(raw: &str) -> (String, Vec<AiContextPart>) {
    // This mirrors the fixture inputs used in this black-box integration test.
    // Contract-level mapping assertions live in crate-local tests.
    let mut cleaned_lines = Vec::new();
    let mut parts = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        let part = match trimmed {
            "@context" => Some(AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Current Context".to_string(),
            }),
            "@context-full" => Some(AiContextPart::ResourceUri {
                uri: "kit://context".to_string(),
                label: "Current Context (Full)".to_string(),
            }),
            "@selection" => Some(AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0".to_string(),
                label: "Selection".to_string(),
            }),
            "@browser" => Some(AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0".to_string(),
                label: "Browser URL".to_string(),
            }),
            "@window" => Some(AiContextPart::ResourceUri {
                uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1".to_string(),
                label: "Focused Window".to_string(),
            }),
            "@diagnostics" => Some(AiContextPart::ResourceUri {
                uri: "kit://context?diagnostics=1".to_string(),
                label: "Context Diagnostics".to_string(),
            }),
            _ => None,
        };

        if let Some(p) = part {
            parts.push(p);
        } else {
            cleaned_lines.push(line);
        }
    }

    let cleaned = cleaned_lines
        .join("\n")
        .trim_matches(|c| c == '\n' || c == '\r')
        .to_string();

    (cleaned, parts)
}

#[test]
fn explicit_context_surfaces_share_one_contract_end_to_end() {
    // Step 1: Parse mentions from raw composer input
    let raw = "@context\n@browser\n\nPlease summarize what matters.";
    let (cleaned, mention_parts) = parse_mentions(raw);

    assert_eq!(cleaned, "Please summarize what matters.");
    assert_eq!(mention_parts.len(), 2, "should parse @context and @browser");

    // Step 2: Simulate pending context chips (from UI actions)
    let pending = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?diagnostics=1".to_string(),
            label: "Context Diagnostics".to_string(),
        },
    ];

    // Step 3: Merge — dedup should remove the duplicate "current context"
    let merged = merge_context_parts(&mention_parts, &pending);
    assert_eq!(
        merged.len(),
        3,
        "duplicate current context should be removed; got {:?}",
        merged.iter().map(|p| p.label()).collect::<Vec<_>>()
    );

    // Step 4: Prepare the message through the full pipeline
    let scripts: Vec<Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let receipt = prepare_user_message_with_receipt(&cleaned, &merged, &scripts, &scriptlets);

    assert_eq!(
        receipt.decision,
        PreparedMessageDecision::Ready,
        "all kit://context URIs should resolve; decision was {:?}",
        receipt.decision
    );
    assert_eq!(receipt.context.attempted, 3);
    assert_eq!(receipt.context.resolved, 3);
    assert!(
        receipt.context.failures.is_empty(),
        "unexpected failures: {:?}",
        receipt.context.failures
    );

    // Verify the resolved content contains expected URI markers
    assert!(
        receipt
            .final_user_content
            .contains("kit://context?profile=minimal"),
        "final content should contain minimal context URI"
    );
    assert!(
        receipt.final_user_content.contains(
            "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
        ),
        "final content should contain browser context URI"
    );
    assert!(
        receipt
            .final_user_content
            .contains("kit://context?diagnostics=1"),
        "final content should contain diagnostics URI"
    );
    assert!(
        receipt
            .final_user_content
            .ends_with("Please summarize what matters."),
        "final content should end with user text"
    );
}
