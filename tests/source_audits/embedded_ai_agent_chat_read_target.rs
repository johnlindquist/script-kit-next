//! Source audit tests verifying that `getAgentChatState {target:{kind:"ai"}}`
//! routes through the live Agent Chat entity when available, and only falls
//! back to the main window collector when no entity is active.
//!
//! Background: Pass #7 of Run 4 added an `AutomationWindowKind::Ai` entry
//! to the automation registry whenever the embedded Agent Chat chat is the active
//! subview of main. Before this pass, `resolve_agent_chat_read_target` accepted
//! only `Main` and `AgentChatDetached`, so a `{kind:"ai"}` target would fall
//! through to the catch-all arm and return a `target_unsupported` warning
//! even though the embedded surface is a real addressable subview of main
//! (see `audits/afk/stories.md` → `tool-get-agent_chat-state-target-selector`).
//!
//! The current model treats the AI automation window as an Agent Chat-capable target:
//! when there is a live Agent Chat entity, reads use the entity-backed target;
//! otherwise the main collector remains the fallback for embedded main state.
//!
//! These tests pin:
//! 1. The `Ai` match arm exists in `resolve_agent_chat_read_target`.
//! 2. It routes to the entity-backed target when present and keeps the main
//!    fallback when absent.
//! 3. It emits distinct trace lines for the entity and fallback paths.

use super::read_source as read;

const HANDLER_PATH: &str = "src/prompt_handler/mod.rs";

fn resolve_fn_body<'a>(content: &'a str) -> &'a str {
    let needle = "fn resolve_agent_chat_read_target(";
    let start = content
        .find(needle)
        .unwrap_or_else(|| panic!("Expected `{needle}` in {HANDLER_PATH}"));
    let rest = &content[start..];
    let end = rest
        .find("\nfn build_agent_chat_resolved_target")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn resolve_agent_chat_read_target_has_ai_arm() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);
    assert!(
        body.contains("crate::protocol::AutomationWindowKind::Ai =>"),
        "Expected `AutomationWindowKind::Ai =>` arm in resolve_agent_chat_read_target; \
         without it, `getAgentChatState {{target:{{kind:\"ai\"}}}}` falls through to \
         the catch-all and is rejected as target_unsupported.",
    );
}

#[test]
fn ai_arm_routes_to_entity_with_main_fallback() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);

    let ai_arm_start = body
        .find("crate::protocol::AutomationWindowKind::Ai =>")
        .expect("Ai arm must exist (see resolve_agent_chat_read_target_has_ai_arm)");
    let after_ai = &body[ai_arm_start..];
    let other_start = after_ai.find("other_kind =>").unwrap_or(after_ai.len());
    let ai_arm_body = &after_ai[..other_start];

    assert!(
        ai_arm_body.contains("Ok(AgentChatReadTarget::Detached {"),
        "The Ai arm must route to the live Agent Chat entity when available. \
         Found arm body:\n{ai_arm_body}",
    );
    assert!(
        ai_arm_body.contains("Ok(AgentChatReadTarget::Main {")
            && ai_arm_body.contains("info: Some(resolved)"),
        "The Ai arm must keep a main collector fallback that preserves resolved \
         target telemetry.",
    );
    assert!(
        ai_arm_body.contains("active_agent_chat_entity(embedded_agent_chat)"),
        "The Ai arm must check for the active Agent Chat entity before falling back.",
    );
}

#[test]
fn ai_arm_emits_distinct_entity_and_fallback_traces() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);
    assert!(
        body.contains("automation.agent_chat_target.ai_resolved_to_entity")
            && body.contains("automation.agent_chat_target.ai_fallback_main_collector"),
        "The Ai arm must emit distinct trace lines for entity-backed reads and \
         main-collector fallback reads.",
    );
}
