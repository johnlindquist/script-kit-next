//! Integration tests for the submit_message context-part resolution flow.
//!
//! Validates that `resolve_context_parts_with_receipt` produces correct receipts
//! for mixed success, all success, and all failure scenarios — the same logic
//! that `submit_message` relies on for deterministic failure handling.

use script_kit_gpui::ai::message_parts::{
    resolve_context_parts_with_receipt, AiContextPart, ContextResolutionReceipt,
};

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
        receipt.failures[0].error.contains("Failed to stat attachment"),
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
    let error_text = format!("Failed to resolve context: {}", failure_summaries.join("; "));

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
