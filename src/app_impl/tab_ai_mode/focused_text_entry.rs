use super::*;

impl ScriptListApp {
    pub(crate) fn open_focused_text_agent_chat_from_snapshot(
        &mut self,
        snapshot: crate::platform::accessibility::FocusedTextSnapshot,
        instruction: Option<String>,
        source: &'static str,
        cx: &mut Context<Self>,
    ) {
        let source_view = self.current_view.clone();
        self.seed_acp_return_origin_for_view(&source_view);

        tracing::info!(
            target: "script_kit::focused_text",
            event = "focused_text_agent_chat_open",
            source,
            session_id = %snapshot.session_id,
            app_name = %snapshot.app.name,
            chars = snapshot.metrics.chars,
            source_view = ?source_view,
        );

        self.begin_tab_ai_harness_entry_from_source_view(
            source_view,
            None,
            true,
            None,
            crate::ai::TabAiCaptureKind::DefaultContext,
            // force_acp_surface: focused-text apply semantics must not route to the terminal.
            true,
            crate::ai::acp::ui_variant::AcpChatUiVariant::FocusedTextMini,
            cx,
        );

        let AppView::AcpChatView { entity } = self.current_view.clone() else {
            tracing::warn!(
                target: "script_kit::focused_text",
                event = "focused_text_agent_chat_open_failed_no_embedded_view",
            );
            return;
        };

        entity.update(cx, |chat, cx| {
            chat.set_ui_variant(
                crate::ai::acp::ui_variant::AcpChatUiVariant::FocusedTextMini,
                cx,
            );
            if let Err(error) =
                chat.stage_focused_text_from_host(snapshot, instruction, source, cx)
            {
                tracing::warn!(
                    target: "script_kit::focused_text",
                    event = "focused_text_agent_chat_stage_failed",
                    error = %error,
                );
            }
        });
        cx.notify();
    }
}
