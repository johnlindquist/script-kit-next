//! Source audit for the Agent Chat / Agent Chat rename boundary.
//!
//! The project renamed the *feature* to "Agent Chat" while keeping "Agent Chat" as the
//! name of frozen compatibility contracts (action IDs, route IDs, serialized
//! surface IDs, `getAgentChatState`, telemetry labels). This audit proves two things
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
    let reexports = reexported_names(&ui);
    for name in [
        "AgentChatView",
        "AgentChatThread",
        "AgentChatEvent",
        "AgentChatSession",
        "AgentChatInlineSetupState",
        "AgentChatRetryRequest",
        "AgentChatPermissionBroker",
    ] {
        assert!(
            reexports.iter().any(|item| item == name),
            "agent_chat::ui must re-export `{name}` as part of the canonical boundary"
        );
    }
}

/// Names re-exported by a module's `pub(crate) use ...;` statements.
///
/// This used to be asserted as the literal text `"AgentChatView as AgentChatView"`.
/// That spelling is a redundant self-alias, so a routine cleanup deleted the
/// `as` halves and every one of these assertions went red while the boundary
/// itself was untouched — all seven types are still re-exported. The invariant
/// worth keeping is "the facade re-exports this type", not "the facade spells
/// it in the one style it happened to use in 2026".
///
/// Parsing the use statements also makes the check STRICTER than the old
/// substring test: a type merely mentioned in a doc comment or a function body
/// no longer counts, because only text inside `pub(crate) use ... ;` is
/// considered.
fn reexported_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("pub(crate) use ") {
        let after = &rest[start + "pub(crate) use ".len()..];
        let Some(end) = after.find(';') else { break };
        let statement = &after[..end];
        // Keep only the leaf identifiers: drop path prefixes and braces, and
        // resolve `foo as bar` to the exported name `bar`.
        for piece in statement.split([',', '{', '}']) {
            let piece = piece.trim();
            if piece.is_empty() {
                continue;
            }
            let exported = match piece.rsplit_once(" as ") {
                Some((_, alias)) => alias.trim(),
                None => piece.rsplit("::").next().unwrap_or(piece).trim(),
            };
            if !exported.is_empty() && exported.chars().all(|c| c.is_alphanumeric() || c == '_') {
                names.push(exported.to_string());
            }
        }
        rest = &after[end..];
    }
    names
}

#[test]
fn app_view_state_uses_agent_chat_ui_boundary() {
    let app_view = read("src/main_sections/app_view_state.rs");
    assert!(
        app_view.contains("crate::ai::agent_chat::ui::AgentChatView"),
        "AgentChatView variant entity must flow through the agent_chat::ui boundary"
    );
}

#[test]
fn frozen_serialized_surface_ids_are_unchanged() {
    let app_view = read("src/main_sections/app_view_state.rs");
    // Serialized view-type ids feed launcher surface contracts and automation;
    // these MUST stay stable even though the feature is now "Agent Chat".
    assert!(
        app_view.contains("Some(\"agent_chat\")"),
        "frozen serialized surface id `agent_chat` must remain"
    );
    assert!(
        app_view.contains("SurfaceKind::AgentChat"),
        "frozen SurfaceKind::AgentChat variant must remain"
    );
    assert!(
        app_view.contains("AppView::AgentChatView"),
        "frozen AppView::AgentChatView variant must remain"
    );
}

#[test]
fn frozen_action_and_route_ids_are_unchanged() {
    let script_context = read("src/actions/builders/script_context.rs");
    for id in [
        "agent_chat:root",
        "agent_chat:change_model",
        "agent_chat_switch_model:",
    ] {
        assert!(
            script_context.contains(id),
            "frozen action/route id `{id}` must remain in script_context.rs"
        );
    }
}

#[test]
fn frozen_get_agent_chat_state_protocol_contract_is_unchanged() {
    let agent_chat_state = read("src/protocol/types/agent_chat_state.rs");
    assert!(
        agent_chat_state.contains("AGENT_CHAT_STATE_SCHEMA_VERSION"),
        "frozen `AGENT_CHAT_STATE_SCHEMA_VERSION` automation contract must remain"
    );
    assert!(
        agent_chat_state.contains("AgentChatStateSnapshot"),
        "frozen `AgentChatStateSnapshot` automation type must remain"
    );
}

#[test]
fn agent_chat_contract_strings_remain_frozen_until_contract_rename_slice() {
    // Step 6 changes only display copy. Native window title, serialized surface
    // ids, automation snapshot, and detached-target wire ids stay frozen.
    assert!(
        read("src/platform/secondary_window_config.rs").contains("Script Kit Agent Chat"),
        "frozen native window title `Script Kit Agent Chat` must remain"
    );
    assert!(
        read("src/ai/agent_chat/ui/chat_window.rs").contains("agentChatDetached"),
        "frozen detached-target wire id `agentChatDetached` must remain"
    );
    let app_view = read("src/main_sections/app_view_state.rs");
    assert!(
        app_view.contains("Some(\"agent_chat_history\")"),
        "frozen serialized surface id `agent_chat_history` must remain"
    );
}
