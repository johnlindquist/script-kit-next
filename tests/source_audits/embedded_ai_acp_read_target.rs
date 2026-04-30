//! Source audit tests verifying that `getAcpState {target:{kind:"ai"}}`
//! is routed to the main window's ACP collector rather than rejected as
//! an unsupported target.
//!
//! Background: Pass #7 of Run 4 added an `AutomationWindowKind::Ai` entry
//! to the automation registry whenever the embedded ACP chat is the active
//! subview of main. Before this pass, `resolve_acp_read_target` accepted
//! only `Main` and `AcpDetached`, so a `{kind:"ai"}` target would fall
//! through to the catch-all arm and return a `target_unsupported` warning
//! even though the embedded surface is a real addressable subview of main
//! (see `audits/afk/stories.md` → `tool-get-acp-state-target-selector`).
//!
//! This pass adds an explicit `AutomationWindowKind::Ai` arm that routes
//! to `AcpReadTarget::Main { info: Some(resolved) }` — the embedded AI
//! IS a subview of main, so reading its state IS reading main's ACP state.
//!
//! These tests pin:
//! 1. The `Ai` match arm exists in `resolve_acp_read_target`.
//! 2. It routes to `AcpReadTarget::Main` (not `Detached`, not `Err`).
//! 3. It emits the `embedded_ai_routed_to_main` trace line so ops can tell
//!    from the log that a `{kind:"ai"}` request was served from main.

use super::read_source as read;

const HANDLER_PATH: &str = "src/prompt_handler/mod.rs";

fn resolve_fn_body<'a>(content: &'a str) -> &'a str {
    let needle = "fn resolve_acp_read_target(";
    let start = content
        .find(needle)
        .unwrap_or_else(|| panic!("Expected `{needle}` in {HANDLER_PATH}"));
    let rest = &content[start..];
    let end = rest
        .find("\nfn build_acp_resolved_target")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn resolve_acp_read_target_has_ai_arm() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);
    assert!(
        body.contains("crate::protocol::AutomationWindowKind::Ai =>"),
        "Expected `AutomationWindowKind::Ai =>` arm in resolve_acp_read_target; \
         without it, `getAcpState {{target:{{kind:\"ai\"}}}}` falls through to \
         the catch-all and is rejected as target_unsupported.",
    );
}

#[test]
fn ai_arm_routes_to_main_read_target() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);

    let ai_arm_start = body
        .find("crate::protocol::AutomationWindowKind::Ai =>")
        .expect("Ai arm must exist (see resolve_acp_read_target_has_ai_arm)");
    let after_ai = &body[ai_arm_start..];
    let other_start = after_ai.find("other_kind =>").unwrap_or(after_ai.len());
    let ai_arm_body = &after_ai[..other_start];

    assert!(
        ai_arm_body.contains("Ok(AcpReadTarget::Main {"),
        "The Ai arm must route to AcpReadTarget::Main (the embedded AI is a \
         subview of main, so its ACP state IS main's ACP state). Found arm body:\n{ai_arm_body}",
    );
    assert!(
        ai_arm_body.contains("info: Some(resolved)"),
        "The Ai arm must pass `info: Some(resolved)` so telemetry carries the \
         ai window id — the Main arm uses the same shape.",
    );
    assert!(
        !ai_arm_body.contains("AcpReadTarget::Detached"),
        "The Ai arm MUST NOT route to Detached — embedded AI has no separate \
         entity; reusing Detached would call `get_detached_acp_view_entity()` \
         and hit the no-entity branch in normal operation.",
    );
}

#[test]
fn ai_arm_emits_embedded_ai_routed_trace() {
    let content = read(HANDLER_PATH);
    let body = resolve_fn_body(&content);
    assert!(
        body.contains("automation.acp_target.embedded_ai_routed_to_main"),
        "The Ai arm must emit a distinct trace line \
         `automation.acp_target.embedded_ai_routed_to_main` so ops can see in \
         the log that a `{{kind:\"ai\"}}` request was served via main. The \
         generic Main trace would mis-attribute the target.",
    );
}
