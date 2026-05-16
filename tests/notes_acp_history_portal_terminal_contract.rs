//! Source-level contract for Notes ACP history portal terminal cleanup.

const NOTES_ACP_HOST_SOURCE: &str = include_str!("../src/notes/window/acp_host.rs");

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_idx = source.find(start).expect("start marker should exist");
    let rest = &source[start_idx..];
    let end_idx = rest.find(end).unwrap_or(rest.len());
    &rest[..end_idx]
}

// doc-anchor-removed: [[tests/acp-portal-contract#Host transitions#Notes history host refusal clears staged session]]
#[test]
fn notes_history_portal_failure_and_refusal_clear_staged_session() {
    let body = function_body(
        NOTES_ACP_HOST_SOURCE,
        "fn handle_acp_portal_static(",
        "/// Wire ACP host callbacks",
    );
    assert!(body.contains("view.take_pending_history_portal_query().unwrap_or_default()"));
    assert!(body.contains("view.open_history_portal_with_entries(query, hits, cx)"));
    assert!(body.contains("if !opened"));
    assert!(body.contains("view.cancel_pending_portal_session(PortalKind::AcpHistory, cx)"));
    assert!(body.contains("view.cancel_pending_portal_session(kind, cx)"));
    assert!(body.contains("view.cancel_pending_portal_session(PortalKind::ClipboardHistory, cx)"));
}

// doc-anchor-removed: [[tests/acp-portal-contract#Host query seeding#Notes history popup uses originating view]]
#[test]
fn notes_portal_callback_uses_originating_view() {
    assert!(NOTES_ACP_HOST_SOURCE.contains("let portal_view = view.downgrade();"));
    assert!(
        NOTES_ACP_HOST_SOURCE.contains("if let Some(chat) = portal_view.upgrade()")
            && NOTES_ACP_HOST_SOURCE
                .contains("Self::handle_acp_portal_static(Some(chat), kind, cx);")
    );
}
