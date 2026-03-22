use crate::ai::message_parts::{
    ContextPartPreparationOutcomeKind, PreparedMessageDecision, PreparedMessageReceipt,
};
use crate::ai::window::context_preflight::estimate_tokens_from_text;

/// High-level send readiness derived from `PreparedMessageDecision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromptCompilerDecision {
    Ready,
    Partial,
    Blocked,
}

/// Classification for each row in the compiler preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromptCompilerRowKind {
    FullContent,
    MetadataOnly,
    Failed,
    DuplicateDropped,
    UnresolvedPart,
}

/// A single line-item in the prompt compiler preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptCompilerRow {
    pub(crate) kind: PromptCompilerRowKind,
    pub(crate) label: String,
    pub(crate) source: String,
    pub(crate) detail: Option<String>,
}

/// Human-readable view model derived from a `PreparedMessageReceipt`.
///
/// `final_user_content` exactly matches the payload that submit stores/sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptCompilerPreview {
    pub(crate) decision: PromptCompilerDecision,
    pub(crate) raw_content: String,
    pub(crate) prompt_prefix: String,
    pub(crate) final_user_content: String,
    pub(crate) attempted: usize,
    pub(crate) resolved: usize,
    pub(crate) failures: usize,
    pub(crate) duplicates_removed: usize,
    pub(crate) approx_tokens: usize,
    pub(crate) rows: Vec<PromptCompilerRow>,
}

impl PromptCompilerPreview {
    /// Build a preview from an existing receipt. Pure data transformation —
    /// no IO, no side-effects.
    pub(crate) fn from_receipt(receipt: &PreparedMessageReceipt) -> Self {
        let mut rows = Vec::new();

        // Duplicate-dropped rows (from assembly dedup)
        if let Some(assembly) = &receipt.assembly {
            for dup in &assembly.duplicates {
                rows.push(PromptCompilerRow {
                    kind: PromptCompilerRowKind::DuplicateDropped,
                    label: dup.label.clone(),
                    source: dup.source.clone(),
                    detail: Some(format!(
                        "kept_from={:?} dropped_from={:?}",
                        dup.kept_from, dup.dropped_from
                    )),
                });
            }
        }

        // Per-part outcome rows
        for outcome in &receipt.outcomes {
            let kind = match outcome.kind {
                ContextPartPreparationOutcomeKind::FullContent => {
                    PromptCompilerRowKind::FullContent
                }
                ContextPartPreparationOutcomeKind::MetadataOnly => {
                    PromptCompilerRowKind::MetadataOnly
                }
                ContextPartPreparationOutcomeKind::Failed => PromptCompilerRowKind::Failed,
            };
            rows.push(PromptCompilerRow {
                kind,
                label: outcome.label.clone(),
                source: outcome.source.clone(),
                detail: outcome.detail.clone(),
            });
        }

        // Unresolved parts that were still pending at submit time
        for part in &receipt.unresolved_parts {
            rows.push(PromptCompilerRow {
                kind: PromptCompilerRowKind::UnresolvedPart,
                label: part.label().to_string(),
                source: part.source().to_string(),
                detail: Some("unresolved at submit time".to_string()),
            });
        }

        let decision = match receipt.decision {
            PreparedMessageDecision::Ready => PromptCompilerDecision::Ready,
            PreparedMessageDecision::Partial => PromptCompilerDecision::Partial,
            PreparedMessageDecision::Blocked => PromptCompilerDecision::Blocked,
        };

        Self {
            decision,
            raw_content: receipt.raw_content.clone(),
            prompt_prefix: receipt.context.prompt_prefix.clone(),
            final_user_content: receipt.final_user_content.clone(),
            attempted: receipt.context.attempted,
            resolved: receipt.context.resolved,
            failures: receipt.context.failures.len(),
            duplicates_removed: receipt
                .assembly
                .as_ref()
                .map_or(0, |a| a.duplicates_removed),
            approx_tokens: estimate_tokens_from_text(&receipt.final_user_content),
            rows,
        }
    }

    /// Serializable snapshot for deterministic verification without screenshots.
    pub(crate) fn snapshot(&self) -> PromptCompilerSnapshot {
        PromptCompilerSnapshot {
            decision: format!("{:?}", self.decision),
            attempted: self.attempted,
            resolved: self.resolved,
            failures: self.failures,
            duplicates_removed: self.duplicates_removed,
            approx_tokens: self.approx_tokens,
            raw_content_len: self.raw_content.len(),
            prompt_prefix_len: self.prompt_prefix.len(),
            final_user_content_len: self.final_user_content.len(),
            row_count: self.rows.len(),
        }
    }
}

/// Machine-readable snapshot for deterministic test assertions and
/// future copy/export actions.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub(crate) struct PromptCompilerSnapshot {
    pub(crate) decision: String,
    pub(crate) attempted: usize,
    pub(crate) resolved: usize,
    pub(crate) failures: usize,
    pub(crate) duplicates_removed: usize,
    pub(crate) approx_tokens: usize,
    pub(crate) raw_content_len: usize,
    pub(crate) prompt_prefix_len: usize,
    pub(crate) final_user_content_len: usize,
    pub(crate) row_count: usize,
}
