//! Source-level contract for Notes-hosted ACP draft snapshots during agent switch.

const THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

// @lat: [[tests/notes-acp#Notes ACP draft snapshot across agent switch#Draft snapshot preserves text and context]]
#[test]
fn draft_snapshot_types_exist_on_thread_and_view() {
    assert!(
        THREAD_SOURCE.contains("pub(crate) struct AcpThreadDraftSnapshot"),
        "thread draft snapshot type must exist"
    );
    for needle in [
        "pub input: String",
        "pub pending_context_parts: Vec<crate::ai::message_parts::AiContextPart>",
        "pub pending_context_consumed: bool",
        "pub(crate) fn draft_snapshot(&self) -> AcpThreadDraftSnapshot",
        "pub(crate) fn restore_draft_snapshot(",
        "self.context_bootstrap_state = AcpContextBootstrapState::Ready;",
        "self.pending_context_blocks.clear();",
    ] {
        assert!(
            THREAD_SOURCE.contains(needle),
            "thread snapshot must contain {needle}"
        );
    }
    assert!(
        VIEW_SOURCE.contains("pub(crate) struct AcpViewDraftSnapshot")
            && VIEW_SOURCE.contains("pub(crate) fn capture_draft_snapshot")
            && VIEW_SOURCE.contains("pub(crate) fn restore_draft_snapshot"),
        "view draft snapshot capture/restore helpers must exist"
    );
}

// @lat: [[tests/notes-acp#Notes ACP draft snapshot across agent switch#Agent switch restores snapshot after relaunch]]
#[test]
fn notes_agent_switch_captures_before_drop_and_restores_after_relaunch() {
    assert!(
        !NOTES_ACP_HOST_SOURCE.contains("thread.input.text().trim().to_string()"),
        "Notes agent switch must not trim draft input"
    );
    let body = source_between(
        NOTES_ACP_HOST_SOURCE,
        "if let Some(agent_id) = crate::actions::acp_switch_agent_id_from_action(action_id)",
        "// Handle model switch.",
    );
    let capture = body
        .find("view.capture_draft_snapshot(cx)")
        .expect("agent switch must capture a full draft snapshot");
    let drop = body
        .find("app.embedded_acp_chat = None;")
        .expect("agent switch must drop the old embedded view before relaunch");
    let relaunch = body
        .find("app.open_or_focus_embedded_acp(None, window, cx)")
        .expect("agent switch must relaunch without reducing draft to initial_input");
    let restore = body
        .find("chat.restore_draft_snapshot(snapshot, cx);")
        .expect("agent switch must restore the snapshot into the new view");
    assert!(
        capture < drop && drop < relaunch && relaunch < restore,
        "agent switch must capture -> drop -> relaunch -> restore"
    );
}

// @lat: [[tests/notes-acp#Notes ACP draft snapshot across agent switch#Reused Notes ACP does not overwrite composer]]
#[test]
fn ensure_embedded_acp_view_does_not_set_input_on_reuse() {
    let body = source_between(
        NOTES_ACP_HOST_SOURCE,
        "pub(super) fn ensure_embedded_acp_view(",
        "pub(super) fn open_or_focus_embedded_acp(",
    );
    let reuse_block = body
        .split("if let Some(entity) = self.embedded_acp_chat.as_ref().cloned()")
        .nth(1)
        .and_then(|rest| rest.split("let requirements =").next())
        .expect("reuse branch should exist");
    assert!(
        !reuse_block.contains(".set_input("),
        "reused embedded ACP views must not overwrite the composer"
    );
}
