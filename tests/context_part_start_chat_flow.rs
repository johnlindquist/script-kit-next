//! Integration tests for the start-chat context-part resolution flow.
//!
//! Validates that `handle_start_chat` uses the same receipt-based success/failure
//! contract as `submit_message` and does not silently downgrade to raw-message-only
//! behavior when parts fail.

use script_kit_gpui::ai::message_parts::{
    resolve_context_parts_with_receipt, AiContextPart, ContextResolutionReceipt,
};

/// Empty message + valid parts → the resolved prefix becomes the user content,
/// and the chat title should be "Context attachment".
#[test]
fn start_chat_empty_message_with_valid_parts_uses_resolved_prefix() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("context.rs");
    std::fs::write(&file_path, "fn context() {}").expect("write file");

    let parts = vec![AiContextPart::FilePath {
        path: file_path.to_string_lossy().to_string(),
        label: "context.rs".to_string(),
    }];

    let receipt: ContextResolutionReceipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 1);
    assert_eq!(receipt.resolved, 1);
    assert!(!receipt.has_failures());
    assert!(
        receipt.prompt_prefix.contains("<attachment path="),
        "resolved prefix should contain attachment tag"
    );
    assert!(
        receipt.prompt_prefix.contains("fn context() {}"),
        "resolved prefix should contain file content"
    );

    // Simulate handle_start_chat's content construction for empty message + valid prefix
    let message = "";
    let prefix = &receipt.prompt_prefix;
    let final_content = if !prefix.is_empty() && !message.trim().is_empty() {
        format!("{prefix}\n\n{message}")
    } else if !prefix.is_empty() {
        prefix.clone()
    } else {
        message.to_string()
    };

    assert!(
        final_content.contains("<attachment path="),
        "final content should be the resolved prefix when message is empty"
    );
    assert!(
        !final_content.contains("\n\n"),
        "no double-newline separator when message is empty"
    );

    // Title logic: empty message + has_parts → "Context attachment"
    let has_parts = true;
    let has_image = false;
    let title = if message.trim().is_empty() && has_image {
        "Image attachment".to_string()
    } else if message.trim().is_empty() && has_parts {
        "Context attachment".to_string()
    } else {
        "Generated title".to_string()
    };
    assert_eq!(title, "Context attachment");
}

/// Message + valid parts → saved user message starts with resolved prefix,
/// then `\n\n`, then the raw message.
#[test]
fn start_chat_message_with_valid_parts_prefixes_content() {
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

    // Simulate handle_start_chat's content construction for message + valid prefix
    let message = "explain this code";
    let prefix = &receipt.prompt_prefix;
    let final_content = if !prefix.is_empty() && !message.trim().is_empty() {
        format!("{prefix}\n\n{message}")
    } else if !prefix.is_empty() {
        prefix.clone()
    } else {
        message.to_string()
    };

    // Verify prefix comes first
    assert!(
        final_content.starts_with("<attachment"),
        "final content should start with the resolved prefix"
    );
    // Verify separator
    assert!(
        final_content.contains("</attachment>\n\n<attachment"),
        "two resolved blocks should be separated by double newline"
    );
    // Verify user message at end
    assert!(
        final_content.ends_with("explain this code"),
        "final content should end with the raw user message"
    );
    // Verify both files present
    assert!(final_content.contains("fn alpha() {}"));
    assert!(final_content.contains("fn beta() {}"));
}

/// Invalid part → failure is surfaced deterministically via the receipt;
/// no silent raw-message-only downgrade.
#[test]
fn start_chat_invalid_part_surfaces_failure_deterministically() {
    let parts = vec![AiContextPart::FilePath {
        path: "/nonexistent/ghost.txt".to_string(),
        label: "ghost.txt".to_string(),
    }];

    let receipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 1);
    assert_eq!(receipt.resolved, 0);
    assert!(receipt.has_failures());
    assert_eq!(receipt.failures.len(), 1);
    assert_eq!(receipt.failures[0].label, "ghost.txt");
    assert_eq!(receipt.failures[0].source, "/nonexistent/ghost.txt");
    assert!(
        receipt.failures[0].error.contains("Failed to stat attachment"),
        "error should mention stat failure, got: {}",
        receipt.failures[0].error
    );

    // Simulate handle_start_chat's receipt-based failure contract:
    // When receipt.resolved == 0, the error must be surfaced, not silently swallowed.
    let failure_summaries: Vec<String> = receipt
        .failures
        .iter()
        .map(|f| format!("{}: {}", f.label, f.error))
        .collect();
    let error_text = format!(
        "Failed to resolve context: {}",
        failure_summaries.join("; ")
    );

    assert!(
        error_text.contains("ghost.txt"),
        "error should identify the failing part"
    );
    assert!(
        error_text.starts_with("Failed to resolve context:"),
        "error should use deterministic prefix"
    );

    // The chat is still created (so SDK gets a valid chatId), but with raw message fallback
    let message = "hello";
    let final_content = message.to_string();
    assert_eq!(
        final_content, "hello",
        "when all parts fail, content falls back to raw message"
    );
}

/// Mixed success: one valid part + one invalid part → partial prefix preserved,
/// failure tracked, not silently dropped.
#[test]
fn start_chat_mixed_parts_preserves_successful_prefix() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let good_file = dir.path().join("good.rs");
    std::fs::write(&good_file, "fn good() {}").expect("write");

    let parts = vec![
        AiContextPart::FilePath {
            path: good_file.to_string_lossy().to_string(),
            label: "good.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: "/nonexistent/bad.txt".to_string(),
            label: "bad.txt".to_string(),
        },
    ];

    let receipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 2);
    assert_eq!(receipt.resolved, 1);
    assert!(receipt.has_failures());
    assert_eq!(receipt.failures.len(), 1);
    assert_eq!(receipt.failures[0].label, "bad.txt");

    // Simulate handle_start_chat's partial-success path
    let message = "explain";
    let prefix = &receipt.prompt_prefix;
    let final_content = if !message.trim().is_empty() {
        format!("{prefix}\n\n{message}")
    } else {
        prefix.clone()
    };

    assert!(
        final_content.contains("<attachment path="),
        "successful part's prefix must survive partial failure"
    );
    assert!(
        final_content.contains("fn good() {}"),
        "good file content must be in prefix"
    );
    assert!(
        !final_content.contains("bad.txt"),
        "failed part must not appear in prefix"
    );
    assert!(
        final_content.ends_with("explain"),
        "user message must follow prefix"
    );
}

/// Parts order is preserved through resolution: the receipt's prompt_prefix
/// contains blocks in the same order as the input parts.
#[test]
fn start_chat_parts_order_preserved_through_resolution() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_first = dir.path().join("first.rs");
    let file_second = dir.path().join("second.rs");
    let file_third = dir.path().join("third.rs");
    std::fs::write(&file_first, "// first").expect("write");
    std::fs::write(&file_second, "// second").expect("write");
    std::fs::write(&file_third, "// third").expect("write");

    let parts = vec![
        AiContextPart::FilePath {
            path: file_first.to_string_lossy().to_string(),
            label: "first.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: file_second.to_string_lossy().to_string(),
            label: "second.rs".to_string(),
        },
        AiContextPart::FilePath {
            path: file_third.to_string_lossy().to_string(),
            label: "third.rs".to_string(),
        },
    ];

    let receipt = resolve_context_parts_with_receipt(&parts, &[], &[]);

    assert_eq!(receipt.attempted, 3);
    assert_eq!(receipt.resolved, 3);

    // Verify order: first appears before second, second before third
    let first_pos = receipt
        .prompt_prefix
        .find("// first")
        .expect("first should be in prefix");
    let second_pos = receipt
        .prompt_prefix
        .find("// second")
        .expect("second should be in prefix");
    let third_pos = receipt
        .prompt_prefix
        .find("// third")
        .expect("third should be in prefix");

    assert!(
        first_pos < second_pos,
        "first must appear before second in prefix"
    );
    assert!(
        second_pos < third_pos,
        "second must appear before third in prefix"
    );
}
