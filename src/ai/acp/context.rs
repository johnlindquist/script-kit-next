//! ACP context block builders.
//!
//! Wraps the existing Tab AI context formatter output as ACP `ContentBlock`
//! values suitable for prepending to the first ACP `session/prompt` turn.
//! Keeps the first ACP slice text-only for capability safety — no image or
//! resource blocks are emitted here.

use agent_client_protocol::{ContentBlock, TextContent};

use crate::ai::{
    build_tab_ai_harness_context_block, should_include_artifact_authoring_guidance,
    TabAiContextBlob,
};

/// Compile-time embed of the artifact authoring guidance.
const ACP_ARTIFACT_GUIDANCE: &str = include_str!("../../../kit-init/examples/START_HERE.md");

/// Convert an existing `TabAiContextBlob` into a `Vec<ContentBlock>` with a
/// single text block containing the canonical `Script Kit context` header.
///
/// This reuses `build_tab_ai_harness_context_block` so the ACP path and the
/// PTY path emit identical context content.
pub(crate) fn build_tab_ai_acp_context_blocks(
    context: &TabAiContextBlob,
) -> Result<Vec<ContentBlock>, String> {
    let context_text = build_tab_ai_harness_context_block(context)?;
    Ok(vec![ContentBlock::Text(TextContent::new(context_text))])
}

/// Return guidance blocks only when the intent looks like an authoring request
/// (e.g. "build a clipboard cleanup script").  Non-authoring intents (e.g.
/// "explain this selection") receive no guidance blocks.
pub(crate) fn build_tab_ai_acp_guidance_blocks(intent: Option<&str>) -> Vec<ContentBlock> {
    if !should_include_artifact_authoring_guidance(intent) {
        return Vec::new();
    }
    vec![ContentBlock::Text(TextContent::new(
        ACP_ARTIFACT_GUIDANCE.trim_end(),
    ))]
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
        let authoring = build_tab_ai_acp_guidance_blocks(Some("build a clipboard cleanup script"));
        assert_eq!(
            authoring.len(),
            1,
            "authoring intent should produce one guidance block"
        );

        // Non-authoring intent → no guidance
        let non_authoring = build_tab_ai_acp_guidance_blocks(Some("explain this selection"));
        assert!(
            non_authoring.is_empty(),
            "non-authoring intent should produce no guidance blocks"
        );
    }

    #[test]
    fn guidance_blocks_empty_for_none_intent() {
        let blocks = build_tab_ai_acp_guidance_blocks(None);
        assert!(
            blocks.is_empty(),
            "None intent should produce no guidance blocks"
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
