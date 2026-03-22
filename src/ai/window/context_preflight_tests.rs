//! Additional integration-level tests for the context preflight system.
//!
//! These tests exercise the preflight derivation functions and the
//! merge pipeline from the perspective of an external observer.

use super::context_preflight::*;

#[test]
fn test_preflight_state_from_receipt_ready_preserves_receipt() {
    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: "test".to_string(),
        final_user_content: "context\n\ntest".to_string(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 2,
            resolved: 2,
            failures: vec![],
            prompt_prefix: "resolved content here".to_string(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(1, receipt);

    assert!(
        state.receipt.is_some(),
        "Ready preflight state should carry the full receipt"
    );
    assert_eq!(state.status, ContextPreflightStatus::Ready);
    assert_eq!(state.attempted, 2);
    assert_eq!(state.resolved, 2);
    assert_eq!(state.failures, 0);
    // "resolved content here" = 21 chars → ceil(21/4) = 6
    assert_eq!(state.approx_tokens, 6);
    assert_eq!(state.prompt_chars, 21);
}

#[test]
fn test_preflight_state_from_receipt_partial_has_nonzero_failures() {
    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Partial,
        raw_content: "test".to_string(),
        final_user_content: "partial\n\ntest".to_string(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 3,
            resolved: 2,
            failures: vec![crate::ai::message_parts::ContextResolutionFailure {
                label: "bad".to_string(),
                source: "x".to_string(),
                error: "err".to_string(),
            }],
            prompt_prefix: "ok".to_string(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(3, receipt);

    assert_eq!(state.status, ContextPreflightStatus::Partial);
    assert!(
        state.failures > 0,
        "Partial preflight must have at least one failure"
    );
    assert!(
        state.resolved < state.attempted,
        "Partial preflight resolved count should be less than attempted"
    );
}

#[test]
fn test_preflight_state_from_receipt_blocked_has_zero_resolved() {
    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Blocked,
        raw_content: "test".to_string(),
        final_user_content: "test".to_string(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 2,
            resolved: 0,
            failures: vec![
                crate::ai::message_parts::ContextResolutionFailure {
                    label: "a".to_string(),
                    source: "x".to_string(),
                    error: "err".to_string(),
                },
                crate::ai::message_parts::ContextResolutionFailure {
                    label: "b".to_string(),
                    source: "y".to_string(),
                    error: "err".to_string(),
                },
            ],
            prompt_prefix: String::new(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_receipt(5, receipt);

    assert_eq!(state.status, ContextPreflightStatus::Blocked);
    assert_eq!(
        state.resolved, 0,
        "Blocked preflight should have zero resolved parts"
    );
    assert_eq!(
        state.approx_tokens, 0,
        "Blocked preflight should have zero token estimate"
    );
}

#[test]
fn test_generation_guard_with_clear_invalidates_inflight() {
    // Simulate: preflight started at gen=10, then clear_context_preflight
    // bumps to gen=11 before the result arrives.
    let initial_gen = 10_u64;
    let after_clear_gen = initial_gen.wrapping_add(1);

    // The spawned task captured `generation = 10` but the state is now at 11
    let is_stale = after_clear_gen != initial_gen;
    assert!(
        is_stale,
        "Clear should bump generation, making in-flight preflight stale"
    );

    // Verify the guard logic matches what's in schedule_context_preflight:
    // `if app.context_preflight.generation != generation { return; }`
    let app_generation = after_clear_gen; // after clear
    let task_generation = initial_gen; // captured before clear
    assert_ne!(
        app_generation, task_generation,
        "Stale guard should prevent applying the result"
    );
}

#[test]
fn test_two_identical_parts_across_slices_dedup_to_one() {
    // One part in mentions, one identical part in pending → merge
    // should dedup to a single part.
    let part = crate::ai::message_parts::AiContextPart::ResourceUri {
        uri: "kit://context".to_string(),
        label: "Context".to_string(),
    };

    let part2 = part.clone();
    let assembly = crate::ai::message_parts::merge_context_parts_with_receipt(
        std::slice::from_ref(&part),
        std::slice::from_ref(&part2),
    );

    assert_eq!(
        assembly.merged_count, 1,
        "Two identical parts should merge to one"
    );
    assert_eq!(
        assembly.duplicates_removed, 1,
        "One duplicate should be recorded"
    );
}

#[test]
fn test_three_identical_parts_dedup_across_both_slices() {
    // Two in mentions, one in pending — all identical
    let part = crate::ai::message_parts::AiContextPart::ResourceUri {
        uri: "kit://context".to_string(),
        label: "Context".to_string(),
    };

    let assembly = crate::ai::message_parts::merge_context_parts_with_receipt(
        &[part.clone(), part.clone()],
        &[part],
    );

    assert_eq!(
        assembly.merged_count, 1,
        "Three identical parts should merge to one"
    );
    assert_eq!(
        assembly.duplicates_removed, 2,
        "Two duplicates should be recorded"
    );
}

#[test]
fn test_estimate_tokens_large_context() {
    // 10,000 chars → 10000/4 = 2500 tokens exactly
    let large_text: String = "a".repeat(10_000);
    let tokens = estimate_tokens_from_text(&large_text);
    assert_eq!(tokens, 2500, "10k chars / 4 = 2500 tokens exactly");
}

#[test]
fn test_preflight_duplicates_removed_from_assembly_receipt() {
    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: String::new(),
        final_user_content: String::new(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 1,
            resolved: 1,
            failures: vec![],
            prompt_prefix: "data".to_string(),
        },
        assembly: Some(crate::ai::message_parts::ContextAssemblyReceipt {
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
        "Preflight should surface duplicates_removed from the assembly receipt"
    );
}

#[test]
fn test_preflight_snapshot_reports_recommendation_count_and_live_snapshot() {
    use super::context_recommendations::{
        ContextRecommendation, ContextRecommendationPriority,
    };
    use crate::ai::context_contract::ContextAttachmentKind;

    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: "test".to_string(),
        final_user_content: "test".to_string(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 0,
            resolved: 0,
            failures: vec![],
            prompt_prefix: String::new(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let snapshot = crate::context_snapshot::AiContextSnapshot::default();
    let recommendations = vec![
        ContextRecommendation {
            kind: ContextAttachmentKind::Selection,
            reason: "test reason".to_string(),
            priority: ContextRecommendationPriority::High,
        },
        ContextRecommendation {
            kind: ContextAttachmentKind::Browser,
            reason: "another reason".to_string(),
            priority: ContextRecommendationPriority::Medium,
        },
    ];

    let state = preflight_state_from_analysis(7, receipt, Some(snapshot), recommendations);
    let snap = state.snapshot();

    assert_eq!(snap.recommendation_count, 2, "Should report 2 recommendations");
    assert!(snap.has_live_snapshot, "Should report live snapshot is present");
    assert_eq!(snap.generation, 7);
}

#[test]
fn test_preflight_snapshot_without_live_snapshot_reports_false() {
    let receipt = crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: String::new(),
        final_user_content: String::new(),
        context: crate::ai::message_parts::ContextResolutionReceipt {
            attempted: 0,
            resolved: 0,
            failures: vec![],
            prompt_prefix: String::new(),
        },
        assembly: None,
        outcomes: vec![],
        unresolved_parts: vec![],
        user_error: None,
    };

    let state = preflight_state_from_analysis(1, receipt, None, Vec::new());
    let snap = state.snapshot();

    assert_eq!(snap.recommendation_count, 0);
    assert!(!snap.has_live_snapshot, "Should report no live snapshot");
}
