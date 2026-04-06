use super::*;

impl ScriptListApp {
    /// Whether the app is currently in an attachment portal (file search or
    /// clipboard history opened from the ACP chat context picker).
    pub(crate) fn is_in_attachment_portal(&self) -> bool {
        self.attachment_portal_return_view.is_some()
    }

    /// Open a full built-in view as an attachment portal. The user browses
    /// files or clipboard entries; Enter attaches the selection back to the
    /// ACP chat, Escape cancels and returns.
    pub(crate) fn open_attachment_portal(
        &mut self,
        kind: crate::ai::window::context_picker::types::PortalKind,
        cx: &mut Context<Self>,
    ) {
        use crate::ai::window::context_picker::types::PortalKind;

        // Prevent nesting — only one portal at a time.
        if self.is_in_attachment_portal() {
            tracing::warn!(
                target: "script_kit::acp",
                event = "attachment_portal_nested_prevented",
            );
            return;
        }

        // Save the current view and focus target for restoration on return.
        // The portal is always opened from AcpChatView, so ChatPrompt is correct.
        self.attachment_portal_return_view = Some(self.current_view.clone());
        self.attachment_portal_return_focus_target = Some(FocusTarget::ChatPrompt);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_opened",
            kind = ?kind,
        );

        match kind {
            PortalKind::FileSearch => {
                self.open_file_search(String::new(), cx);
            }
            PortalKind::ClipboardHistory => {
                self.cached_clipboard_entries =
                    crate::clipboard_history::get_cached_entries(100);
                self.open_builtin_filterable_view(
                    AppView::ClipboardHistoryView {
                        filter: String::new(),
                        selected_index: 0,
                    },
                    "Search clipboard history...",
                    cx,
                );
            }
        }

        cx.notify();
    }

    /// Close the attachment portal and attach the selected part to the ACP chat.
    pub(crate) fn close_attachment_portal_with_part(
        &mut self,
        part: crate::ai::message_parts::AiContextPart,
        cx: &mut Context<Self>,
    ) {
        let return_view = self
            .attachment_portal_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_focus_target = self
            .attachment_portal_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_closed_with_part",
            focus_target = ?return_focus_target,
        );

        self.current_view = return_view.clone();
        self.pending_focus = Some(return_focus_target);
        self.focused_input = match return_focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };

        // Stage the context part as a chip-only attachment (no inline text).
        // The chip renders above the composer input from pending_context_parts.
        // We intentionally do NOT insert inline @mention text because:
        // - Full file paths are too long and ugly as inline text
        // - Chips are the correct visual for portal-attached context
        if let AppView::AcpChatView { entity } = &return_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| {
                view.live_thread().update(cx, |thread, cx| {
                    thread.add_context_part(part, cx);
                });
                cx.notify();
            });
        }

        cx.notify();
    }

    /// Close the attachment portal without attaching anything (Escape).
    pub(crate) fn close_attachment_portal_cancel(&mut self, cx: &mut Context<Self>) {
        let return_view = self
            .attachment_portal_return_view
            .take()
            .unwrap_or(AppView::ScriptList);
        let return_focus_target = self
            .attachment_portal_return_focus_target
            .take()
            .unwrap_or(FocusTarget::MainFilter);

        tracing::info!(
            target: "script_kit::acp",
            event = "attachment_portal_cancelled",
            focus_target = ?return_focus_target,
        );

        self.current_view = return_view;
        self.pending_focus = Some(return_focus_target);
        self.focused_input = match return_focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };

        cx.notify();
    }
}
