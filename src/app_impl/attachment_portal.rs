use super::*;

impl ScriptListApp {
    fn restore_attachment_portal_return_view(
        &mut self,
        return_view: AppView,
        return_focus_target: FocusTarget,
    ) {
        self.current_view = return_view;
        self.pending_focus = Some(return_focus_target);
        self.focused_input = match return_focus_target {
            FocusTarget::MainFilter => FocusedInput::MainFilter,
            FocusTarget::ActionsDialog => FocusedInput::ActionsSearch,
            _ => FocusedInput::None,
        };

        // Portal views can temporarily expand the window; restore the
        // originating surface sizing before control returns to the user.
        self.update_window_size();
    }

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
                self.cached_clipboard_entries = crate::clipboard_history::get_cached_entries(100);
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

        self.restore_attachment_portal_return_view(return_view.clone(), return_focus_target);

        // Stage the context part with an inline @mention token.
        // Uses the canonical token format so the mention sync system can track
        // it — deleting characters from the mention removes the part.
        if let AppView::AcpChatView { entity } = &return_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| {
                let inline_token = crate::ai::context_mentions::part_to_inline_token(&part)
                    .unwrap_or_else(|| format!("@{}", part.label()));

                // Append the inline token to the current input text.
                let current_text = view.live_thread().read(cx).input.text().to_string();
                let separator = if current_text.is_empty() || current_text.ends_with(' ') {
                    ""
                } else {
                    " "
                };
                let new_text = format!("{current_text}{separator}{inline_token} ");
                let new_cursor = new_text.len();

                view.live_thread().update(cx, |thread, cx| {
                    thread.input.set_text(new_text);
                    thread.input.set_cursor(new_cursor);
                    thread.add_context_part(part, cx);
                    cx.notify();
                });

                // Register inline ownership so deleting the mention removes the part.
                view.register_inline_owned_token(inline_token);
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

        self.restore_attachment_portal_return_view(return_view, return_focus_target);

        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn attachment_portal_source() -> String {
        fs::read_to_string("src/app_impl/attachment_portal.rs")
            .expect("Failed to read src/app_impl/attachment_portal.rs")
    }

    #[test]
    fn attachment_portal_restore_helper_reapplies_window_size_contract() {
        let source = attachment_portal_source();
        let helper_start = source
            .find("fn restore_attachment_portal_return_view(")
            .expect("restore helper must exist");
        let helper_body = &source[helper_start..];

        assert!(
            helper_body.contains("self.update_window_size();"),
            "restore helper must reapply the originating surface window size"
        );
    }

    #[test]
    fn attachment_portal_exit_paths_use_shared_restore_helper() {
        let source = attachment_portal_source();
        let helper_calls = source
            .matches("self.restore_attachment_portal_return_view(")
            .count();

        assert!(
            helper_calls >= 2,
            "attach + cancel portal exits must share the same restore helper"
        );
    }
}
