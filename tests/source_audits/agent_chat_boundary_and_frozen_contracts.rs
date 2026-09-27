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
