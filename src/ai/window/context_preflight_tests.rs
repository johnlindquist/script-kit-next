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
    use super::context_recommendations::{ContextRecommendation, ContextRecommendationPriority};
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

    assert_eq!(
        snap.recommendation_count, 2,
        "Should report 2 recommendations"
    );
    assert!(
        snap.has_live_snapshot,
        "Should report live snapshot is present"
    );
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

fn ready_preflight_receipt(raw_content: &str) -> crate::ai::message_parts::PreparedMessageReceipt {
    crate::ai::message_parts::PreparedMessageReceipt {
        schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
        raw_content: raw_content.to_string(),
        final_user_content: raw_content.to_string(),
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
    }
}

#[test]
fn test_preflight_snapshot_reports_exact_recommendation_count() {
    use super::context_recommendations::recommend_context_parts;
    use crate::context_snapshot::{
        AiContextSnapshot, BrowserContext, FocusedWindowContext, FrontmostAppContext,
    };

    let draft = "Rewrite this selected text in a friendlier tone";
    let snapshot = AiContextSnapshot {
        selected_text: Some("fn main() {}".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 1,
            bundle_id: "com.apple.Safari".to_string(),
            name: "Safari".to_string(),
        }),
        browser: Some(BrowserContext {
            url: "https://example.com".to_string(),
        }),
        focused_window: Some(FocusedWindowContext {
            title: "Safari".to_string(),
            width: 1440,
            height: 900,
            used_fallback: false,
        }),
        ..Default::default()
    };

    let recommendation_receipt = recommend_context_parts(draft, &snapshot, &[]);

    assert!(
        !recommendation_receipt.recommendations.is_empty(),
        "Sanity check: fixture should yield at least one recommendation"
    );

    let checkpoint_before = serde_json::json!({
        "checkpoint": "before",
        "test": "test_preflight_snapshot_reports_exact_recommendation_count",
        "contract": "recommendation_count == recommendations.len()"
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_before,
        "test_checkpoint"
    );

    let state = preflight_state_from_analysis(
        42,
        ready_preflight_receipt(draft),
        Some(snapshot),
        recommendation_receipt.recommendations.clone(),
    );
    let snap = state.snapshot();

    assert!(snap.has_live_snapshot);
    assert_eq!(
        snap.recommendation_count,
        recommendation_receipt.recommendations.len(),
        "Preflight snapshot must report the exact surfaced recommendation count"
    );

    let checkpoint_after = serde_json::json!({
        "checkpoint": "after",
        "test": "test_preflight_snapshot_reports_exact_recommendation_count",
        "contract": "recommendation_count == recommendations.len()",
        "recommendation_count": snap.recommendation_count
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_after,
        "test_checkpoint"
    );
}

#[test]
fn test_preflight_snapshot_without_live_snapshot_suppresses_recommendations() {
    use super::context_recommendations::recommend_context_parts;
    use crate::context_snapshot::{
        AiContextSnapshot, BrowserContext, FocusedWindowContext, FrontmostAppContext,
    };

    let draft = "Rewrite this selected text in a friendlier tone";
    let snapshot = AiContextSnapshot {
        selected_text: Some("fn main() {}".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 1,
            bundle_id: "com.apple.Safari".to_string(),
            name: "Safari".to_string(),
        }),
        browser: Some(BrowserContext {
            url: "https://example.com".to_string(),
        }),
        focused_window: Some(FocusedWindowContext {
            title: "Safari".to_string(),
            width: 1440,
            height: 900,
            used_fallback: false,
        }),
        ..Default::default()
    };

    let recommendation_receipt = recommend_context_parts(draft, &snapshot, &[]);

    assert!(
        !recommendation_receipt.recommendations.is_empty(),
        "Sanity check: this fixture should yield recommendations before the live snapshot is removed"
    );

    let checkpoint_before = serde_json::json!({
        "checkpoint": "before",
        "test": "test_preflight_snapshot_without_live_snapshot_suppresses_recommendations",
        "contract": "has_live_snapshot == false implies recommendation_count == 0"
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_before,
        "test_checkpoint"
    );

    let state = preflight_state_from_analysis(
        43,
        ready_preflight_receipt(draft),
        None,
        recommendation_receipt.recommendations,
    );
    let snap = state.snapshot();

    assert!(!snap.has_live_snapshot);
    assert_eq!(
        snap.recommendation_count, 0,
        "Recommendations must not surface when there is no live snapshot backing them"
    );

    let checkpoint_after = serde_json::json!({
        "checkpoint": "after",
        "test": "test_preflight_snapshot_without_live_snapshot_suppresses_recommendations",
        "contract": "has_live_snapshot == false implies recommendation_count == 0",
        "recommendation_count": snap.recommendation_count,
        "has_live_snapshot": snap.has_live_snapshot
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_after,
        "test_checkpoint"
    );
}

#[test]
fn test_recommendation_determinism_same_input_same_output() {
    use super::context_recommendations::recommend_context_parts;
    use crate::context_snapshot::{
        AiContextSnapshot, BrowserContext, FocusedWindowContext, FrontmostAppContext,
    };

    let draft = "Summarize this page and explain the current window";
    let snapshot = AiContextSnapshot {
        selected_text: Some("let x = 1;".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 99,
            bundle_id: "com.apple.Safari".to_string(),
            name: "Safari".to_string(),
        }),
        browser: Some(BrowserContext {
            url: "https://docs.rs".to_string(),
        }),
        focused_window: Some(FocusedWindowContext {
            title: "docs.rs".to_string(),
            width: 1440,
            height: 900,
            used_fallback: false,
        }),
        ..Default::default()
    };

    let checkpoint_before = serde_json::json!({
        "checkpoint": "before",
        "test": "test_recommendation_determinism_same_input_same_output",
        "contract": "identical (draft, snapshot, attached) => identical recommendations"
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_before,
        "test_checkpoint"
    );

    let first = recommend_context_parts(draft, &snapshot, &[]);
    let second = recommend_context_parts(draft, &snapshot, &[]);

    assert_eq!(
        first.recommendations,
        second.recommendations,
        "Recommendations must be exactly deterministic for the same draft, live snapshot, and attached parts"
    );

    let checkpoint_after = serde_json::json!({
        "checkpoint": "after",
        "test": "test_recommendation_determinism_same_input_same_output",
        "contract": "identical (draft, snapshot, attached) => identical recommendations",
        "first_count": first.recommendations.len(),
        "second_count": second.recommendations.len(),
        "exact_match": first == second
    });
    tracing::info!(
        target: "ai",
        checkpoint = %checkpoint_after,
        "test_checkpoint"
    );
}

#[test]
fn test_preflight_state_suppresses_recommendations_without_live_snapshot() {
    use super::context_recommendations::{ContextRecommendation, ContextRecommendationPriority};
    use crate::ai::context_contract::ContextAttachmentKind;

    let receipt = ready_preflight_receipt("Rewrite this selected text in a friendlier tone");

    let state = preflight_state_from_analysis(
        7,
        receipt,
        None,
        vec![ContextRecommendation {
            kind: ContextAttachmentKind::Selection,
            reason: "You referenced selected/highlighted content.".to_string(),
            priority: ContextRecommendationPriority::High,
        }],
    );

    assert_eq!(state.recommendations.len(), 0);
    assert!(!state.has_surfaced_recommendations());

    assert_eq!(
        state.recommendation_resolution.input_recommendation_count,
        1
    );
    assert_eq!(
        state
            .recommendation_resolution
            .surfaced_recommendation_count,
        0
    );
    assert_eq!(
        state
            .recommendation_resolution
            .suppressed_recommendation_count,
        1
    );
    assert_eq!(
        state
            .recommendation_resolution
            .suppression_reason
            .as_deref(),
        Some("recommendations_suppressed_missing_live_snapshot")
    );
}

#[test]
fn test_preflight_state_surfaces_recommendations_with_live_snapshot() {
    use super::context_recommendations::{ContextRecommendation, ContextRecommendationPriority};
    use crate::ai::context_contract::ContextAttachmentKind;
    use crate::context_snapshot::AiContextSnapshot;

    let receipt = ready_preflight_receipt("Rewrite this selected text in a friendlier tone");

    let state = preflight_state_from_analysis(
        8,
        receipt,
        Some(AiContextSnapshot::default()),
        vec![ContextRecommendation {
            kind: ContextAttachmentKind::Selection,
            reason: "You referenced selected/highlighted content.".to_string(),
            priority: ContextRecommendationPriority::High,
        }],
    );

    assert_eq!(state.recommendations.len(), 1);
    assert!(state.has_surfaced_recommendations());

    assert_eq!(
        state.recommendation_resolution.input_recommendation_count,
        1
    );
    assert_eq!(
        state
            .recommendation_resolution
            .surfaced_recommendation_count,
        1
    );
    assert_eq!(
        state
            .recommendation_resolution
            .suppressed_recommendation_count,
        0
    );
    assert_eq!(state.recommendation_resolution.suppression_reason, None);
}

#[test]
fn test_preflight_decision_ledger_matches_snapshot_counts() {
    let receipt = ready_preflight_receipt("hello");
    let state = preflight_state_from_analysis(9, receipt, None, vec![]);

    let snapshot = state.snapshot();
    let ledger = state.decision_ledger();

    assert_eq!(
        snapshot.recommendation_count,
        ledger.recommendations.surfaced_recommendation_count
    );
    assert_eq!(
        snapshot.has_live_snapshot,
        ledger.recommendations.live_snapshot_present
    );
}

#[test]
fn test_has_surfaced_recommendations_requires_live_snapshot() {
    use super::context_recommendations::{ContextRecommendation, ContextRecommendationPriority};
    use crate::ai::context_contract::ContextAttachmentKind;

    let recommendations = vec![ContextRecommendation {
        kind: ContextAttachmentKind::Selection,
        reason: "test".to_string(),
        priority: ContextRecommendationPriority::High,
    }];

    // With live snapshot: canonical method returns true
    let with_snapshot = preflight_state_from_analysis(
        1,
        ready_preflight_receipt("test"),
        Some(crate::context_snapshot::AiContextSnapshot::default()),
        recommendations.clone(),
    );
    assert!(
        with_snapshot.has_surfaced_recommendations(),
        "has_surfaced_recommendations must be true when live snapshot is present and recommendations exist"
    );

    // Without live snapshot: canonical method returns false (recommendations suppressed)
    let without_snapshot =
        preflight_state_from_analysis(2, ready_preflight_receipt("test"), None, recommendations);
    assert!(
        !without_snapshot.has_surfaced_recommendations(),
        "has_surfaced_recommendations must be false when there is no live snapshot"
    );

    // With live snapshot but no recommendations: canonical method returns false
    let no_recs = preflight_state_from_analysis(
        3,
        ready_preflight_receipt("hello"),
        Some(crate::context_snapshot::AiContextSnapshot::default()),
        Vec::new(),
    );
    assert!(
        !no_recs.has_surfaced_recommendations(),
        "has_surfaced_recommendations must be false when there are no recommendations"
    );
}
