use crate::ai::TabAiTargetContext;

/// Shared handoff helper for secondary windows (Notes, detached actions, etc.)
/// that need to send a canonical ACP target to the main window.
///
/// This avoids per-surface bespoke string injection and ensures every
/// secondary surface produces the same target shape that ACP consumes.
///
/// The caller is responsible for showing or activating the main window
/// after enqueueing, since different surfaces have different activation
/// requirements (e.g. detached actions use `platform::activate_main_window`
/// with specific timing, while Notes may use a different mechanism).
pub(crate) fn request_explicit_acp_handoff_from_secondary_window(
    target: TabAiTargetContext,
    source_window: &'static str,
    show_main_window: bool,
) {
    tracing::info!(
        target: "script_kit::tab_ai",
        event = "secondary_window_explicit_acp_handoff_requested",
        source_window,
        item_source = %target.source,
        item_kind = %target.kind,
        semantic_id = %target.semantic_id,
        show_main_window,
    );

    crate::ai::enqueue_explicit_acp_target(target);

    if show_main_window {
        crate::platform::activate_main_window();
        tracing::info!(
            target: "script_kit::tab_ai",
            event = "secondary_window_explicit_acp_handoff_main_window_requested",
            source_window,
        );
    }
}
