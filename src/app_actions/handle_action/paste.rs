// Shared paste finalization for all clipboard-backed paste flows.
//
// Both emoji picker and clipboard history route through `finalize_paste_after_clipboard_ready`
// so the two paths cannot drift apart.

/// Whether to hide the window after pasting or keep it open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum PasteCloseBehavior {
    HideWindow,
    KeepWindowOpen,
}

impl ScriptListApp {
    /// Finalize a paste after the clipboard contents are already prepared.
    ///
    /// This is the single shared boundary for all clipboard-backed paste flows
    /// so clipboard history and emoji picker cannot diverge.
    fn finalize_paste_after_clipboard_ready(
        &mut self,
        source_kind: &str,
        source_id: &str,
        close_behavior: PasteCloseBehavior,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        tracing::info!(
            action = "finalize_paste_after_clipboard_ready",
            source_kind,
            source_id,
            close_behavior = %close_behavior,
            paste_strategy = "clipboard_then_simulated_cmd_v",
            status = "start",
            "starting paste finalization"
        );

        if matches!(close_behavior, PasteCloseBehavior::HideWindow) {
            self.hide_main_and_reset(cx);
        }

        self.spawn_clipboard_paste_simulation();

        tracing::info!(
            action = "finalize_paste_after_clipboard_ready",
            source_kind,
            source_id,
            close_behavior = %close_behavior,
            paste_strategy = "clipboard_then_simulated_cmd_v",
            status = "queued",
            "paste queued"
        );

        DispatchOutcome::success()
    }
}