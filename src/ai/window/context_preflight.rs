use super::*;

/// Status of the pre-submit context preflight check.
///
/// Mirrors [`PreparedMessageDecision`] but adds lifecycle states
/// (`Idle`, `Loading`) so the UI can show spinners and transitions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextPreflightStatus {
    /// No preflight has been requested yet (or it was cleared).
    #[default]
    Idle,
    /// A preflight resolution is in progress.
    Loading,
    /// All context parts resolved successfully.
    Ready,
    /// Some parts failed, but the message can still be sent.
    Partial,
    /// All parts failed — the message cannot be sent.
    Blocked,
}

/// Snapshot of a pre-submit context preflight run.
///
/// This is derived entirely from [`PreparedMessageReceipt`] so it can
/// never drift from the real submit path. The struct exists to give
/// render code a cheap, read-only view without cloning the full receipt.
#[derive(Debug, Clone, Default)]
pub struct ContextPreflightState {
    /// Monotonically increasing generation counter.
    /// Each call to `schedule_context_preflight` bumps this value;
    /// stale completions (where `gen != self.preflight_generation`) are dropped.
    pub generation: u64,

    /// Current lifecycle status.
    pub status: ContextPreflightStatus,

    /// Number of context parts that were sent to the resolver.
    pub attempted: usize,

    /// Number of parts that resolved successfully.
    pub resolved: usize,

    /// Number of parts that failed to resolve.
    pub failures: usize,

    /// Duplicates removed during the assembly/merge phase.
    pub duplicates_removed: usize,

    /// Approximate token count derived from `estimate_tokens_from_text`
    /// applied to the resolved prompt prefix. Labeled as approximate
    /// because we don't have access to a provider-specific tokenizer.
    pub approx_tokens: usize,

    /// Character count of the resolved prompt prefix.
    pub prompt_chars: usize,

    /// The full receipt, stored for drawer/inspector views.
    pub receipt: Option<crate::ai::message_parts::PreparedMessageReceipt>,
}

/// Rough token estimate: divide character count by 4 (the widely-used
/// English-text heuristic for BPE tokenizers). This is intentionally
/// labeled "approximate" everywhere it surfaces in the UI.
pub fn estimate_tokens_from_text(text: &str) -> usize {
    let char_count = text.chars().count();
    ((char_count as f64) / 4.0).ceil() as usize
}

/// Map a [`PreparedMessageDecision`] to a [`ContextPreflightStatus`].
pub fn status_from_decision(
    decision: &crate::ai::message_parts::PreparedMessageDecision,
) -> ContextPreflightStatus {
    match decision {
        crate::ai::message_parts::PreparedMessageDecision::Ready => ContextPreflightStatus::Ready,
        crate::ai::message_parts::PreparedMessageDecision::Partial => {
            ContextPreflightStatus::Partial
        }
        crate::ai::message_parts::PreparedMessageDecision::Blocked => {
            ContextPreflightStatus::Blocked
        }
    }
}

/// Derive a [`ContextPreflightState`] from a [`PreparedMessageReceipt`].
///
/// This is the canonical way to build preflight state from the same
/// receipt pipeline used at submit time.
pub fn preflight_state_from_receipt(
    generation: u64,
    receipt: crate::ai::message_parts::PreparedMessageReceipt,
) -> ContextPreflightState {
    let duplicates_removed = receipt
        .assembly
        .as_ref()
        .map(|a| a.duplicates_removed)
        .unwrap_or(0);
    let prompt_chars = receipt.context.prompt_prefix.chars().count();
    let approx_tokens = estimate_tokens_from_text(&receipt.context.prompt_prefix);
    let status = status_from_decision(&receipt.decision);

    ContextPreflightState {
        generation,
        status,
        attempted: receipt.context.attempted,
        resolved: receipt.context.resolved,
        failures: receipt.context.failures.len(),
        duplicates_removed,
        approx_tokens,
        prompt_chars,
        receipt: Some(receipt),
    }
}

impl AiApp {
    /// Schedule a context preflight check.
    ///
    /// Bumps the generation counter, sets status to `Loading`, then
    /// spawns a background task that runs the canonical resolution
    /// pipeline (the same code path used at submit time). When the
    /// task completes, stale results (generation mismatch) are
    /// silently dropped, ensuring fast typing never sees outdated
    /// preflight results.
    pub(super) fn schedule_context_preflight(
        &mut self,
        raw_content: String,
        cx: &mut Context<Self>,
    ) {
        // Bump generation to invalidate any in-flight preflight
        self.context_preflight.generation = self.context_preflight.generation.wrapping_add(1);
        let generation = self.context_preflight.generation;

        // Snapshot the pending parts for the preflight run
        let parts_snapshot: Vec<crate::ai::message_parts::AiContextPart> =
            self.pending_context_parts.clone();

        // Fast path: nothing to preflight
        if parts_snapshot.is_empty() && raw_content.trim().is_empty() {
            self.context_preflight = ContextPreflightState {
                generation,
                status: ContextPreflightStatus::Idle,
                ..Default::default()
            };
            tracing::info!(
                target: "ai",
                generation,
                "ai_context_preflight_cleared"
            );
            cx.notify();
            return;
        }

        self.context_preflight.status = ContextPreflightStatus::Loading;
        cx.notify();

        // Spawn the resolution work so it doesn't block the UI thread.
        // The resolution pipeline (file reads, MCP resource queries) is
        // the same code path used at submit time. We run it in a
        // background task and apply results via cx.update(), guarding
        // against stale generations.
        cx.spawn(async move |this, cx| {
            // Run the expensive resolution on the background executor
            let receipt = cx
                .background_executor()
                .spawn(async move {
                    let parsed = crate::ai::context_mentions::parse_context_mentions(&raw_content);
                    let scripts = crate::scripts::read_scripts();
                    let scriptlets = crate::scripts::load_scriptlets();

                    crate::ai::message_parts::prepare_user_message_from_sources_with_receipt(
                        &parsed.cleaned_content,
                        &parsed.parts,
                        &parts_snapshot,
                        &scripts,
                        &scriptlets,
                    )
                })
                .await;

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    // Guard: only apply if this is still the current generation
                    if app.context_preflight.generation != generation {
                        tracing::info!(
                            target: "ai",
                            generation,
                            current_generation = app.context_preflight.generation,
                            "ai_context_preflight_stale_dropped"
                        );
                        return;
                    }

                    app.context_preflight = preflight_state_from_receipt(generation, receipt);

                    tracing::info!(
                        target: "ai",
                        generation,
                        attempted = app.context_preflight.attempted,
                        resolved = app.context_preflight.resolved,
                        failures = app.context_preflight.failures,
                        approx_tokens = app.context_preflight.approx_tokens,
                        "ai_context_preflight_applied"
                    );

                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Reset the preflight state to `Idle` and bump the generation so any
    /// in-flight async work becomes stale.
    pub(super) fn clear_context_preflight(&mut self, cx: &mut Context<Self>) {
        self.context_preflight.generation = self.context_preflight.generation.wrapping_add(1);
        let generation = self.context_preflight.generation;
        self.context_preflight = ContextPreflightState {
            generation,
            status: ContextPreflightStatus::Idle,
            ..Default::default()
        };
        tracing::info!(
            target: "ai",
            generation,
            "ai_context_preflight_cleared"
        );
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_from_text_empty_input_returns_zero() {
        assert_eq!(
            estimate_tokens_from_text(""),
            0,
            "Empty string should produce zero tokens"
        );
    }

    #[test]
    fn test_estimate_tokens_from_text_single_char_rounds_up() {
        assert_eq!(
            estimate_tokens_from_text("a"),
            1,
            "Single character should ceil to 1 token"
        );
    }

    #[test]
    fn test_estimate_tokens_from_text_exact_multiple() {
        assert_eq!(
            estimate_tokens_from_text("abcd"),
            1,
            "Exactly 4 chars should produce 1 token"
        );
    }

    #[test]
    fn test_estimate_tokens_from_text_rounds_up_partial() {
        // 5 chars / 4 = 1.25 → ceil = 2
        assert_eq!(
            estimate_tokens_from_text("abcde"),
            2,
            "5 chars should ceil to 2 tokens"
        );
    }

    #[test]
    fn test_estimate_tokens_from_text_multibyte_counts_chars_not_bytes() {
        // "café" = 4 chars (c, a, f, é) but 5 bytes in UTF-8
        let text = "café";
        assert_eq!(text.len(), 5, "Sanity: UTF-8 byte length should be 5");
        assert_eq!(
            estimate_tokens_from_text(text),
            1,
            "Token estimate should use char count (4), not byte count (5)"
        );
    }

    #[test]
    fn test_context_preflight_status_defaults_to_idle() {
        assert_eq!(
            ContextPreflightStatus::default(),
            ContextPreflightStatus::Idle,
            "Default preflight status should be Idle"
        );
    }

    #[test]
    fn test_context_preflight_state_defaults_are_zeroed() {
        let state = ContextPreflightState::default();
        assert_eq!(state.generation, 0);
        assert_eq!(state.status, ContextPreflightStatus::Idle);
        assert_eq!(state.attempted, 0);
        assert_eq!(state.resolved, 0);
        assert_eq!(state.failures, 0);
        assert_eq!(state.duplicates_removed, 0);
        assert_eq!(state.approx_tokens, 0);
        assert_eq!(state.prompt_chars, 0);
        assert!(state.receipt.is_none());
    }

    #[test]
    fn test_duplicate_parts_do_not_inflate_budget_after_merge() {
        // One part in mentions, one identical part in pending → merge
        // should dedup to a single part.
        let part = crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        };

        let part2 = part.clone();
        let assembly = crate::ai::message_parts::merge_context_parts_with_receipt(
            std::slice::from_ref(&part),
            std::slice::from_ref(&part2),
        );

        assert_eq!(
            assembly.duplicates_removed, 1,
            "Identical parts from mention and pending should produce exactly one duplicate"
        );
        assert_eq!(
            assembly.merged_count, 1,
            "Merged output should contain only the unique part"
        );
    }

    #[test]
    fn test_status_from_decision_maps_all_variants() {
        assert_eq!(
            status_from_decision(&crate::ai::message_parts::PreparedMessageDecision::Ready),
            ContextPreflightStatus::Ready
        );
        assert_eq!(
            status_from_decision(&crate::ai::message_parts::PreparedMessageDecision::Partial),
            ContextPreflightStatus::Partial
        );
        assert_eq!(
            status_from_decision(&crate::ai::message_parts::PreparedMessageDecision::Blocked),
            ContextPreflightStatus::Blocked
        );
    }

    #[test]
    fn test_preflight_state_from_receipt_derives_token_count() {
        let receipt = crate::ai::message_parts::PreparedMessageReceipt {
            schema_version: crate::ai::message_parts::AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
            decision: crate::ai::message_parts::PreparedMessageDecision::Ready,
            raw_content: "test".to_string(),
            final_user_content: "prefix\n\ntest".to_string(),
            context: crate::ai::message_parts::ContextResolutionReceipt {
                attempted: 1,
                resolved: 1,
                failures: vec![],
                // 20 chars → 5 tokens
                prompt_prefix: "a]".repeat(10),
            },
            assembly: Some(crate::ai::message_parts::ContextAssemblyReceipt {
                mention_count: 0,
                pending_count: 1,
                merged_count: 1,
                duplicates_removed: 0,
                duplicates: vec![],
                merged_parts: vec![],
            }),
            outcomes: vec![],
            unresolved_parts: vec![],
            user_error: None,
        };

        let state = preflight_state_from_receipt(42, receipt);
        assert_eq!(state.generation, 42);
        assert_eq!(state.status, ContextPreflightStatus::Ready);
        assert_eq!(state.attempted, 1);
        assert_eq!(state.resolved, 1);
        assert_eq!(state.failures, 0);
        assert_eq!(state.duplicates_removed, 0);
        assert_eq!(state.prompt_chars, 20);
        assert_eq!(state.approx_tokens, 5);
        assert!(state.receipt.is_some());
    }

    #[test]
    fn test_preflight_state_from_receipt_blocked_has_zero_tokens() {
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
            user_error: Some("all failed".to_string()),
        };

        let state = preflight_state_from_receipt(99, receipt);
        assert_eq!(state.status, ContextPreflightStatus::Blocked);
        assert_eq!(state.resolved, 0);
        assert_eq!(state.failures, 2);
        assert_eq!(state.approx_tokens, 0);
    }

    #[test]
    fn test_generation_wrapping_does_not_panic() {
        let max_gen = u64::MAX;
        let wrapped = max_gen.wrapping_add(1);
        assert_eq!(
            wrapped, 0,
            "Generation counter should wrap to 0 at u64::MAX"
        );
    }
}
