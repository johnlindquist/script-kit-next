//! Source audit for the Agent Chat / ACP rename boundary.
//!
//! The project renamed the *feature* to "Agent Chat" while keeping "ACP" as the
//! name of frozen compatibility contracts (action IDs, route IDs, serialized
//! surface IDs, `getAcpState`, telemetry labels). This audit proves two things
//! at once:
//!
//! 1. The canonical `agent_chat::ui` boundary exists and is wired into the
//!    launcher view state, so new code has a stable `AgentChat*` import surface.
//! 2. The frozen external contracts are still present verbatim. If a future
//!    rename pass deletes or edits one of these strings, this audit fails and
//!    forces the change to be deliberate (and paired with a contract migration).

use std::fs;
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn agent_chat_ui_boundary_exists() {
    let mod_rs = read("src/ai/agent_chat/mod.rs");
    assert!(
        mod_rs.contains("pub(crate) mod ui;"),
        "agent_chat must expose the canonical `ui` boundary module"
    );
    assert!(
        mod_rs.contains("pub(crate) mod content;"),
        "agent_chat must expose the `content` boundary module"
    );

    let ui = read("src/ai/agent_chat/ui/mod.rs");
    for alias in [
        "AcpChatView as AgentChatView",
        "AcpThread as AgentChatThread",
        "AcpEvent as AgentChatEvent",
        "AcpChatSession as AgentChatSession",
        "AcpInlineSetupState as AgentChatInlineSetupState",
        "AcpRetryRequest as AgentChatRetryRequest",
        "AcpPermissionBroker as AgentChatPermissionBroker",
    ] {
        assert!(
            ui.contains(alias),
            "agent_chat::ui must re-export `{alias}` as part of the canonical boundary"
        );
    }
}

#[test]
fn app_view_state_uses_agent_chat_ui_boundary() {
    let app_view = read("src/main_sections/app_view_state.rs");
    assert!(
        app_view.contains("crate::ai::agent_chat::ui::AgentChatView"),
        "AcpChatView variant entity must flow through the agent_chat::ui boundary"
    );
}

#[test]
fn frozen_serialized_surface_ids_are_unchanged() {
    let app_view = read("src/main_sections/app_view_state.rs");
    // Serialized view-type ids feed launcher surface contracts and automation;
    // these MUST stay stable even though the feature is now "Agent Chat".
    assert!(
        app_view.contains("Some(\"acp_chat\")"),
        "frozen serialized surface id `acp_chat` must remain"
    );
    assert!(
        app_view.contains("SurfaceKind::AcpChat"),
        "frozen SurfaceKind::AcpChat variant must remain"
    );
    assert!(
        app_view.contains("AppView::AcpChatView"),
        "frozen AppView::AcpChatView variant must remain"
    );
}

#[test]
fn frozen_action_and_route_ids_are_unchanged() {
    let script_context = read("src/actions/builders/script_context.rs");
    for id in ["acp:root", "acp:change_model", "acp_switch_model:"] {
        assert!(
            script_context.contains(id),
            "frozen action/route id `{id}` must remain in script_context.rs"
        );
    }
}

#[test]
fn frozen_get_acp_state_protocol_contract_is_unchanged() {
    let acp_state = read("src/protocol/types/acp_state.rs");
    assert!(
        acp_state.contains("ACP_STATE_SCHEMA_VERSION"),
        "frozen `ACP_STATE_SCHEMA_VERSION` automation contract must remain"
    );
    assert!(
        acp_state.contains("AcpStateSnapshot"),
        "frozen `AcpStateSnapshot` automation type must remain"
    );
}
