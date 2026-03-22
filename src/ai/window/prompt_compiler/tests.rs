use super::model::{
    PromptCompilerDecision, PromptCompilerPreview, PromptCompilerRowKind, PromptCompilerSnapshot,
};
use crate::ai::message_parts::PreparedMessageReceipt;
use serde_json::json;

fn make_receipt(value: serde_json::Value) -> PreparedMessageReceipt {
    serde_json::from_value(value).expect("fixture should deserialize")
}

#[test]
fn ready_receipt_maps_to_ready_decision() {
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "ready",
        "rawContent": "hello world",
        "finalUserContent": "<context source=\"kit://context\" mimeType=\"application/json\">\n{}\n</context>\n\nhello world",
        "context": {
            "attempted": 1,
            "resolved": 1,
            "failures": [],
            "promptPrefix": "<context source=\"kit://context\" mimeType=\"application/json\">\n{}\n</context>"
        },
        "outcomes": [
            {
                "label": "Current Context",
                "source": "kit://context",
                "kind": "fullContent",
                "detail": "mimeType=application/json"
            }
        ]
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    assert_eq!(preview.decision, PromptCompilerDecision::Ready);
    assert_eq!(preview.attempted, 1);
    assert_eq!(preview.resolved, 1);
    assert_eq!(preview.failures, 0);
    assert_eq!(preview.duplicates_removed, 0);
    assert!(preview.final_user_content.ends_with("hello world"));
    assert!(preview.final_user_content.contains("<context"));
    assert_eq!(preview.rows.len(), 1);
    assert_eq!(preview.rows[0].kind, PromptCompilerRowKind::FullContent);

    // Snapshot emits valid JSON
    let snapshot = preview.snapshot();
    let json_str =
        serde_json::to_string_pretty(&snapshot).expect("snapshot should serialize to JSON");
    tracing::info!(snapshot_json = %json_str, "ready_receipt snapshot");
    assert_eq!(snapshot.decision, "Ready");
    assert_eq!(snapshot.row_count, 1);
}

#[test]
fn partial_receipt_surfaces_failures_duplicates_and_unresolved() {
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "partial",
        "rawContent": "explain it",
        "finalUserContent": "<context source=\"kit://context?profile=minimal\" mimeType=\"application/json\">\n{\"app\":\"Finder\"}\n</context>\n\nexplain it",
        "context": {
            "attempted": 2,
            "resolved": 1,
            "failures": [
                {
                    "label": "secret.txt",
                    "source": "/tmp/secret.txt",
                    "error": "permission denied"
                }
            ],
            "promptPrefix": "<context source=\"kit://context?profile=minimal\" mimeType=\"application/json\">\n{\"app\":\"Finder\"}\n</context>"
        },
        "assembly": {
            "mentionCount": 1,
            "pendingCount": 1,
            "mergedCount": 1,
            "duplicatesRemoved": 1,
            "duplicates": [
                {
                    "keptFrom": "mention",
                    "droppedFrom": "pending",
                    "label": "Current Context",
                    "source": "kit://context?profile=minimal"
                }
            ],
            "mergedParts": [
                {
                    "kind": "resourceUri",
                    "uri": "kit://context?profile=minimal",
                    "label": "Current Context"
                }
            ]
        },
        "outcomes": [
            {
                "label": "Current Context",
                "source": "kit://context?profile=minimal",
                "kind": "fullContent",
                "detail": "mimeType=application/json"
            },
            {
                "label": "secret.txt",
                "source": "/tmp/secret.txt",
                "kind": "failed",
                "detail": "permission denied"
            }
        ],
        "unresolvedParts": [
            {
                "kind": "filePath",
                "path": "/tmp/secret.txt",
                "label": "secret.txt"
            }
        ],
        "userError": "Failed to resolve context: secret.txt: permission denied"
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    assert_eq!(preview.decision, PromptCompilerDecision::Partial);
    assert_eq!(preview.resolved, 1);
    assert_eq!(preview.failures, 1);
    assert_eq!(preview.duplicates_removed, 1);
    assert!(preview.final_user_content.ends_with("explain it"));

    // Verify row kinds present
    assert!(preview
        .rows
        .iter()
        .any(|r| r.kind == PromptCompilerRowKind::DuplicateDropped));
    assert!(preview
        .rows
        .iter()
        .any(|r| r.kind == PromptCompilerRowKind::Failed));
    assert!(preview
        .rows
        .iter()
        .any(|r| r.kind == PromptCompilerRowKind::UnresolvedPart));
    assert!(preview
        .rows
        .iter()
        .any(|r| r.kind == PromptCompilerRowKind::FullContent));

    // Total rows: 1 duplicate + 2 outcomes + 1 unresolved = 4
    assert_eq!(preview.rows.len(), 4);

    // Snapshot
    let snapshot = preview.snapshot();
    let json_str =
        serde_json::to_string_pretty(&snapshot).expect("snapshot should serialize to JSON");
    tracing::info!(snapshot_json = %json_str, "partial_receipt snapshot");
    assert_eq!(snapshot.decision, "Partial");
    assert_eq!(snapshot.attempted, 2);
    assert_eq!(snapshot.resolved, 1);
    assert_eq!(snapshot.failures, 1);
    assert_eq!(snapshot.duplicates_removed, 1);
    assert_eq!(snapshot.row_count, 4);
}

#[test]
fn blocked_receipt_maps_to_blocked_decision() {
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "blocked",
        "rawContent": "",
        "finalUserContent": "",
        "context": {
            "attempted": 0,
            "resolved": 0,
            "failures": [],
            "promptPrefix": ""
        },
        "userError": "No content to send"
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    assert_eq!(preview.decision, PromptCompilerDecision::Blocked);
    assert_eq!(preview.attempted, 0);
    assert_eq!(preview.resolved, 0);
    assert_eq!(preview.failures, 0);
    assert_eq!(preview.rows.len(), 0);
    assert_eq!(preview.approx_tokens, 0);

    let snapshot = preview.snapshot();
    let json_str =
        serde_json::to_string_pretty(&snapshot).expect("snapshot should serialize to JSON");
    tracing::info!(snapshot_json = %json_str, "blocked_receipt snapshot");
    assert_eq!(snapshot.decision, "Blocked");
    assert_eq!(snapshot.raw_content_len, 0);
    assert_eq!(snapshot.final_user_content_len, 0);
}

#[test]
fn final_user_content_matches_receipt_payload() {
    let final_content = "<context source=\"kit://context\" mimeType=\"application/json\">\n{\"app\":\"Terminal\"}\n</context>\n\ndo something";
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "ready",
        "rawContent": "do something",
        "finalUserContent": final_content,
        "context": {
            "attempted": 1,
            "resolved": 1,
            "failures": [],
            "promptPrefix": "<context source=\"kit://context\" mimeType=\"application/json\">\n{\"app\":\"Terminal\"}\n</context>"
        },
        "outcomes": [
            {
                "label": "Context",
                "source": "kit://context",
                "kind": "fullContent"
            }
        ]
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    // The preview's final_user_content must exactly match the receipt's
    assert_eq!(preview.final_user_content, receipt.final_user_content);
    assert_eq!(preview.final_user_content, final_content);
}

#[test]
fn snapshot_serializes_to_json() {
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "partial",
        "rawContent": "test",
        "finalUserContent": "prefix\n\ntest",
        "context": {
            "attempted": 1,
            "resolved": 1,
            "failures": [],
            "promptPrefix": "prefix"
        }
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);
    let snapshot = preview.snapshot();

    // Roundtrip via JSON
    let json_val = serde_json::to_value(&snapshot).expect("should serialize");
    assert!(json_val.is_object());
    assert_eq!(json_val["decision"], "Partial");
    assert_eq!(json_val["row_count"], 0);
    assert!(json_val["approx_tokens"].as_u64().unwrap_or(0) > 0);
}

/// Proves that the same receipt produces an identical preview whether it comes
/// from the preflight path or the post-send path. Both paths store the same
/// `PreparedMessageReceipt` type, so `from_receipt` must be deterministic.
#[test]
fn preflight_and_post_send_produce_identical_preview() {
    let receipt_json = json!({
        "schemaVersion": 1,
        "decision": "partial",
        "rawContent": "summarize this",
        "finalUserContent": "<context source=\"kit://context\" mimeType=\"application/json\">\n{\"app\":\"Finder\"}\n</context>\n\nsummarize this",
        "context": {
            "attempted": 2,
            "resolved": 1,
            "failures": [
                {
                    "label": "missing.txt",
                    "source": "/tmp/missing.txt",
                    "error": "not found"
                }
            ],
            "promptPrefix": "<context source=\"kit://context\" mimeType=\"application/json\">\n{\"app\":\"Finder\"}\n</context>"
        },
        "outcomes": [
            {
                "label": "Current Context",
                "source": "kit://context",
                "kind": "fullContent",
                "detail": "mimeType=application/json"
            },
            {
                "label": "missing.txt",
                "source": "/tmp/missing.txt",
                "kind": "failed",
                "detail": "not found"
            }
        ],
        "unresolvedParts": [
            {
                "kind": "filePath",
                "path": "/tmp/missing.txt",
                "label": "missing.txt"
            }
        ],
        "userError": "Failed to resolve context: missing.txt: not found"
    });

    // Simulate preflight receipt (same receipt object, two independent builds)
    let preflight_receipt: PreparedMessageReceipt =
        serde_json::from_value(receipt_json.clone()).expect("preflight fixture");
    let post_send_receipt: PreparedMessageReceipt =
        serde_json::from_value(receipt_json).expect("post_send fixture");

    let preflight_preview = PromptCompilerPreview::from_receipt(&preflight_receipt);
    let post_send_preview = PromptCompilerPreview::from_receipt(&post_send_receipt);

    // Core assertion: both previews are identical
    assert_eq!(
        preflight_preview, post_send_preview,
        "from_receipt must be deterministic: preflight and post-send previews must match"
    );

    // Snapshots must also match
    let preflight_snap = preflight_preview.snapshot();
    let post_send_snap = post_send_preview.snapshot();
    assert_eq!(preflight_snap, post_send_snap);

    let snap_json =
        serde_json::to_string_pretty(&preflight_snap).expect("snapshot should serialize");
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{{\n  \"test\": \"preflight_and_post_send_produce_identical_preview\",\n  \"snapshot\": {}\n}}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        snap_json
    );
}

/// Asserts that the exact outbound message ends with the authored user text
/// and that the resolved prompt prefix is preserved at the start.
#[test]
fn outbound_message_ends_with_authored_text_and_preserves_prefix() {
    let authored = "explain why Finder is frontmost";
    let prefix = "<context source=\"kit://context?profile=minimal\" mimeType=\"application/json\">\n{\"frontmostApp\":\"Finder\"}\n</context>";
    let final_content = format!("{}\n\n{}", prefix, authored);

    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "ready",
        "rawContent": authored,
        "finalUserContent": final_content,
        "context": {
            "attempted": 1,
            "resolved": 1,
            "failures": [],
            "promptPrefix": prefix
        },
        "outcomes": [
            {
                "label": "Current Context",
                "source": "kit://context?profile=minimal",
                "kind": "fullContent",
                "detail": "mimeType=application/json"
            }
        ]
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    // The final content must end with the authored text
    assert!(
        preview.final_user_content.ends_with(authored),
        "outbound message must end with authored text. Got: {:?}",
        &preview.final_user_content[preview.final_user_content.len().saturating_sub(50)..]
    );

    // The prompt prefix must be preserved at the start of the final content
    assert!(
        preview.final_user_content.starts_with(&preview.prompt_prefix),
        "outbound message must start with resolved prompt prefix"
    );

    // The prompt prefix must match what we put in
    assert_eq!(preview.prompt_prefix, prefix);

    // The raw content must be the authored text
    assert_eq!(preview.raw_content, authored);

    let snapshot = preview.snapshot();
    let snap_json = serde_json::to_string_pretty(&snapshot).expect("serialize");
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{{\n  \"test\": \"outbound_message_ends_with_authored_text_and_preserves_prefix\",\n  \"snapshot\": {},\n  \"prefix_preserved\": true,\n  \"ends_with_authored\": true\n}}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        snap_json
    );
}

/// Asserts that partial failures do not remove healthy resolved content from
/// the final payload. The healthy context block must survive even when other
/// parts fail.
#[test]
fn partial_failure_preserves_healthy_resolved_content() {
    let healthy_context = "<context source=\"kit://context\" mimeType=\"application/json\">\n{\"app\":\"Terminal\"}\n</context>";
    let authored = "check logs";
    let final_content = format!("{}\n\n{}", healthy_context, authored);

    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "partial",
        "rawContent": authored,
        "finalUserContent": final_content,
        "context": {
            "attempted": 2,
            "resolved": 1,
            "failures": [
                {
                    "label": "credentials.json",
                    "source": "/etc/credentials.json",
                    "error": "permission denied"
                }
            ],
            "promptPrefix": healthy_context
        },
        "outcomes": [
            {
                "label": "Current Context",
                "source": "kit://context",
                "kind": "fullContent",
                "detail": "mimeType=application/json"
            },
            {
                "label": "credentials.json",
                "source": "/etc/credentials.json",
                "kind": "failed",
                "detail": "permission denied"
            }
        ],
        "userError": "Failed to resolve: credentials.json: permission denied"
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    // Healthy content must be preserved in the final payload
    assert!(
        preview.final_user_content.contains(healthy_context),
        "healthy resolved context must survive partial failure"
    );

    // Authored text must also survive
    assert!(
        preview.final_user_content.ends_with(authored),
        "authored text must survive partial failure"
    );

    // Failure count must be nonzero
    assert!(preview.failures > 0, "failures must be surfaced");

    // But resolved count must also be nonzero
    assert!(preview.resolved > 0, "healthy resolved count must be nonzero");

    // The prompt prefix must contain the healthy content
    assert_eq!(preview.prompt_prefix, healthy_context);

    // Verify rows: at least one FullContent and one Failed
    let full_content_rows: Vec<_> = preview
        .rows
        .iter()
        .filter(|r| r.kind == PromptCompilerRowKind::FullContent)
        .collect();
    let failed_rows: Vec<_> = preview
        .rows
        .iter()
        .filter(|r| r.kind == PromptCompilerRowKind::Failed)
        .collect();

    assert!(
        !full_content_rows.is_empty(),
        "must have at least one FullContent row for healthy part"
    );
    assert!(
        !failed_rows.is_empty(),
        "must have at least one Failed row for broken part"
    );

    let snapshot = preview.snapshot();
    let snap_json = serde_json::to_string_pretty(&snapshot).expect("serialize");
    println!(
        "--- RECEIPT_SNAPSHOT_JSON ---\n{{\n  \"test\": \"partial_failure_preserves_healthy_resolved_content\",\n  \"snapshot\": {},\n  \"healthy_content_preserved\": true,\n  \"authored_text_preserved\": true\n}}\n--- END_RECEIPT_SNAPSHOT_JSON ---",
        snap_json
    );
}

#[test]
fn metadata_only_outcome_produces_correct_row_kind() {
    let receipt = make_receipt(json!({
        "schemaVersion": 1,
        "decision": "ready",
        "rawContent": "check this",
        "finalUserContent": "<attachment path=\"/tmp/big.bin\" unreadable=\"true\" bytes=\"999999\" />\n\ncheck this",
        "context": {
            "attempted": 1,
            "resolved": 1,
            "failures": [],
            "promptPrefix": "<attachment path=\"/tmp/big.bin\" unreadable=\"true\" bytes=\"999999\" />"
        },
        "outcomes": [
            {
                "label": "big.bin",
                "source": "/tmp/big.bin",
                "kind": "metadataOnly",
                "detail": "unreadable, 999999 bytes"
            }
        ]
    }));

    let preview = PromptCompilerPreview::from_receipt(&receipt);

    assert_eq!(preview.rows.len(), 1);
    assert_eq!(preview.rows[0].kind, PromptCompilerRowKind::MetadataOnly);
    assert_eq!(preview.rows[0].label, "big.bin");
}
