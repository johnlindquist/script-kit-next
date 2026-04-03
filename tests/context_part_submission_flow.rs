//! Integration tests for the submit_message context-part resolution flow.
//!
//! Validates that both `resolve_context_parts_with_receipt` and the canonical
//! `prepare_user_message_with_receipt` produce correct receipts for mixed success,
//! all success, and all failure scenarios — the same logic that `submit_message`
//! relies on for deterministic failure handling.

use script_kit_gpui::ai::message_parts::{
    prepare_user_message_with_receipt, resolve_context_parts_with_receipt, AiContextPart,
    ContextPartPreparationOutcomeKind, ContextResolutionReceipt, PreparedMessageDecision,
    AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
};

/// Enable deterministic context capture so `kit://context` resolution
/// does not trigger real Cmd+C keystrokes.
fn init() {
    script_kit_gpui::context_snapshot::enable_deterministic_context_capture();
}

/// When one file resolves and another is missing, the receipt must report
/// partial failure while preserving the successful prefix.
#[test]
fn submit_flow_mixed_success_keeps_unresolved_part_pending() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let good_file = dir.path().join("good.rs");
    std::fs::write(&good_file, "fn ok() {}").expect("write good file");

    let parts = vec![
        AiContextPart::FilePath {
            path: good_file.to_string_lossy().to_string(),
            label: "good.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/ghost.txt".to_string(),
            label: "ghost.txt".to_string(),
        },
    ];

    let receipt: ContextResolutionReceipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 2);
    assert_eq!(receipt.resolved, 1);
    assert!(receipt.has_failures());
    assert_eq!(receipt.failures.len(), 1);
    assert_eq!(receipt.failures[0].label, "ghost.txt");
    assert_eq!(receipt.failures[0].source, "/nonexistent/ghost.txt");
    assert!(
        receipt.failures[0]
            .error
            .contains("Failed to stat attachment"),
        "error should mention stat failure, got: {}",
        receipt.failures[0].error
    );

    // The good file's prefix must survive
    assert!(
        receipt.prompt_prefix.contains("<attachment path="),
        "prompt_prefix should contain the successful attachment"
    );
    assert!(
        receipt.prompt_prefix.contains("fn ok() {}"),
        "prompt_prefix should contain the successful file content"
    );
    // The failed file must NOT appear in the prefix
    assert!(
        !receipt.prompt_prefix.contains("ghost.txt"),
        "prompt_prefix should not contain the failed file"
    );

    // Simulate submit_message's restore logic: unresolved parts are identified
    // by matching their source against receipt failures.
    let failed_sources: std::collections::HashSet<&str> =
        receipt.failures.iter().map(|f| f.source.as_str()).collect();
    let unresolved: Vec<_> = parts
        .iter()
        .filter(|p| failed_sources.contains(p.source()))
        .cloned()
        .collect();

    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].source(), "/nonexistent/ghost.txt");
}

/// When all parts resolve successfully, the receipt reports no failures
/// and the prompt prefix contains all resolved blocks.
#[test]
fn submit_flow_all_success_persists_prefixed_content() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_a = dir.path().join("alpha.rs");
    let file_b = dir.path().join("beta.rs");
    std::fs::write(&file_a, "fn alpha() {}").expect("write a");
    std::fs::write(&file_b, "fn beta() {}").expect("write b");

    let parts = vec![
        AiContextPart::FilePath {
            path: file_a.to_string_lossy().to_string(),
            label: "alpha.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: file_b.to_string_lossy().to_string(),
            label: "beta.rs".to_string(),
        },
    ];

    let receipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 2);
    assert_eq!(receipt.resolved, 2);
    assert!(!receipt.has_failures());
    assert!(receipt.failures.is_empty());
    assert!(receipt.prompt_prefix.contains("fn alpha() {}"));
    assert!(receipt.prompt_prefix.contains("fn beta() {}"));

    // Simulate how submit_message builds final content with user text
    let user_text = "explain this code";
    let final_content = format!("{}\n\n{}", receipt.prompt_prefix, user_text);
    assert!(final_content.contains("<attachment path="));
    assert!(final_content.contains("explain this code"));
}

/// When all parts fail, the receipt reports zero resolved, and submit_message
/// must not save any message (receipt.resolved == 0 guard).
#[test]
fn submit_flow_all_failures_saves_no_message_and_sets_error() {
    let parts = vec![
        AiContextPart::FilePath {
            path: "/nonexistent/a.txt".to_string(),
            label: "a.txt".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/b.txt".to_string(),
            label: "b.txt".to_string(),
        },
    ];

    let receipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 2);
    assert_eq!(receipt.resolved, 0);
    assert!(receipt.has_failures());
    assert_eq!(receipt.failures.len(), 2);
    assert!(
        receipt.prompt_prefix.is_empty(),
        "prompt_prefix must be empty when nothing resolved"
    );

    // Simulate submit_message's error summary construction
    let failure_summaries: Vec<String> = receipt
        .failures
        .iter()
        .map(|f| format!("{}: {}", f.label, f.error))
        .collect();
    let error_text = format!(
        "Failed to resolve context: {}",
        failure_summaries.join("; ")
    );

    assert!(error_text.contains("a.txt"));
    assert!(error_text.contains("b.txt"));
    assert!(error_text.starts_with("Failed to resolve context:"));

    // In submit_message, when receipt.resolved == 0, we return early without saving.
    // Verify this guard condition.
    assert_eq!(receipt.resolved, 0, "zero resolved means no message saved");
}

/// Receipt serde roundtrip: ensures receipts can be serialized for structured logging
/// and deserialized for machine verification.
#[test]
fn receipt_serde_roundtrip() {
    let receipt = ContextResolutionReceipt {
        attempted: 3,
        resolved: 2,
        failures: vec![
            script_kit_gpui::ai::message_parts::ContextResolutionFailure {
                label: "ghost.txt".to_string(),
                source: "/tmp/ghost.txt".to_string(),
                error: "file not found".to_string(),
            },
        ],
        prompt_prefix: "<attachment path=\"/tmp/a.rs\">\nfn a() {}\n</attachment>".to_string(),
    };

    let json = serde_json::to_string(&receipt).expect("serialize receipt");
    let deserialized: ContextResolutionReceipt =
        serde_json::from_str(&json).expect("deserialize receipt");

    assert_eq!(receipt, deserialized);
    assert!(deserialized.has_failures());
    assert_eq!(deserialized.failures.len(), 1);
}

/// Empty failures are skipped in serialization (skip_serializing_if).
#[test]
fn receipt_serde_omits_empty_failures() {
    let receipt = ContextResolutionReceipt {
        attempted: 1,
        resolved: 1,
        failures: vec![],
        prompt_prefix: "ok".to_string(),
    };

    let json = serde_json::to_string(&receipt).expect("serialize receipt");
    assert!(
        !json.contains("failures"),
        "empty failures should be omitted from JSON"
    );
    assert!(!receipt.has_failures());
}

// ==========================================================================
// End-to-end submit path tests via prepare_user_message_with_receipt
// ==========================================================================

/// Successful ResourceUri attachment produces `<context source="...">` block
/// in the prepared `final_user_content`.
#[test]
fn e2e_submit_resource_uri_produces_context_source_block() {
    init();
    let parts = vec![AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    }];

    let receipt = prepare_user_message_with_receipt("explain this", &parts, &[], &[]);

    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
    assert_eq!(
        receipt.schema_version,
        AI_MESSAGE_PREPARATION_SCHEMA_VERSION
    );
    assert_eq!(receipt.context.attempted, 1);
    assert_eq!(receipt.context.resolved, 1);
    assert!(!receipt.context.has_failures());

    // The final_user_content must contain the resolved <context source="..."> block
    assert!(
        receipt
            .final_user_content
            .contains("source=\"kit://context?profile=minimal\""),
        "final_user_content should contain <context source=\"...\"> block, got: {}",
        &receipt.final_user_content[..receipt.final_user_content.len().min(200)]
    );
    assert!(
        receipt
            .final_user_content
            .contains("mimeType=\"application/json\""),
        "final_user_content should contain MIME type"
    );
    assert!(
        receipt.final_user_content.ends_with("explain this"),
        "user message should follow context block"
    );

    // Outcome should be FullContent for the resource
    assert_eq!(receipt.outcomes.len(), 1);
    assert_eq!(
        receipt.outcomes[0].kind,
        ContextPartPreparationOutcomeKind::FullContent
    );
    assert_eq!(receipt.outcomes[0].label, "Current Context");
    assert_eq!(receipt.outcomes[0].source, "kit://context?profile=minimal");

    assert!(receipt.unresolved_parts.is_empty());
    assert!(receipt.user_error.is_none());
    assert!(receipt.can_send_message());
}

/// Mixed success/failure through prepare_user_message_with_receipt:
/// successful context blocks remain in prepared content, unresolved parts
/// are inspectable, and decision is Partial.
#[test]
fn e2e_submit_partial_failure_preserves_successful_blocks_and_tracks_unresolved() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let good_file = dir.path().join("code.rs");
    std::fs::write(&good_file, "fn important() {}").expect("write");

    let parts = vec![
        AiContextPart::FilePath {
            path: good_file.to_string_lossy().to_string(),
            label: "code.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/vanished.txt".to_string(),
            label: "vanished.txt".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("review this", &parts, &[], &[]);

    // Decision: Partial (some resolved, some failed)
    assert_eq!(receipt.decision, PreparedMessageDecision::Partial);
    assert!(
        receipt.can_send_message(),
        "Partial should still be sendable"
    );

    // Context receipt tracks both attempted and resolved
    assert_eq!(receipt.context.attempted, 2);
    assert_eq!(receipt.context.resolved, 1);
    assert_eq!(receipt.context.failures.len(), 1);
    assert_eq!(receipt.context.failures[0].label, "vanished.txt");

    // Successful block survives in final_user_content
    assert!(
        receipt.final_user_content.contains("<attachment path="),
        "successful attachment block must be in final_user_content"
    );
    assert!(
        receipt.final_user_content.contains("fn important() {}"),
        "successful file content must be in final_user_content"
    );
    assert!(
        !receipt.final_user_content.contains("vanished.txt"),
        "failed part must not appear in final_user_content"
    );
    assert!(
        receipt.final_user_content.ends_with("review this"),
        "user message must follow prefix"
    );

    // Unresolved parts are inspectable
    assert_eq!(receipt.unresolved_parts.len(), 1);
    assert_eq!(
        receipt.unresolved_parts[0].source(),
        "/nonexistent/vanished.txt"
    );
    assert_eq!(receipt.unresolved_parts[0].label(), "vanished.txt");

    // Per-part outcomes explain what happened
    assert_eq!(receipt.outcomes.len(), 2);
    assert_eq!(
        receipt.outcomes[0].kind,
        ContextPartPreparationOutcomeKind::FullContent
    );
    assert_eq!(receipt.outcomes[0].label, "code.rs");
    assert_eq!(
        receipt.outcomes[1].kind,
        ContextPartPreparationOutcomeKind::Failed
    );
    assert_eq!(receipt.outcomes[1].label, "vanished.txt");

    // User error is present for UI display
    assert!(receipt.user_error.is_some());
    assert!(
        receipt
            .user_error
            .as_ref()
            .unwrap()
            .contains("vanished.txt"),
        "user_error should mention the failing part"
    );
}

/// When all attached context parts fail, the decision is Blocked,
/// no false-success receipt is emitted, and can_send_message returns false.
#[test]
fn e2e_submit_all_failures_blocked_state_surfaced_explicitly() {
    let parts = vec![
        AiContextPart::FilePath {
            path: "/nonexistent/first.txt".to_string(),
            label: "first.txt".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/second.txt".to_string(),
            label: "second.txt".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("help me", &parts, &[], &[]);

    // Decision must be Blocked (not Ready or Partial)
    assert_eq!(receipt.decision, PreparedMessageDecision::Blocked);
    assert!(
        !receipt.can_send_message(),
        "Blocked state must prevent sending"
    );

    // Context receipt: zero resolved
    assert_eq!(receipt.context.attempted, 2);
    assert_eq!(receipt.context.resolved, 0);
    assert_eq!(receipt.context.failures.len(), 2);
    assert!(
        receipt.context.prompt_prefix.is_empty(),
        "prompt_prefix must be empty when nothing resolved"
    );

    // All parts remain unresolved
    assert_eq!(receipt.unresolved_parts.len(), 2);
    assert_eq!(receipt.unresolved_parts[0].label(), "first.txt");
    assert_eq!(receipt.unresolved_parts[1].label(), "second.txt");

    // Per-part outcomes are all Failed
    assert_eq!(receipt.outcomes.len(), 2);
    assert!(
        receipt
            .outcomes
            .iter()
            .all(|o| o.kind == ContextPartPreparationOutcomeKind::Failed),
        "all outcomes must be Failed"
    );

    // User error mentions both failing parts
    assert!(receipt.user_error.is_some());
    let err = receipt.user_error.as_ref().unwrap();
    assert!(err.contains("first.txt"), "error should mention first.txt");
    assert!(
        err.contains("second.txt"),
        "error should mention second.txt"
    );
    assert!(
        err.starts_with("Failed to resolve context:"),
        "error should use deterministic prefix"
    );

    // final_user_content falls back to raw content only (no context prefix)
    assert_eq!(
        receipt.final_user_content, "help me",
        "when all parts fail, content is raw message only"
    );
}

/// Receipt data is sufficient for machine verification: serializable,
/// contains all fields needed to reconstruct what was sent, failed, and pending.
#[test]
fn e2e_submit_receipt_is_machine_verifiable() {
    init();
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("data.rs");
    std::fs::write(&file, "struct Data;").expect("write");

    let parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: file.to_string_lossy().to_string(),
            label: "data.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/gone.txt".to_string(),
            label: "gone.txt".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("query", &parts, &[], &[]);

    // Serde roundtrip: receipt can be serialized for structured logging
    let json = serde_json::to_string(&receipt).expect("serialize");
    let rt: script_kit_gpui::ai::message_parts::PreparedMessageReceipt =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(receipt, rt, "receipt must survive serde roundtrip");

    // Machine-readable fields present
    assert!(json.contains("\"schemaVersion\""));
    assert!(json.contains("\"decision\""));
    assert!(json.contains("\"rawContent\""));
    assert!(json.contains("\"finalUserContent\""));
    assert!(json.contains("\"unresolvedParts\""));
    assert!(json.contains("\"outcomes\""));

    // Receipt explains what was sent
    assert_eq!(rt.context.attempted, 3);
    assert_eq!(rt.context.resolved, 2);
    assert_eq!(rt.context.failures.len(), 1);

    // Receipt explains what failed
    assert_eq!(rt.context.failures[0].label, "gone.txt");

    // Receipt explains what remains pending
    assert_eq!(rt.unresolved_parts.len(), 1);
    assert_eq!(rt.unresolved_parts[0].label(), "gone.txt");

    // Outcome kinds are machine-parseable
    let outcome_kinds: Vec<_> = rt.outcomes.iter().map(|o| &o.kind).collect();
    assert_eq!(
        outcome_kinds,
        vec![
            &ContextPartPreparationOutcomeKind::FullContent,
            &ContextPartPreparationOutcomeKind::FullContent,
            &ContextPartPreparationOutcomeKind::Failed,
        ]
    );
}

/// No parts + no message: receipt is Ready with empty content.
#[test]
fn e2e_submit_no_parts_no_message_is_ready_empty() {
    let receipt = prepare_user_message_with_receipt("", &[], &[], &[]);

    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
    assert!(receipt.can_send_message());
    assert_eq!(receipt.context.attempted, 0);
    assert_eq!(receipt.context.resolved, 0);
    assert!(receipt.outcomes.is_empty());
    assert!(receipt.unresolved_parts.is_empty());
    assert!(receipt.user_error.is_none());
    assert!(receipt.final_user_content.is_empty());
}

// ==========================================================================
// FocusedTarget resolution tests
// ==========================================================================

/// A FocusedTarget part resolves into a deterministic `<context source="focusedTarget">` block.
#[test]
fn focused_target_part_resolves_to_prompt_block() {
    let part = AiContextPart::FocusedTarget {
        label: "File: tab_ai_mode.rs".to_string(),
        target: script_kit_gpui::ai::TabAiTargetContext {
            source: "FileSearch".to_string(),
            kind: "file".to_string(),
            semantic_id: "choice:0:tab_ai_mode.rs".to_string(),
            label: "tab_ai_mode.rs".to_string(),
            metadata: Some(serde_json::json!({
                "path": "/tmp/tab_ai_mode.rs"
            })),
        },
    };

    let receipt = resolve_context_parts_with_receipt(&[part], &[], &[]);

    assert_eq!(receipt.attempted, 1);
    assert_eq!(receipt.resolved, 1);
    assert!(!receipt.has_failures());
    assert!(receipt.prompt_prefix.contains("focusedTarget"));
    assert!(receipt.prompt_prefix.contains("tab_ai_mode.rs"));
    assert!(receipt.prompt_prefix.contains("/tmp/tab_ai_mode.rs"));
    assert!(receipt.prompt_prefix.contains("itemSource=\"FileSearch\""));
    assert!(receipt.prompt_prefix.contains("itemKind=\"file\""));
}

/// FocusedTarget without metadata still resolves with empty metadata block.
#[test]
fn focused_target_part_resolves_without_metadata() {
    let part = AiContextPart::FocusedTarget {
        label: "Command: hello".to_string(),
        target: script_kit_gpui::ai::TabAiTargetContext {
            source: "ScriptList".to_string(),
            kind: "script".to_string(),
            semantic_id: "choice:0:hello".to_string(),
            label: "hello".to_string(),
            metadata: None,
        },
    };

    let receipt = resolve_context_parts_with_receipt(&[part], &[], &[]);

    assert_eq!(receipt.resolved, 1);
    assert!(receipt.prompt_prefix.contains("focusedTarget"));
    assert!(receipt.prompt_prefix.contains("{}"));
}

/// FocusedTarget works alongside FilePath parts through prepare_user_message_with_receipt.
#[test]
fn e2e_submit_focused_target_with_file_path() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = dir.path().join("helper.rs");
    std::fs::write(&file, "fn helper() {}").expect("write");

    let parts = vec![
        AiContextPart::FocusedTarget {
            label: "File: main.rs".to_string(),
            target: script_kit_gpui::ai::TabAiTargetContext {
                source: "FileSearch".to_string(),
                kind: "file".to_string(),
                semantic_id: "choice:0:main.rs".to_string(),
                label: "main.rs".to_string(),
                metadata: None,
            },
        },
        AiContextPart::FilePath {
            path: file.to_string_lossy().to_string(),
            label: "helper.rs".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("explain", &parts, &[], &[]);

    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
    assert_eq!(receipt.context.attempted, 2);
    assert_eq!(receipt.context.resolved, 2);
    assert!(receipt.final_user_content.contains("focusedTarget"));
    assert!(receipt.final_user_content.contains("fn helper() {}"));
    assert!(receipt.final_user_content.ends_with("explain"));

    // FocusedTarget block should come first (order preservation)
    let focused_pos = receipt
        .final_user_content
        .find("focusedTarget")
        .expect("focused target block present");
    let file_pos = receipt
        .final_user_content
        .find("fn helper() {}")
        .expect("file block present");
    assert!(
        focused_pos < file_pos,
        "focused target should come before file attachment"
    );
}

/// Full success with multiple context sources: final_user_content preserves
/// block order and all blocks are present.
#[test]
fn e2e_submit_full_success_preserves_all_blocks_in_order() {
    init();
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_a = dir.path().join("first.rs");
    let file_b = dir.path().join("second.rs");
    std::fs::write(&file_a, "// first").expect("write a");
    std::fs::write(&file_b, "// second").expect("write b");

    let parts = vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Context".to_string(),
        },
        AiContextPart::FilePath {
            path: file_a.to_string_lossy().to_string(),
            label: "first.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: file_b.to_string_lossy().to_string(),
            label: "second.rs".to_string(),
        },
    ];

    let receipt = prepare_user_message_with_receipt("explain", &parts, &[], &[]);

    assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
    assert_eq!(receipt.context.attempted, 3);
    assert_eq!(receipt.context.resolved, 3);
    assert!(receipt.unresolved_parts.is_empty());

    // All blocks present in final content
    assert!(receipt.final_user_content.contains("<context source="));
    assert!(receipt.final_user_content.contains("// first"));
    assert!(receipt.final_user_content.contains("// second"));
    assert!(receipt.final_user_content.ends_with("explain"));

    // Order preserved: context block before file blocks
    let context_pos = receipt
        .final_user_content
        .find("<context source=")
        .expect("context block present");
    let first_pos = receipt
        .final_user_content
        .find("// first")
        .expect("first file present");
    let second_pos = receipt
        .final_user_content
        .find("// second")
        .expect("second file present");

    assert!(
        context_pos < first_pos,
        "context block should come before first file"
    );
    assert!(
        first_pos < second_pos,
        "first file should come before second file"
    );
}
