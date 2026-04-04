use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Canonical label for the Ask Anything ambient context chip.
pub const ASK_ANYTHING_LABEL: &str = "Ask Anything";

/// Canonical resource URI for the Ask Anything minimal desktop context.
pub const ASK_ANYTHING_RESOURCE_URI: &str = "kit://context?profile=minimal";

const DEFERRED_AMBIENT_CAPTURE_LABELS: &[&str] = &[
    ASK_ANYTHING_LABEL,
    "Full Screen",
    "Focused Window",
    "Selected Text",
    "Browser Tab",
];

/// A typed context part that can be attached to an AI composer message.
///
/// Each variant represents a different source of context that will be
/// resolved into a prompt block at submit time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum AiContextPart {
    /// An MCP resource URI (e.g. `kit://context?profile=minimal`)
    ResourceUri { uri: String, label: String },
    /// A local file path attachment
    FilePath { path: String, label: String },
    /// A focused UI target resolved from the active surface (e.g. a selected
    /// script, clipboard entry, or file). Carries the full target context so
    /// it can be rendered as a chip and resolved into a deterministic prompt
    /// block at submit time.
    FocusedTarget {
        target: crate::ai::tab_context::TabAiTargetContext,
        label: String,
    },
    /// Display-only ambient context chip. Represents promoted Ask Anything
    /// context that has already been staged as `pending_context_blocks`.
    /// Resolves to an empty prompt block (the real content lives in the
    /// staged blocks).
    AmbientContext { label: String },
}

impl AiContextPart {
    pub fn label(&self) -> &str {
        match self {
            Self::ResourceUri { label, .. }
            | Self::FilePath { label, .. }
            | Self::FocusedTarget { label, .. }
            | Self::AmbientContext { label } => label,
        }
    }

    /// Returns the originating URI or file path for this context part.
    pub fn source(&self) -> &str {
        match self {
            Self::ResourceUri { uri, .. } => uri,
            Self::FilePath { path, .. } => path,
            Self::FocusedTarget { target, .. } => &target.semantic_id,
            Self::AmbientContext { .. } => "ambient://ask-anything",
        }
    }

    /// Returns `true` when this part is the initial Ask Anything resource
    /// chip (before promotion to `AmbientContext`).
    pub fn is_ask_anything_resource(&self) -> bool {
        matches!(
            self,
            Self::ResourceUri { uri, label }
                if uri == ASK_ANYTHING_RESOURCE_URI && label == ASK_ANYTHING_LABEL
        )
    }

    /// Returns `true` when this part is a promoted ambient context chip.
    pub fn is_ambient_context_chip(&self) -> bool {
        matches!(self, Self::AmbientContext { .. })
    }

    /// Returns `true` only for resource chips that must wait on a deferred
    /// ambient capture task before submit.
    ///
    /// Inline picker attachments such as `@context` also use the minimal
    /// desktop-context URI, but they should resolve directly on submit rather
    /// than entering the ambient bootstrap state machine.
    pub fn is_ambient_bootstrap_resource(&self) -> bool {
        matches!(
            self,
            Self::ResourceUri { uri, label }
                if uri == ASK_ANYTHING_RESOURCE_URI
                    && DEFERRED_AMBIENT_CAPTURE_LABELS.contains(&label.as_str())
        )
    }

    /// Return the display label for an ambient bootstrap or promoted ambient chip.
    pub fn ambient_chip_label(&self) -> Option<&str> {
        match self {
            Self::ResourceUri { uri, label } if uri == ASK_ANYTHING_RESOURCE_URI => {
                Some(label.as_str())
            }
            Self::AmbientContext { label } => Some(label.as_str()),
            _ => None,
        }
    }
}

/// Extract file paths from a slice of context parts.
///
/// Returns only the `path` values from `AiContextPart::FilePath` variants,
/// preserving order. This is the canonical way to derive the attachment list
/// from the single source of truth (`pending_context_parts`).
pub fn file_path_parts(parts: &[AiContextPart]) -> Vec<String> {
    parts
        .iter()
        .filter_map(|part| match part {
            AiContextPart::FilePath { path, .. } => Some(path.clone()),
            _ => None,
        })
        .collect()
}

/// Records a single context part that failed to resolve.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextResolutionFailure {
    pub label: String,
    pub source: String,
    pub error: String,
}

/// A deterministic receipt produced by resolving a set of context parts.
///
/// Captures how many parts were attempted, how many resolved successfully,
/// any failures encountered, and the concatenated prompt prefix from all
/// successful resolutions. Successful blocks are never dropped when another
/// part fails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextResolutionReceipt {
    pub attempted: usize,
    pub resolved: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<ContextResolutionFailure>,
    pub prompt_prefix: String,
}

impl ContextResolutionReceipt {
    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

/// Resolve a single context part into a prompt block string.
///
/// - `ResourceUri` resolves via `mcp_resources::read_resource`.
/// - `FilePath` reads the file; falls back to metadata-only if unreadable.
pub fn resolve_context_part_to_prompt_block(
    part: &AiContextPart,
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> Result<String> {
    match part {
        AiContextPart::ResourceUri { uri, .. } => {
            let content = crate::mcp_resources::read_resource(uri, scripts, scriptlets, None)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("Failed to read MCP resource: {uri}"))?;

            tracing::info!(
                kind = "resource_uri",
                uri = %content.uri,
                mime_type = %content.mime_type,
                "Resolved resource URI context part"
            );

            Ok(format!(
                "<context source=\"{}\" mimeType=\"{}\">\n{}\n</context>",
                content.uri, content.mime_type, content.text
            ))
        }
        AiContextPart::FilePath { path, .. } => match std::fs::read_to_string(path) {
            Ok(text) => {
                tracing::info!(
                    kind = "file_path_readable",
                    path = %path,
                    bytes = text.len(),
                    "Resolved file path context part"
                );
                Ok(format!(
                    "<attachment path=\"{}\">\n{}\n</attachment>",
                    path, text
                ))
            }
            Err(_) => {
                let metadata = std::fs::metadata(path)
                    .with_context(|| format!("Failed to stat attachment: {path}"))?;

                tracing::info!(
                    kind = "file_path_unreadable",
                    path = %path,
                    bytes = metadata.len(),
                    "Resolved unreadable file path context part (metadata-only fallback)"
                );

                Ok(format!(
                    "<attachment path=\"{}\" unreadable=\"true\" bytes=\"{}\" />",
                    path,
                    metadata.len()
                ))
            }
        },
        AiContextPart::FocusedTarget { target, label } => {
            resolve_focused_target_part(target, label)
        }
        AiContextPart::AmbientContext { label } => {
            tracing::info!(
                kind = "ambient_context_display_only",
                label = %label,
                "Skipped display-only ambient context chip during prompt resolution"
            );
            Ok(String::new())
        }
    }
}

/// Resolve a `FocusedTarget` context part into a deterministic prompt block.
///
/// The block includes target source, kind, semantic ID, label, and serialized
/// metadata so the agent can unambiguously identify the focused subject.
fn resolve_focused_target_part(
    target: &crate::ai::tab_context::TabAiTargetContext,
    label: &str,
) -> Result<String> {
    let metadata_json = target
        .metadata
        .as_ref()
        .map(serde_json::to_string_pretty)
        .transpose()
        .context("Failed to serialize focused target metadata")?
        .unwrap_or_else(|| "{}".to_string());

    tracing::info!(
        kind = "focused_target",
        source = %target.source,
        item_kind = %target.kind,
        semantic_id = %target.semantic_id,
        label = %label,
        "Resolved focused target context part"
    );

    Ok(format!(
        "<context source=\"focusedTarget\" itemSource=\"{}\" itemKind=\"{}\" semanticId=\"{}\">\nLabel: {}\nMetadata:\n{}\n</context>",
        target.source, target.kind, target.semantic_id, label, metadata_json,
    ))
}

/// Resolve multiple context parts, returning a machine-readable receipt.
///
/// Successful resolutions are never dropped when another part fails.
/// The `prompt_prefix` contains all successfully resolved blocks joined
/// by double newlines.
pub fn resolve_context_parts_with_receipt(
    parts: &[AiContextPart],
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> ContextResolutionReceipt {
    let attempted = parts.len();
    let mut blocks = Vec::new();
    let mut failures = Vec::new();

    for part in parts {
        match resolve_context_part_to_prompt_block(part, scripts, scriptlets) {
            Ok(block) => {
                if block.trim().is_empty() {
                    tracing::info!(
                        checkpoint = "resolution_display_only",
                        source = %part.source(),
                        label = %part.label(),
                        "context part produced no prompt block"
                    );
                    continue;
                }
                tracing::info!(
                    checkpoint = "resolution_ok",
                    source = %part.source(),
                    label = %part.label(),
                    "context part resolved successfully"
                );
                blocks.push(block);
            }
            Err(err) => {
                tracing::warn!(
                    checkpoint = "resolution_failed",
                    source = %part.source(),
                    label = %part.label(),
                    error = %err,
                    "context part resolution failed"
                );
                failures.push(ContextResolutionFailure {
                    label: part.label().to_string(),
                    source: part.source().to_string(),
                    error: format!("{err:#}"),
                });
            }
        }
    }

    let resolved = blocks.len();
    let prompt_prefix = blocks.join("\n\n");

    ContextResolutionReceipt {
        attempted,
        resolved,
        failures,
        prompt_prefix,
    }
}

/// Resolve multiple context parts into a single prompt prefix string.
///
/// This is a compatibility wrapper around [`resolve_context_parts_with_receipt`].
/// It returns the prompt prefix from all successfully resolved parts. If any
/// part fails, the error is returned and successful blocks are lost — prefer
/// the receipt-based API for partial-failure tolerance.
pub fn resolve_context_parts_to_prompt_prefix(
    parts: &[AiContextPart],
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> Result<String> {
    let receipt = resolve_context_parts_with_receipt(parts, scripts, scriptlets);
    if receipt.has_failures() {
        let first = &receipt.failures[0];
        anyhow::bail!(
            "Failed to resolve context part '{}' ({}): {}",
            first.label,
            first.source,
            first.error
        );
    }
    Ok(receipt.prompt_prefix)
}

/// Provenance tag for a context part in the assembly pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContextAssemblyOrigin {
    /// Part came from parsed `@context` / `@file` directives in the message text.
    Mention,
    /// Part came from the pending context chips (UI or SDK).
    Pending,
}

/// A duplicate that was dropped during context assembly, with provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextAssemblyDuplicate {
    pub kept_from: ContextAssemblyOrigin,
    pub dropped_from: ContextAssemblyOrigin,
    pub label: String,
    pub source: String,
}

/// Deterministic receipt from merging mention-derived and pending context parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextAssemblyReceipt {
    pub mention_count: usize,
    pub pending_count: usize,
    pub merged_count: usize,
    pub duplicates_removed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duplicates: Vec<ContextAssemblyDuplicate>,
    pub merged_parts: Vec<AiContextPart>,
}

/// Merge mention-derived and pending context parts with full provenance tracking.
///
/// Returns a [`ContextAssemblyReceipt`] recording which parts survived, which
/// duplicates were dropped, and where each came from. Mentions are processed
/// first so they take priority in first-seen deduplication.
pub(crate) fn merge_context_parts_with_receipt(
    mentions: &[AiContextPart],
    pending: &[AiContextPart],
) -> ContextAssemblyReceipt {
    let mut merged = Vec::with_capacity(mentions.len() + pending.len());
    let mut origins = Vec::with_capacity(mentions.len() + pending.len());
    let mut duplicates = Vec::new();

    for (origin, part) in mentions
        .iter()
        .map(|part| (ContextAssemblyOrigin::Mention, part))
        .chain(
            pending
                .iter()
                .map(|part| (ContextAssemblyOrigin::Pending, part)),
        )
    {
        if let Some(existing_idx) = merged.iter().position(|existing| existing == part) {
            duplicates.push(ContextAssemblyDuplicate {
                kept_from: origins[existing_idx],
                dropped_from: origin,
                label: part.label().to_string(),
                source: part.source().to_string(),
            });
            continue;
        }

        merged.push(part.clone());
        origins.push(origin);
    }

    let receipt = ContextAssemblyReceipt {
        mention_count: mentions.len(),
        pending_count: pending.len(),
        merged_count: merged.len(),
        duplicates_removed: duplicates.len(),
        duplicates,
        merged_parts: merged,
    };

    tracing::info!(
        target: "ai",
        checkpoint = "context_assembly",
        mention_count = receipt.mention_count,
        pending_count = receipt.pending_count,
        merged_count = receipt.merged_count,
        duplicates_removed = receipt.duplicates_removed,
        "context parts assembled"
    );

    receipt
}

/// Merge two slices of context parts into a single list with first-seen order
/// preserved and duplicates removed by value equality.
///
/// This is a backward-compatible wrapper around [`merge_context_parts_with_receipt`].
/// It treats `left` as mentions and `right` as pending parts, returning only the
/// merged list without provenance metadata.
pub fn merge_context_parts(left: &[AiContextPart], right: &[AiContextPart]) -> Vec<AiContextPart> {
    merge_context_parts_with_receipt(left, right).merged_parts
}

// ---------------------------------------------------------------------------
// Schema-versioned message-preparation receipt
// ---------------------------------------------------------------------------

pub const AI_MESSAGE_PREPARATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PreparedMessageDecision {
    Ready,
    Partial,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContextPartPreparationOutcomeKind {
    FullContent,
    MetadataOnly,
    Failed,
    /// Display-only chip that produces no prompt block (e.g. `AmbientContext`).
    DisplayOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextPartPreparationOutcome {
    pub label: String,
    pub source: String,
    pub kind: ContextPartPreparationOutcomeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMessageReceipt {
    pub schema_version: u32,
    pub decision: PreparedMessageDecision,
    pub raw_content: String,
    pub final_user_content: String,
    pub context: ContextResolutionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assembly: Option<ContextAssemblyReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<ContextPartPreparationOutcome>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_parts: Vec<AiContextPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_error: Option<String>,
}

impl PreparedMessageReceipt {
    pub fn can_send_message(&self) -> bool {
        self.decision != PreparedMessageDecision::Blocked
    }
}

/// Join a resolved prompt prefix with raw user content.
fn join_prompt_prefix_and_raw_content(prompt_prefix: &str, raw_content: &str) -> String {
    if !prompt_prefix.is_empty() && !raw_content.trim().is_empty() {
        format!("{prompt_prefix}\n\n{raw_content}")
    } else if !prompt_prefix.is_empty() {
        prompt_prefix.to_string()
    } else {
        raw_content.to_string()
    }
}

/// Build a user-visible error string from resolution failures.
fn build_user_visible_context_error(failures: &[ContextResolutionFailure]) -> Option<String> {
    if failures.is_empty() {
        None
    } else {
        Some(format!(
            "Failed to resolve context: {}",
            failures
                .iter()
                .map(|f| format!("{}: {}", f.label, f.error))
                .collect::<Vec<_>>()
                .join("; ")
        ))
    }
}

/// Resolve a single context part, returning both the per-part outcome and
/// an optional prompt block. Unlike [`resolve_context_part_to_prompt_block`],
/// this distinguishes full-content from metadata-only successes.
fn resolve_context_part_for_preparation(
    part: &AiContextPart,
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> (ContextPartPreparationOutcome, Option<String>) {
    match part {
        AiContextPart::ResourceUri { uri, label } => {
            match crate::mcp_resources::read_resource(uri, scripts, scriptlets, None) {
                Ok(content) => {
                    let prompt_block = format!(
                        "<context source=\"{}\" mimeType=\"{}\">\n{}\n</context>",
                        content.uri, content.mime_type, content.text
                    );
                    (
                        ContextPartPreparationOutcome {
                            label: label.clone(),
                            source: uri.clone(),
                            kind: ContextPartPreparationOutcomeKind::FullContent,
                            detail: Some(format!("mimeType={}", content.mime_type)),
                        },
                        Some(prompt_block),
                    )
                }
                Err(error) => (
                    ContextPartPreparationOutcome {
                        label: label.clone(),
                        source: uri.clone(),
                        kind: ContextPartPreparationOutcomeKind::Failed,
                        detail: Some(format!("Failed to read MCP resource: {uri}: {error}")),
                    },
                    None,
                ),
            }
        }
        AiContextPart::FilePath { path, label } => match std::fs::read_to_string(path) {
            Ok(text) => {
                let prompt_block =
                    format!("<attachment path=\"{}\">\n{}\n</attachment>", path, text);
                (
                    ContextPartPreparationOutcome {
                        label: label.clone(),
                        source: path.clone(),
                        kind: ContextPartPreparationOutcomeKind::FullContent,
                        detail: None,
                    },
                    Some(prompt_block),
                )
            }
            Err(read_error) => match std::fs::metadata(path) {
                Ok(metadata) => {
                    let prompt_block = format!(
                        "<attachment path=\"{}\" unreadable=\"true\" bytes=\"{}\" />",
                        path,
                        metadata.len()
                    );
                    (
                        ContextPartPreparationOutcome {
                            label: label.clone(),
                            source: path.clone(),
                            kind: ContextPartPreparationOutcomeKind::MetadataOnly,
                            detail: Some(format!("textReadError={read_error}")),
                        },
                        Some(prompt_block),
                    )
                }
                Err(stat_error) => (
                    ContextPartPreparationOutcome {
                        label: label.clone(),
                        source: path.clone(),
                        kind: ContextPartPreparationOutcomeKind::Failed,
                        detail: Some(format!("Failed to stat attachment: {stat_error}")),
                    },
                    None,
                ),
            },
        },
        AiContextPart::FocusedTarget { target, label } => {
            match resolve_focused_target_part(target, label) {
                Ok(prompt_block) => (
                    ContextPartPreparationOutcome {
                        label: label.clone(),
                        source: target.semantic_id.clone(),
                        kind: ContextPartPreparationOutcomeKind::FullContent,
                        detail: Some(format!(
                            "itemSource={}, itemKind={}",
                            target.source, target.kind
                        )),
                    },
                    Some(prompt_block),
                ),
                Err(error) => (
                    ContextPartPreparationOutcome {
                        label: label.clone(),
                        source: target.semantic_id.clone(),
                        kind: ContextPartPreparationOutcomeKind::Failed,
                        detail: Some(format!("Failed to resolve focused target: {error}")),
                    },
                    None,
                ),
            }
        }
        AiContextPart::AmbientContext { label } => (
            ContextPartPreparationOutcome {
                label: label.clone(),
                source: "ambient://ask-anything".to_string(),
                kind: ContextPartPreparationOutcomeKind::DisplayOnly,
                detail: Some("Display-only ambient context chip".to_string()),
            },
            None,
        ),
    }
}

/// The single canonical message-preparation function.
///
/// Takes raw user text and pending context parts, resolves each part into
/// a prompt block (or records failure), and returns a schema-versioned
/// [`PreparedMessageReceipt`] with per-part outcomes, the aggregate
/// [`ContextResolutionReceipt`], and a final decision.
pub fn prepare_user_message_with_receipt(
    raw_content: &str,
    parts: &[AiContextPart],
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> PreparedMessageReceipt {
    let mut outcomes = Vec::with_capacity(parts.len());
    let mut failures = Vec::new();
    let mut unresolved_parts = Vec::new();
    let mut prompt_blocks = Vec::new();

    for part in parts {
        let (outcome, prompt_block) =
            resolve_context_part_for_preparation(part, scripts, scriptlets);

        if let Some(block) = prompt_block {
            prompt_blocks.push(block);
        } else if outcome.kind == ContextPartPreparationOutcomeKind::Failed {
            unresolved_parts.push(part.clone());
            failures.push(ContextResolutionFailure {
                label: outcome.label.clone(),
                source: outcome.source.clone(),
                error: outcome
                    .detail
                    .clone()
                    .unwrap_or_else(|| "unknown context resolution failure".to_string()),
            });
        }

        outcomes.push(outcome);
    }

    let context = ContextResolutionReceipt {
        attempted: parts.len(),
        resolved: prompt_blocks.len(),
        failures,
        prompt_prefix: prompt_blocks.join("\n\n"),
    };

    let final_user_content =
        join_prompt_prefix_and_raw_content(&context.prompt_prefix, raw_content);

    let decision = if context.has_failures() && context.resolved == 0 {
        PreparedMessageDecision::Blocked
    } else if context.has_failures() {
        PreparedMessageDecision::Partial
    } else {
        PreparedMessageDecision::Ready
    };

    let user_error = build_user_visible_context_error(&context.failures);

    tracing::info!(
        checkpoint = "message_prepare",
        decision = ?decision,
        attempted = context.attempted,
        resolved = context.resolved,
        failures = context.failures.len(),
        unresolved = unresolved_parts.len(),
        final_user_content_len = final_user_content.len(),
        "ai message preparation complete"
    );

    PreparedMessageReceipt {
        schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
        decision,
        raw_content: raw_content.to_string(),
        final_user_content,
        context,
        assembly: None,
        outcomes,
        unresolved_parts,
        user_error,
    }
}

/// Canonical preflight function that assembles context from two sources.
///
/// 1. Merges `mention_parts` (from parsed directives) with `pending_parts`
///    (from the composer UI / SDK), recording duplicate provenance.
/// 2. Resolves the merged parts into prompt blocks via
///    [`prepare_user_message_with_receipt`].
/// 3. Attaches the assembly receipt so callers can inspect the full pipeline.
pub fn prepare_user_message_from_sources_with_receipt(
    raw_content: &str,
    mention_parts: &[AiContextPart],
    pending_parts: &[AiContextPart],
    scripts: &[Arc<crate::scripts::Script>],
    scriptlets: &[Arc<crate::scripts::Scriptlet>],
) -> PreparedMessageReceipt {
    let assembly = merge_context_parts_with_receipt(mention_parts, pending_parts);
    let mut prepared =
        prepare_user_message_with_receipt(raw_content, &assembly.merged_parts, scripts, scriptlets);
    prepared.assembly = Some(assembly);

    tracing::info!(
        target: "ai",
        checkpoint = "message_preflight",
        decision = ?prepared.decision,
        attempted = prepared.context.attempted,
        resolved = prepared.context.resolved,
        duplicates_removed = prepared
            .assembly
            .as_ref()
            .map(|a| a.duplicates_removed)
            .unwrap_or(0),
        "ai message preflight complete"
    );

    prepared
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_roundtrip_resource_uri() {
        let part = AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let deserialized: AiContextPart = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(part, deserialized);
        assert!(json.contains("\"kind\":\"resourceUri\""));
    }

    #[test]
    fn test_serde_roundtrip_file_path() {
        let part = AiContextPart::FilePath {
            path: "/tmp/test.rs".to_string(),
            label: "test.rs".to_string(),
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let deserialized: AiContextPart = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(part, deserialized);
        assert!(json.contains("\"kind\":\"filePath\""));
    }

    #[test]
    fn test_label_accessor() {
        let uri_part = AiContextPart::ResourceUri {
            uri: "kit://context".to_string(),
            label: "Context".to_string(),
        };
        assert_eq!(uri_part.label(), "Context");

        let file_part = AiContextPart::FilePath {
            path: "/tmp/foo.rs".to_string(),
            label: "foo.rs".to_string(),
        };
        assert_eq!(file_part.label(), "foo.rs");
    }

    #[test]
    fn test_resolve_readable_file_path() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("hello.txt");
        std::fs::write(&file_path, "Hello, world!").expect("write temp file");

        let part = AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "hello.txt".to_string(),
        };

        let block =
            resolve_context_part_to_prompt_block(&part, &[], &[]).expect("resolve should succeed");

        assert!(block.contains("<attachment path=\""));
        assert!(block.contains("Hello, world!"));
        assert!(block.contains("</attachment>"));
        assert!(!block.contains("unreadable"));
    }

    #[test]
    fn test_resolve_unreadable_file_path_does_not_panic() {
        // Create a file, make it exist but unreadable by removing read permissions
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("binary.dat");
        std::fs::write(&file_path, vec![0u8; 64]).expect("write temp file");

        // On Unix, remove read permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000))
                .expect("set permissions");
        }

        let part = AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "binary.dat".to_string(),
        };

        // On unix, this should produce an unreadable fallback (metadata-only)
        #[cfg(unix)]
        {
            let block = resolve_context_part_to_prompt_block(&part, &[], &[])
                .expect("resolve should not panic");
            assert!(block.contains("unreadable=\"true\""));
            assert!(block.contains("bytes=\"64\""));
        }

        // On non-unix, file is readable, so just verify no panic
        #[cfg(not(unix))]
        {
            let _ = resolve_context_part_to_prompt_block(&part, &[], &[]);
        }

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));
        }
    }

    #[test]
    fn test_resolve_nonexistent_file_returns_error() {
        let part = AiContextPart::FilePath {
            path: "/nonexistent/path/that/does/not/exist.txt".to_string(),
            label: "ghost.txt".to_string(),
        };

        let result = resolve_context_part_to_prompt_block(&part, &[], &[]);
        assert!(result.is_err(), "nonexistent file should error");
    }

    #[test]
    fn test_resolve_multiple_parts() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file1 = dir.path().join("a.txt");
        let file2 = dir.path().join("b.txt");
        std::fs::write(&file1, "content A").expect("write");
        std::fs::write(&file2, "content B").expect("write");

        let parts = vec![
            AiContextPart::FilePath {
                path: file1.to_string_lossy().to_string(),
                label: "a.txt".to_string(),
            },
            AiContextPart::FilePath {
                path: file2.to_string_lossy().to_string(),
                label: "b.txt".to_string(),
            },
        ];

        let prefix =
            resolve_context_parts_to_prompt_prefix(&parts, &[], &[]).expect("resolve prefix");
        assert!(prefix.contains("content A"));
        assert!(prefix.contains("content B"));
        // Two blocks separated by double newline
        assert!(prefix.contains("</attachment>\n\n<attachment"));
    }

    #[test]
    fn test_resolve_empty_parts_returns_empty_string() {
        let prefix = resolve_context_parts_to_prompt_prefix(&[], &[], &[]).expect("resolve empty");
        assert!(prefix.is_empty());
    }

    // --- PreparedMessageReceipt tests ---

    #[test]
    fn test_prepare_user_message_no_parts_is_ready() {
        let receipt = prepare_user_message_with_receipt("hello", &[], &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
        assert_eq!(
            receipt.schema_version,
            AI_MESSAGE_PREPARATION_SCHEMA_VERSION
        );
        assert_eq!(receipt.raw_content, "hello");
        assert_eq!(receipt.final_user_content, "hello");
        assert!(receipt.outcomes.is_empty());
        assert!(receipt.unresolved_parts.is_empty());
        assert!(receipt.user_error.is_none());
        assert!(receipt.can_send_message());
    }

    #[test]
    fn test_prepare_user_message_blocks_when_all_parts_fail() {
        let parts = vec![AiContextPart::FilePath {
            path: "/definitely/missing/file.txt".to_string(),
            label: "missing.txt".to_string(),
        }];

        let receipt = prepare_user_message_with_receipt("hello", &parts, &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Blocked);
        assert_eq!(receipt.context.attempted, 1);
        assert_eq!(receipt.context.resolved, 0);
        assert_eq!(receipt.unresolved_parts, parts);
        assert!(receipt.user_error.is_some());
        assert_eq!(receipt.outcomes.len(), 1);
        assert_eq!(
            receipt.outcomes[0].kind,
            ContextPartPreparationOutcomeKind::Failed
        );
        assert!(!receipt.can_send_message());
    }

    #[test]
    fn test_prepare_user_message_marks_unreadable_file_as_metadata_only() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("binary.dat");
        std::fs::write(&file_path, vec![0u8; 64]).expect("write temp file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000))
                .expect("set permissions");
        }

        let parts = vec![AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "binary.dat".to_string(),
        }];

        let receipt = prepare_user_message_with_receipt("", &parts, &[], &[]);

        #[cfg(unix)]
        {
            assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
            assert_eq!(receipt.context.resolved, 1);
            assert!(receipt.context.failures.is_empty());
            assert_eq!(receipt.outcomes.len(), 1);
            assert_eq!(
                receipt.outcomes[0].kind,
                ContextPartPreparationOutcomeKind::MetadataOnly
            );
            assert!(receipt.final_user_content.contains("unreadable=\"true\""));
            assert!(receipt.can_send_message());
        }

        #[cfg(not(unix))]
        {
            assert_eq!(receipt.context.resolved, 1);
        }

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));
        }
    }

    #[test]
    fn test_prepare_user_message_appends_prompt_prefix_before_raw_content() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("note.txt");
        std::fs::write(&file_path, "attached text").expect("write temp file");

        let parts = vec![AiContextPart::FilePath {
            path: file_path.to_string_lossy().to_string(),
            label: "note.txt".to_string(),
        }];

        let receipt = prepare_user_message_with_receipt("user text", &parts, &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
        assert!(receipt.final_user_content.contains("attached text"));
        assert!(receipt.final_user_content.ends_with("user text"));
        assert_eq!(receipt.outcomes.len(), 1);
        assert_eq!(
            receipt.outcomes[0].kind,
            ContextPartPreparationOutcomeKind::FullContent
        );
    }

    #[test]
    fn test_prepare_user_message_partial_when_mixed_success_failure() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let good_file = dir.path().join("good.txt");
        std::fs::write(&good_file, "good content").expect("write temp file");

        let parts = vec![
            AiContextPart::FilePath {
                path: good_file.to_string_lossy().to_string(),
                label: "good.txt".to_string(),
            },
            AiContextPart::FilePath {
                path: "/definitely/missing/bad.txt".to_string(),
                label: "bad.txt".to_string(),
            },
        ];

        let receipt = prepare_user_message_with_receipt("query", &parts, &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Partial);
        assert_eq!(receipt.context.attempted, 2);
        assert_eq!(receipt.context.resolved, 1);
        assert_eq!(receipt.context.failures.len(), 1);
        assert_eq!(receipt.unresolved_parts.len(), 1);
        assert!(receipt.final_user_content.contains("good content"));
        assert!(receipt.final_user_content.ends_with("query"));
        assert!(receipt.user_error.is_some());
        assert!(receipt.can_send_message());
    }

    #[test]
    fn merge_context_parts_deduplicates_and_preserves_order() {
        let selection = AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
                    .to_string(),
            label: "Selection".to_string(),
        };
        let browser = AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                    .to_string(),
            label: "Browser URL".to_string(),
        };

        let merged = merge_context_parts(
            &[selection.clone(), browser.clone()],
            std::slice::from_ref(&selection),
        );

        assert_eq!(merged, vec![selection, browser]);
    }

    #[test]
    fn merge_context_parts_empty_inputs() {
        let merged = merge_context_parts(&[], &[]);
        assert!(merged.is_empty());
    }

    #[test]
    fn merge_context_parts_preserves_left_then_right_order() {
        let a = AiContextPart::FilePath {
            path: "/a.rs".to_string(),
            label: "a.rs".to_string(),
        };
        let b = AiContextPart::FilePath {
            path: "/b.rs".to_string(),
            label: "b.rs".to_string(),
        };
        let c = AiContextPart::FilePath {
            path: "/c.rs".to_string(),
            label: "c.rs".to_string(),
        };

        let merged = merge_context_parts(&[a.clone(), b.clone()], &[c.clone(), a.clone()]);
        assert_eq!(merged, vec![a, b, c]);
    }

    #[test]
    fn test_prepare_user_message_receipt_serde_roundtrip() {
        let receipt = PreparedMessageReceipt {
            schema_version: AI_MESSAGE_PREPARATION_SCHEMA_VERSION,
            decision: PreparedMessageDecision::Ready,
            raw_content: "hello".to_string(),
            final_user_content: "prefix\n\nhello".to_string(),
            context: ContextResolutionReceipt {
                attempted: 1,
                resolved: 1,
                failures: vec![],
                prompt_prefix: "prefix".to_string(),
            },
            assembly: None,
            outcomes: vec![ContextPartPreparationOutcome {
                label: "note.txt".to_string(),
                source: "/tmp/note.txt".to_string(),
                kind: ContextPartPreparationOutcomeKind::FullContent,
                detail: None,
            }],
            unresolved_parts: vec![],
            user_error: None,
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let deserialized: PreparedMessageReceipt =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(receipt, deserialized);

        // Verify camelCase serde
        assert!(json.contains("\"schemaVersion\""));
        assert!(json.contains("\"rawContent\""));
        assert!(json.contains("\"finalUserContent\""));
        assert!(json.contains("\"fullContent\""));
    }

    #[test]
    fn merge_context_parts_with_receipt_reports_duplicate_provenance() {
        let selection = AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
                    .to_string(),
            label: "Selection".to_string(),
        };
        let browser = AiContextPart::ResourceUri {
            uri:
                "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                    .to_string(),
            label: "Browser URL".to_string(),
        };

        let receipt = merge_context_parts_with_receipt(
            &[selection.clone(), browser.clone()],
            std::slice::from_ref(&selection),
        );

        assert_eq!(receipt.merged_parts, vec![selection.clone(), browser]);
        assert_eq!(receipt.duplicates_removed, 1);
        assert_eq!(receipt.duplicates.len(), 1);
        assert_eq!(
            receipt.duplicates[0].kept_from,
            ContextAssemblyOrigin::Mention
        );
        assert_eq!(
            receipt.duplicates[0].dropped_from,
            ContextAssemblyOrigin::Pending
        );
        assert_eq!(receipt.duplicates[0].label, "Selection");
    }

    #[test]
    fn prepare_user_message_from_sources_with_receipt_attaches_assembly_receipt() {
        crate::context_snapshot::enable_deterministic_context_capture();
        let prepared = prepare_user_message_from_sources_with_receipt(
            "ship it",
            &[AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Current Context".to_string(),
            }],
            &[AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Current Context".to_string(),
            }],
            &[],
            &[],
        );

        assert!(prepared.can_send_message());
        let assembly = prepared.assembly.expect("assembly receipt must be present");
        assert_eq!(assembly.mention_count, 1);
        assert_eq!(assembly.pending_count, 1);
        assert_eq!(assembly.merged_count, 1);
        assert_eq!(assembly.duplicates_removed, 1);
    }

    #[test]
    fn current_context_picker_part_is_not_treated_as_ambient_bootstrap() {
        let part = AiContextPart::ResourceUri {
            uri: ASK_ANYTHING_RESOURCE_URI.to_string(),
            label: "Current Context".to_string(),
        };

        assert!(
            !part.is_ambient_bootstrap_resource(),
            "@context should resolve directly on submit instead of waiting on deferred capture"
        );
    }

    #[test]
    fn ask_anything_and_explicit_capture_labels_still_use_ambient_bootstrap() {
        for label in [
            ASK_ANYTHING_LABEL,
            "Full Screen",
            "Focused Window",
            "Selected Text",
            "Browser Tab",
        ] {
            let part = AiContextPart::ResourceUri {
                uri: ASK_ANYTHING_RESOURCE_URI.to_string(),
                label: label.to_string(),
            };
            assert!(
                part.is_ambient_bootstrap_resource(),
                "{label} should keep using deferred ambient capture"
            );
        }
    }

    #[test]
    fn test_serde_roundtrip_focused_target() {
        let part = AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "FileSearch".to_string(),
                kind: "file".to_string(),
                semantic_id: "choice:0:main.rs".to_string(),
                label: "main.rs".to_string(),
                metadata: Some(serde_json::json!({ "path": "/tmp/main.rs" })),
            },
            label: "File: main.rs".to_string(),
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let deserialized: AiContextPart = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(part, deserialized);
        assert!(json.contains("\"kind\":\"focusedTarget\""));
        assert!(json.contains("\"semanticId\""));
    }

    #[test]
    fn test_focused_target_label_and_source() {
        let part = AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "ScriptList".to_string(),
                kind: "script".to_string(),
                semantic_id: "choice:2:my-script".to_string(),
                label: "My Script".to_string(),
                metadata: None,
            },
            label: "Command: My Script".to_string(),
        };
        assert_eq!(part.label(), "Command: My Script");
        assert_eq!(part.source(), "choice:2:my-script");
    }

    #[test]
    fn test_resolve_focused_target_produces_context_block() {
        let part = AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "FileSearch".to_string(),
                kind: "file".to_string(),
                semantic_id: "choice:0:tab_ai_mode.rs".to_string(),
                label: "tab_ai_mode.rs".to_string(),
                metadata: Some(serde_json::json!({ "path": "/tmp/tab_ai_mode.rs" })),
            },
            label: "File: tab_ai_mode.rs".to_string(),
        };

        let block =
            resolve_context_part_to_prompt_block(&part, &[], &[]).expect("resolve should succeed");

        assert!(block.contains("source=\"focusedTarget\""));
        assert!(block.contains("itemSource=\"FileSearch\""));
        assert!(block.contains("itemKind=\"file\""));
        assert!(block.contains("semanticId=\"choice:0:tab_ai_mode.rs\""));
        assert!(block.contains("Label: File: tab_ai_mode.rs"));
        assert!(block.contains("/tmp/tab_ai_mode.rs"));
    }

    #[test]
    fn test_resolve_focused_target_no_metadata() {
        let part = AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "ScriptList".to_string(),
                kind: "script".to_string(),
                semantic_id: "choice:0:hello".to_string(),
                label: "hello".to_string(),
                metadata: None,
            },
            label: "Command: hello".to_string(),
        };

        let block =
            resolve_context_part_to_prompt_block(&part, &[], &[]).expect("resolve should succeed");

        assert!(block.contains("source=\"focusedTarget\""));
        assert!(block.contains("{}"), "empty metadata should be '{{}}'");
    }

    #[test]
    fn test_prepare_user_message_with_focused_target() {
        let part = AiContextPart::FocusedTarget {
            target: crate::ai::tab_context::TabAiTargetContext {
                source: "ClipboardHistory".to_string(),
                kind: "clipboard_entry".to_string(),
                semantic_id: "choice:0:clip".to_string(),
                label: "clip".to_string(),
                metadata: Some(serde_json::json!({ "contentType": "text/plain" })),
            },
            label: "Clipboard: clip".to_string(),
        };

        let receipt = prepare_user_message_with_receipt("explain this", &[part], &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
        assert_eq!(receipt.context.attempted, 1);
        assert_eq!(receipt.context.resolved, 1);
        assert!(receipt.final_user_content.contains("focusedTarget"));
        assert!(receipt.final_user_content.ends_with("explain this"));
        assert_eq!(
            receipt.outcomes[0].kind,
            ContextPartPreparationOutcomeKind::FullContent
        );
    }

    #[test]
    fn test_prepare_user_message_with_ambient_context_is_display_only() {
        let part = AiContextPart::AmbientContext {
            label: ASK_ANYTHING_LABEL.to_string(),
        };

        let receipt = prepare_user_message_with_receipt("answer this", &[part], &[], &[]);

        assert_eq!(receipt.decision, PreparedMessageDecision::Ready);
        assert_eq!(receipt.context.attempted, 1);
        assert_eq!(receipt.context.resolved, 0);
        assert!(receipt.context.failures.is_empty());
        assert!(receipt.unresolved_parts.is_empty());
        assert_eq!(receipt.final_user_content, "answer this");
        assert_eq!(
            receipt.outcomes[0].kind,
            ContextPartPreparationOutcomeKind::DisplayOnly
        );
    }

    #[test]
    fn context_assembly_receipt_serde_roundtrip() {
        let receipt = ContextAssemblyReceipt {
            mention_count: 2,
            pending_count: 1,
            merged_count: 2,
            duplicates_removed: 1,
            duplicates: vec![ContextAssemblyDuplicate {
                kept_from: ContextAssemblyOrigin::Mention,
                dropped_from: ContextAssemblyOrigin::Pending,
                label: "Selection".to_string(),
                source: "kit://context?selectedText=1".to_string(),
            }],
            merged_parts: vec![AiContextPart::ResourceUri {
                uri: "kit://context?profile=minimal".to_string(),
                label: "Context".to_string(),
            }],
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let deserialized: ContextAssemblyReceipt =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(receipt, deserialized);
        assert!(json.contains("\"mentionCount\""));
        assert!(json.contains("\"pendingCount\""));
        assert!(json.contains("\"keptFrom\""));
        assert!(json.contains("\"droppedFrom\""));
    }
}
