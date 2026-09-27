//! Source-level contract tests for the Agent Chat portal intent and host handoff flow.

const PORTAL_CONTRACT_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/portal_contract.rs");
const PASTED_TEXT_SOURCE: &str = include_str!("../src/pasted_text.rs");
const PASTED_IMAGE_SOURCE: &str = include_str!("../src/pasted_image.rs");

#[test]
fn portal_contract_detects_synthetic_pasted_tokens_before_portal_lookup() {
    assert!(
        PASTED_TEXT_SOURCE.contains("Pasted text #"),
        "Pasted text token format must remain stable for preview-only detection"
    );
    assert!(
        PASTED_IMAGE_SOURCE.contains("@img:paste"),
        "Pasted image token format must remain stable for preview-only detection"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("preview_only_inline_token_description"),
        "Portal contract must classify synthetic pasted tokens before portal lookup"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("agent_chat_inline_token_forced_preview_only"),
        "Portal contract must log forced preview-only token classification"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("agent_chat_part_forced_preview_only"),
        "Portal contract must log forced preview-only part classification"
    );
}

#[test]
fn portal_contract_compacts_replacement_target_copy() {
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("PREVIEW_TARGET_MAX_CHARS"),
        "Portal contract must define a preview target length cap"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("compact_preview_target_text"),
        "Portal contract must collapse/truncate replacement target copy"
    );
}
