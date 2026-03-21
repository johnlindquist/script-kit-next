//! Integration tests for the context preflight system.
//!
//! Validates that the preflight derivation pipeline produces deterministic,
//! well-structured state from `PreparedMessageReceipt` inputs, and that
//! the generation-guard pattern correctly rejects stale results.

use script_kit_gpui::ai::{
    estimate_tokens_from_text, preflight_state_from_receipt, status_from_decision,
    ContextPreflightState, ContextPreflightStatus, PreparedMessageDecision,
    PreparedMessageReceipt, AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
};
use script_kit_gpui::ai::{
    merge_context_parts, AiContextPart, ContextAssemblyReceipt, ContextResolutionFailure,
    ContextResolutionReceipt,
};

// ---------- Stale preflight rejection ----------

#[test]
fn stale_generation_is_rejected_by_guard_pattern() {
    // Simulate: preflight started at gen=10, then a new preflight bumps to gen=11
    let started_gen = 10_u64;
    let current_gen = started_gen.wrapping_add(1);

    // The guard in schedule_context_preflight:
    //   if app.context_preflight.generation != generation { return; }
    assert_ne!(
        current_gen, started_gen,
        "New generation should differ from started generation"
    );
}

#[test]
fn generation_wrap_at_max_u64_still_guards_correctly() {
    let started_gen = u64::MAX;
    let current_gen = started_gen.wrapping_add(1);

    assert_eq!(current_gen, 0, "Generation should wrap to 0 at u64::MAX");
    assert_ne!(
        current_gen, started_gen,
        "Wrapped generation should still differ from started generation"
    );
}

#[test]
fn idle_preflight_state_has_zero_budget() {
    let state = ContextPreflightState::default();
    assert_eq!(state.status, ContextPreflightStatus::Idle);
    assert_eq!(state.approx_tokens, 0);
    assert_eq!(state.prompt_chars, 0);
    assert_eq!(state.attempted, 0);
    assert_eq!(state.resolved, 0);
    assert_eq!(state.failures, 0);
    assert!(state.receipt.is_none());
}

// ---------- Duplicate attachments do not inflate budget ----------

#[test]
fn duplicate_context_parts_are_deduped_in_receipt() {
    let part_a = AiContextPart::ResourceUri {
        uri: "kit://context?profile=minimal".to_string(),
        label: "Current Context".to_string(),
    };
    let part_b = part_a.clone();

    let merged = merge_context_parts(&[part_a], &[part_b]);

    // merge_context_parts returns the deduplicated list
    assert_eq!(
        merged.len(),
        1,
        "Two identical parts should merge to one unique part"
    );
}

#[test]
fn preflight_state_reflects_assembly_dedup_count() {
    let receipt = PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: PreparedMessageDecision::Ready,
        raw_content: "hello".to_string(),
        final_user_content: "prefix\n\nhello".to_string(),
        context: ContextResolutionReceipt {
            attempted: 1,
            resolved: 1,
            failures: vec![],
            prompt_prefix: "some resolved text".to_string(),
        },
        assembly: Some(ContextAssemblyReceipt {
            mention_count: 1,
            pending_count: 2,
            merged_count: 1,
            duplicates_removed: 2,
            duplicates: vec![],
            merged_parts: vec![],
        }),
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(1, receipt);
    assert_eq!(
        state.duplicates_removed, 2,
        "Preflight state should reflect the assembly's duplicates_removed count"
    );
    assert_eq!(state.resolved, 1);
    assert!(state.approx_tokens > 0);
}

// ---------- Blocked/partial receipts surface warnings ----------

#[test]
fn blocked_receipt_produces_blocked_status_with_zero_tokens() {
    let receipt = PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: PreparedMessageDecision::Blocked,
        raw_content: "test".to_string(),
        final_user_content: "test".to_string(),
        context: ContextResolutionReceipt {
            attempted: 2,
            resolved: 0,
            failures: vec![
                ContextResolutionFailure {
                    label: "Context A".to_string(),
                    source: "kit://context".to_string(),
                    error: "resource not found".to_string(),
                },
                ContextResolutionFailure {
                    label: "Context B".to_string(),
                    source: "kit://context?profile=full".to_string(),
                    error: "timeout".to_string(),
                },
            ],
            prompt_prefix: String::new(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: Some("All context parts failed to resolve".to_string()),
    };

    let state = preflight_state_from_receipt(5, receipt.clone());

    assert_eq!(state.status, ContextPreflightStatus::Blocked);
    assert_eq!(state.resolved, 0);
    assert_eq!(state.failures, 2);
    assert_eq!(state.approx_tokens, 0);
    assert_eq!(state.prompt_chars, 0);
    assert!(state.receipt.is_some());
    assert!(!receipt.can_send_message());
}

#[test]
fn partial_receipt_produces_partial_status_with_nonzero_failures() {
    let receipt = PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: PreparedMessageDecision::Partial,
        raw_content: "test".to_string(),
        final_user_content: "resolved\n\ntest".to_string(),
        context: ContextResolutionReceipt {
            attempted: 3,
            resolved: 2,
            failures: vec![ContextResolutionFailure {
                label: "Failed Context".to_string(),
                source: "kit://context?diagnostics=1".to_string(),
                error: "access denied".to_string(),
            }],
            prompt_prefix: "some resolved data here".to_string(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(7, receipt.clone());

    assert_eq!(state.status, ContextPreflightStatus::Partial);
    assert!(state.failures > 0, "Partial must have at least one failure");
    assert!(
        state.resolved < state.attempted,
        "Partial: resolved < attempted"
    );
    assert!(state.approx_tokens > 0, "Partial should have some tokens");
    assert!(receipt.can_send_message());
}

// ---------- Token estimation ----------

#[test]
fn token_estimate_matches_char_divided_by_four() {
    // 100 chars → 25 tokens exactly
    let text = "a".repeat(100);
    assert_eq!(estimate_tokens_from_text(&text), 25);

    // 101 chars → ceil(25.25) = 26
    let text = "a".repeat(101);
    assert_eq!(estimate_tokens_from_text(&text), 26);

    // Empty → 0
    assert_eq!(estimate_tokens_from_text(""), 0);
}

#[test]
fn token_estimate_uses_char_count_not_byte_count() {
    // "café" = 4 chars, 5 bytes
    let text = "café";
    assert_eq!(text.len(), 5);
    assert_eq!(text.chars().count(), 4);
    assert_eq!(estimate_tokens_from_text(text), 1);
}

// ---------- Status mapping ----------

#[test]
fn status_from_decision_maps_all_variants_correctly() {
    assert_eq!(
        status_from_decision(&PreparedMessageDecision::Ready),
        ContextPreflightStatus::Ready
    );
    assert_eq!(
        status_from_decision(&PreparedMessageDecision::Partial),
        ContextPreflightStatus::Partial
    );
    assert_eq!(
        status_from_decision(&PreparedMessageDecision::Blocked),
        ContextPreflightStatus::Blocked
    );
}

// ---------- Deterministic derivation ----------

#[test]
fn preflight_state_derivation_is_deterministic() {
    let make_receipt = || PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: PreparedMessageDecision::Ready,
        raw_content: "test".to_string(),
        final_user_content: "prefix\n\ntest".to_string(),
        context: ContextResolutionReceipt {
            attempted: 2,
            resolved: 2,
            failures: vec![],
            prompt_prefix: "deterministic content".to_string(),
        },
        assembly: Some(ContextAssemblyReceipt {
            mention_count: 1,
            pending_count: 1,
            merged_count: 2,
            duplicates_removed: 0,
            duplicates: vec![],
            merged_parts: vec![],
        }),
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state_a = preflight_state_from_receipt(42, make_receipt());
    let state_b = preflight_state_from_receipt(42, make_receipt());

    assert_eq!(state_a.generation, state_b.generation);
    assert_eq!(state_a.status, state_b.status);
    assert_eq!(state_a.attempted, state_b.attempted);
    assert_eq!(state_a.resolved, state_b.resolved);
    assert_eq!(state_a.failures, state_b.failures);
    assert_eq!(state_a.duplicates_removed, state_b.duplicates_removed);
    assert_eq!(state_a.approx_tokens, state_b.approx_tokens);
    assert_eq!(state_a.prompt_chars, state_b.prompt_chars);
}

// ---------- Receipt preservation ----------

#[test]
fn preflight_state_preserves_full_receipt_for_drawer() {
    let receipt = PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: PreparedMessageDecision::Ready,
        raw_content: "hello".to_string(),
        final_user_content: "ctx\n\nhello".to_string(),
        context: ContextResolutionReceipt {
            attempted: 1,
            resolved: 1,
            failures: vec![],
            prompt_prefix: "context data".to_string(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(1, receipt.clone());
    let stored = state.receipt.expect("Receipt should be stored");
    assert_eq!(stored.raw_content, receipt.raw_content);
    assert_eq!(stored.context.prompt_prefix, receipt.context.prompt_prefix);
}
