//! ACP context block builders.
//!
//! Wraps the existing Tab AI context formatter output as ACP `ContentBlock`
//! values suitable for prepending to the first ACP `session/prompt` turn.
//! Keeps the first ACP slice text-only for capability safety — no image or
//! resource blocks are emitted here.

use agent_client_protocol::{ContentBlock, ImageContent, TextContent};

use crate::ai::harness::{
    build_tab_ai_artifact_authoring_appendix_for_prompt, TabAiArtifactKind,
    TabAiHarnessSubmissionMode,
};
use crate::ai::{build_tab_ai_harness_context_block, TabAiContextBlob};

/// Convert an existing `TabAiContextBlob` into a `Vec<ContentBlock>` with a
/// single text block containing the canonical `Script Kit context` header.
///
/// This reuses `build_tab_ai_harness_context_block` so the ACP path and the
/// PTY path emit identical context content.
pub(crate) fn build_tab_ai_acp_context_blocks(
    context: &TabAiContextBlob,
) -> Result<Vec<ContentBlock>, String> {
    let context_text = build_tab_ai_harness_context_block(context)?;
    let mut blocks = vec![ContentBlock::Text(TextContent::new(context_text))];

    // If a screenshot was captured, embed it as an Image block so the agent
    // can see what's on the user's screen without an extra tool call.
    if let Some(path) = context.screenshot_path.as_deref() {
        match std::fs::read(path) {
            Ok(bytes) => {
                use base64::Engine as _;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                blocks.push(ContentBlock::Image(ImageContent::new(b64, "image/png")));
                tracing::debug!(
                    path,
                    size_bytes = bytes.len(),
                    "acp_context_screenshot_embedded"
                );
            }
            Err(error) => {
                tracing::debug!(path, %error, "acp_context_screenshot_read_failed");
            }
        }
    }

    Ok(blocks)
}

/// Return guidance blocks for a prompt when the intent looks like an authoring
/// request.  Delegates to the shared appendix builder so ACP and PTY paths
/// emit identical verification contracts.
pub(crate) fn build_tab_ai_acp_guidance_blocks_for_prompt(
    prompt_type: &str,
    intent: Option<&str>,
) -> Vec<ContentBlock> {
    let Some(intent) = intent.map(str::trim).filter(|value| !value.is_empty()) else {
        tracing::debug!(
            event = "tab_ai_acp_guidance_blocks_skipped",
            prompt_type,
            reason = "empty_intent",
        );
        return Vec::new();
    };

    let Some(appendix) = build_tab_ai_artifact_authoring_appendix_for_prompt(
        prompt_type,
        Some(intent),
        TabAiHarnessSubmissionMode::Submit,
    ) else {
        tracing::debug!(
            event = "tab_ai_acp_guidance_blocks_skipped",
            prompt_type,
            reason = "not_authoring_intent",
        );
        return Vec::new();
    };

    tracing::info!(
        event = "tab_ai_acp_guidance_blocks_built",
        prompt_type,
        forced_by_script_list_submit = appendix.forced_by_script_list_submit,
        artifact_kind = appendix
            .artifact_kind
            .map(TabAiArtifactKind::as_str)
            .unwrap_or("unknown"),
        use_quick_terminal = appendix.use_quick_terminal,
        script_verification_gate_present = appendix.has_script_verification_gate_header,
        includes_script_authoring_skill = appendix.markers.includes_script_authoring_skill,
        includes_bun_build_verification = appendix.markers.includes_bun_build_verification,
        includes_bun_execute_verification = appendix.markers.includes_bun_execute_verification,
        text_len = appendix.guidance.len(),
    );

    vec![ContentBlock::Text(TextContent::new(
        appendix.guidance.trim_end(),
    ))]
}

/// Legacy wrapper — do not add new callers.
///
/// This path cannot force the `ScriptList` verification contract because it has
/// no `prompt_type`.  Migrate all callers to
/// [`build_tab_ai_acp_guidance_blocks_for_prompt`].
#[deprecated(note = "Use build_tab_ai_acp_guidance_blocks_for_prompt(prompt_type, intent)")]
pub(crate) fn build_tab_ai_acp_guidance_blocks(intent: Option<&str>) -> Vec<ContentBlock> {
    let has_non_empty_intent = intent
        .map(str::trim)
        .map(|value| !value.is_empty())
        .unwrap_or(false);

    if has_non_empty_intent {
        tracing::error!(
            event = "tab_ai_acp_guidance_blocks_legacy_call",
            reason = "missing_prompt_type",
            "Legacy ACP guidance wrapper was invoked with a non-empty intent. \
             This path can miss the deterministic ScriptList verification contract."
        );
    }

    build_tab_ai_acp_guidance_blocks_for_prompt("Unknown", intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_text_block_from_existing_tab_context_formatter() {
        let context = TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                input_text: Some("rename this".to_string()),
                focused_semantic_id: Some("choice:0:rename".to_string()),
                selected_semantic_id: Some("choice:0:rename".to_string()),
                visible_elements: Vec::new(),
            },
            crate::context_snapshot::AiContextSnapshot {
                schema_version: crate::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
                selected_text: None,
                frontmost_app: None,
                menu_bar_items: Vec::new(),
                browser: None,
                focused_window: None,
                focused_window_image: None,
                script_kit_panel_image: None,
                warnings: Vec::new(),
            },
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            "2026-04-01T00:00:00Z".to_string(),
        );

        let blocks = build_tab_ai_acp_context_blocks(&context).expect("context block");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            ContentBlock::Text(text) => {
                assert!(
                    text.text.contains("Script Kit context"),
                    "expected 'Script Kit context' header, got: {}",
                    &text.text[..text.text.len().min(200)]
                );
                assert!(
                    text.text.contains("prompt type: ScriptList"),
                    "expected prompt type in context block"
                );
            }
            other => panic!("expected text block, got {other:?}"),
        }
    }

    #[test]
    fn guidance_blocks_are_added_only_for_authoring_intents() {
        // Authoring intent → should include guidance
        let authoring = build_tab_ai_acp_guidance_blocks_for_prompt(
            "ScriptList",
            Some("build a clipboard cleanup script"),
        );
        assert_eq!(
            authoring.len(),
            1,
            "authoring intent should produce one guidance block"
        );
        // Must contain the shared verification contract, not the old static embed
        match &authoring[0] {
            ContentBlock::Text(text) => {
                assert!(
                    text.text.contains("MANDATORY SCRIPT VERIFICATION"),
                    "guidance must include shared verification gate header"
                );
                assert!(
                    text.text
                        .contains("~/.scriptkit/kit/authoring/skills/script-authoring/SKILL.md"),
                    "guidance must reference the script-authoring skill"
                );
            }
            other => panic!("expected text block, got {other:?}"),
        }

        // Non-authoring intent → no guidance
        let non_authoring = build_tab_ai_acp_guidance_blocks_for_prompt(
            "FileSearch",
            Some("explain this selection"),
        );
        assert!(
            non_authoring.is_empty(),
            "non-authoring intent should produce no guidance blocks"
        );
    }

    #[test]
    fn guidance_blocks_empty_for_none_intent() {
        let blocks = build_tab_ai_acp_guidance_blocks_for_prompt("ScriptList", None);
        assert!(
            blocks.is_empty(),
            "None intent should produce no guidance blocks"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn legacy_wrapper_still_works_for_authoring_intent() {
        let blocks = build_tab_ai_acp_guidance_blocks(Some("build a clipboard cleanup script"));
        assert_eq!(
            blocks.len(),
            1,
            "legacy wrapper should still produce guidance for authoring intents"
        );
    }

    #[test]
    fn context_block_is_text_only() {
        let context = TabAiContextBlob::from_parts(
            crate::ai::TabAiUiSnapshot {
                prompt_type: "ArgPrompt".to_string(),
                input_text: None,
                focused_semantic_id: None,
                selected_semantic_id: None,
                visible_elements: Vec::new(),
            },
            crate::context_snapshot::AiContextSnapshot::default(),
            Vec::new(),
            None,
            Vec::new(),
            Vec::new(),
            "2026-04-01T00:00:00Z".to_string(),
        );

        let blocks = build_tab_ai_acp_context_blocks(&context).expect("context block");
        for block in &blocks {
            assert!(
                matches!(block, ContentBlock::Text(_)),
                "all blocks should be text-only, got {block:?}"
            );
        }
    }
}
