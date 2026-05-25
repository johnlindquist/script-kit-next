use super::*;

impl ScriptListApp {
    pub(crate) fn dismiss_focused_text_agent_chat_before_recapture(
        &mut self,
        cx: &mut Context<Self>,
    ) -> bool {
        let AppView::AcpChatView { entity } = self.current_view.clone() else {
            return false;
        };

        if !entity.read(cx).has_focused_text_context() {
            return false;
        }

        tracing::info!(
            target: "script_kit::focused_text",
            event = "focused_text_recapture_dismiss_previous_session",
        );

        self.close_tab_ai_harness_terminal_impl(
            None,
            super::TabAiHarnessCloseDisposition::CloseMainWindowStateFirst,
            cx,
        );
        true
    }

    pub(crate) fn open_focused_text_agent_chat_fixture(
        &mut self,
        text: Option<String>,
        instruction: Option<String>,
        provider: &'static str,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let text = text.unwrap_or_else(|| "Hello world".to_string());
        let requested_submit = instruction
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let snapshot =
            crate::platform::accessibility::focused_text::focused_text_snapshot_for_tests(text);

        self.open_focused_text_agent_chat_from_snapshot(
            snapshot,
            instruction.clone(),
            provider,
            cx,
        );

        let AppView::AcpChatView { entity } = self.current_view.clone() else {
            return Err("focused text fixture did not open Agent Chat".to_string());
        };

        if provider == "focused_text_mock_fixture" && requested_submit {
            let user_text = instruction
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let mut fixture_error = None;
            entity.update(cx, |chat, cx| {
                let result = chat.apply_test_fixture(
                    "idle",
                    user_text,
                    Some("Fixture focused text output.".to_string()),
                    cx,
                );
                if let Err(error) = result {
                    fixture_error = Some(error);
                }
            });
            if let Some(error) = fixture_error {
                return Err(error);
            }
        } else if requested_submit {
            entity.update(cx, |chat, cx| {
                if let Err(error) = chat.submit_focused_text_turn(
                    crate::ai::focused_text::FocusedTextEditSemantics::Replace,
                    cx,
                ) {
                    tracing::warn!(
                        target: "script_kit::focused_text",
                        event = "focused_text_fixture_submit_failed",
                        error = %error,
                    );
                }
            });
        }

        Ok(())
    }

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

        self.set_main_window_mode_state_only(
            MainWindowMode::Mini,
            cx,
            "focused_text_agent_chat_open",
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
            if let Err(error) = chat.stage_focused_text_from_host(snapshot, instruction, source, cx)
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
