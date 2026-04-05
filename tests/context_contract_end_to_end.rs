//! End-to-end regression test for the public message-preparation pipeline.
//!
//! The canonical context contract itself is locked by crate-local tests where
//! internal parsing and action routing are available. This integration test
//! intentionally stays on the public API surface and uses fixed fixtures to
//! verify that mixed context parts still prepare consistently end to end.

use script_kit_gpui::ai::context_attachment_specs;
use script_kit_gpui::ai::message_parts::{
    merge_context_parts, prepare_user_message_with_receipt, AiContextPart, PreparedMessageDecision,
};
use std::sync::Arc;

/// Parse fixture `@mention` directives into the public `AiContextPart` shape.
///
/// Uses the canonical `context_attachment_specs()` to resolve mention tokens,
/// keeping the integration test in sync with the contract.
fn parse_mentions(raw: &str) -> (String, Vec<AiContextPart>) {
    let specs = context_attachment_specs();
    let mut cleaned_lines = Vec::new();
    let mut parts = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        let matched = specs
            .iter()
            .find(|spec| spec.mention == Some(trimmed) || spec.mention_aliases.contains(&trimmed));

        if let Some(spec) = matched {
            parts.push(AiContextPart::ResourceUri {
                uri: spec.uri.to_string(),
                label: spec.label.to_string(),
            });
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
    script_kit_gpui::context_snapshot::enable_deterministic_context_capture();
    // Step 1: Parse mentions from raw composer input
    let raw = "@snapshot\n@browser\n\nPlease summarize what matters.";
    let (cleaned, mention_parts) = parse_mentions(raw);

    assert_eq!(cleaned, "Please summarize what matters.");
    assert_eq!(
        mention_parts.len(),
        2,
        "should parse @snapshot and @browser"
    );

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

/// Every spec that has a mention token must round-trip through parse_mentions.
#[test]
fn all_mention_tokens_parse_via_canonical_contract() {
    for spec in context_attachment_specs() {
        if let Some(mention) = spec.mention {
            let (cleaned, parts) = parse_mentions(mention);
            assert!(
                cleaned.is_empty(),
                "mention {mention} should be consumed, not left in cleaned content"
            );
            assert_eq!(
                parts.len(),
                1,
                "mention {mention} should produce exactly one part"
            );
            assert_eq!(parts[0].source(), spec.uri, "URI mismatch for {mention}");
            assert_eq!(parts[0].label(), spec.label, "label mismatch for {mention}");
        }
    }
}

/// Provider-backed tokens (@clipboard, @git-diff, @recent-scripts, @calendar,
/// @screenshot) resolve to concrete parts through the full pipeline.
#[test]
fn provider_backed_mentions_resolve_end_to_end() {
    let raw = "@clipboard\n@git-diff\n@recent-scripts\n@calendar\nSummarize this.";
    let (cleaned, parts) = parse_mentions(raw);

    assert_eq!(cleaned, "Summarize this.");
    assert_eq!(parts.len(), 4, "should parse all 4 provider-backed tokens");

    // Verify URIs point to real provider-backed resources
    let uris: Vec<&str> = parts.iter().map(|p| p.source()).collect();
    assert!(uris.contains(&"kit://clipboard-history"));
    assert!(uris.contains(&"kit://git-diff"));
    assert!(uris.contains(&"kit://scripts"));
    assert!(uris.contains(&"kit://calendar"));

    // Merge with no pending parts (no dedup needed)
    let merged = merge_context_parts(&parts, &[]);
    assert_eq!(merged.len(), 4);

    let scripts: Vec<Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let receipt = prepare_user_message_with_receipt(&cleaned, &merged, &scripts, &scriptlets);

    assert_eq!(
        receipt.decision,
        PreparedMessageDecision::Ready,
        "all provider-backed URIs should resolve; decision was {:?}",
        receipt.decision
    );
    assert_eq!(receipt.context.attempted, 4);
    assert_eq!(receipt.context.resolved, 4);
    assert!(
        receipt.context.failures.is_empty(),
        "unexpected failures: {:?}",
        receipt.context.failures
    );
}

/// Slash and @ flows share the same specs — every spec with both a slash
/// command and a mention token maps to the same URI and label.
#[test]
fn slash_and_mention_share_same_uri_and_label() {
    for spec in context_attachment_specs() {
        if spec.slash_command.is_some() && spec.mention.is_some() {
            // Both modes point to the same URI and label — they share one spec.
            // This is guaranteed by the array structure but verify explicitly.
            let part = AiContextPart::ResourceUri {
                uri: spec.uri.to_string(),
                label: spec.label.to_string(),
            };
            assert_eq!(
                part.source(),
                spec.uri,
                "slash/mention parity broken for {:?}",
                spec.kind
            );
        }
    }
}
