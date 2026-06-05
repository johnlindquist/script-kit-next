//! Source-level contract tests for the ACP portal intent and host handoff flow.

const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const PORTAL_CONTRACT_SOURCE: &str = include_str!("../src/ai/acp/portal_contract.rs");
const ATTACHMENT_PORTAL_SOURCE: &str = include_str!("../src/app_impl/attachment_portal.rs");
const CHAT_WINDOW_SOURCE: &str = include_str!("../src/ai/acp/chat_window.rs");
const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");
const PASTED_TEXT_SOURCE: &str = include_str!("../src/pasted_text.rs");
const PASTED_IMAGE_SOURCE: &str = include_str!("../src/pasted_image.rs");

#[test]
fn pending_portal_session_uses_shared_contract_state() {
    assert!(
        ACP_VIEW_SOURCE.contains("struct AcpPendingPortalSession"),
        "ACP view must keep a pending portal session type"
    );
    assert!(
        ACP_VIEW_SOURCE
            .contains("contract: crate::ai::acp::portal_contract::AcpPortalLaunchContract"),
        "pending portal session must store the shared launch contract"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("composer_text: String"),
        "pending portal session must preserve composer text across portal entry"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("composer_cursor: usize"),
        "pending portal session must preserve composer cursor across portal entry"
    );
}

#[test]
fn focused_preview_and_accept_paths_use_the_shared_contract() {
    assert!(
        ACP_VIEW_SOURCE.contains("fn focused_inline_portal_intent("),
        "ACP view must derive a single focused-inline portal intent"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("format_intent_preview(&intent)"),
        "focused mention preview must render from the shared portal intent"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("crate::ai::acp::portal_contract::apply_portal_replacement("),
        "portal accept must apply replacements through the shared contract helper"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("event = \"acp_portal_reentry_applied\""),
        "portal accept path must log the contract-driven reentry result"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("pub(crate) enum AcpPortalIntent"),
        "portal contract module must define AcpPortalIntent"
    );
}

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
        PORTAL_CONTRACT_SOURCE.contains("acp_inline_token_forced_preview_only"),
        "Portal contract must log forced preview-only token classification"
    );
    assert!(
        PORTAL_CONTRACT_SOURCE.contains("acp_part_forced_preview_only"),
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

#[test]
fn host_transitions_preserve_the_staged_portal_session() {
    let prepare_for_host_hide = ACP_VIEW_SOURCE
        .split("pub(crate) fn prepare_for_host_hide")
        .nth(1)
        .and_then(|rest| {
            rest.split("pub(crate) fn prepare_for_attachment_portal_open")
                .next()
        })
        .expect("prepare_for_host_hide source should exist");

    assert!(
        !prepare_for_host_hide.contains("self.pending_portal_session = None;"),
        "prepare_for_host_hide must preserve the staged portal session across host transitions"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("let portal_view = view.downgrade();"),
        "Notes portal callback must capture the originating ACP view"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("Self::handle_acp_portal_static(Some(chat), kind, cx);"),
        "Notes portal callback must reopen history against the originating ACP view"
    );
}

#[test]
fn history_portal_hosts_seed_query_from_the_pending_contract() {
    assert!(
        ACP_VIEW_SOURCE.contains("&session.contract.query"),
        "ACP portal query accessors must read the staged contract query"
    );
    assert!(
        ATTACHMENT_PORTAL_SOURCE.contains("attachment_portal_query_seeded_from_contract"),
        "main-window attachment portal must log query seeding from the contract"
    );
    assert!(
        CHAT_WINDOW_SOURCE.contains("detached_acp_history_portal_query_seeded_from_contract"),
        "detached ACP must log history query seeding from the contract"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("notes_acp_history_portal_query_seeded_from_contract"),
        "Notes-hosted ACP must log history query seeding from the contract"
    );
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("PortalKind::AcpHistory"),
        "Notes host must remain restricted to history portals in this iteration"
    );
}

#[test]
fn attachment_portal_log_keeps_shared_contract_fields() {
    assert!(
        ATTACHMENT_PORTAL_SOURCE.contains("attachment_portal_query_seeded_from_contract"),
        "Attachment portal must keep the shared contract query-seeding log"
    );
    assert!(
        ATTACHMENT_PORTAL_SOURCE.contains("portal_query_for(kind).unwrap_or_default()"),
        "Attachment portal must still read the query from the staged ACP contract"
    );
}
