impl ScriptListApp {
    fn refresh_dictation_history_selection_after_delete(&mut self) {
        if let AppView::DictationHistoryView {
            filter,
            selected_index,
        } = &mut self.current_view
        {
            let filtered_len = crate::dictation::search_history(filter, 100).len();

            if filtered_len > 0 {
                *selected_index = (*selected_index).min(filtered_len.saturating_sub(1));
                self.dictation_history_scroll_handle
                    .scroll_to_item(*selected_index);
            } else {
                *selected_index = 0;
            }
        }
    }

    fn handle_dictation_history_action(
        &mut self,
        action_id: &str,
        selected_entry: Option<crate::dictation::DictationHistoryEntry>,
        dctx: &DispatchContext,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        match action_id {
            "dictation_history_paste" => {
                let Some(entry) = selected_entry else {
                    self.show_error_toast("No dictation selected", cx);
                    return DispatchOutcome::success();
                };

                let transcript = entry.transcript.clone();
                self.hide_main_and_reset(cx);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    let injector = crate::text_injector::TextInjector::new();
                    if let Err(error) = injector.paste_text(&transcript) {
                        tracing::warn!(%error, "dictation_history_paste_failed");
                    }
                });

                let mut outcome = DispatchOutcome::success();
                outcome.user_message = Some("Pasting to frontmost app…".to_string());
                outcome
            }
            "dictation_history_attach_to_ai" => {
                let Some(entry) = selected_entry else {
                    self.show_error_toast("No dictation selected", cx);
                    return DispatchOutcome::success();
                };

                self.open_ai_window_after_main_hide(
                    action_id,
                    &dctx.trace_id,
                    DeferredAiWindowAction::SetInput {
                        text: entry.transcript,
                        submit: false,
                    },
                    cx,
                );

                let mut outcome = DispatchOutcome::success();
                outcome.user_message = Some("Opening Agent Chat...".to_string());
                outcome
            }
            "dictation_history_save_note" => {
                let Some(entry) = selected_entry else {
                    self.show_error_toast("No dictation selected", cx);
                    return DispatchOutcome::success();
                };

                match crate::notes::save_note_with_content(&mut **cx, entry.transcript) {
                    Ok(()) => {
                        self.show_hud(
                            "Saved dictation as note".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                    Err(error) => {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            format!("Failed to save note: {error}"),
                        );
                    }
                }

                DispatchOutcome::success()
            }
            "dictation_history_copy" => {
                let Some(entry) = selected_entry else {
                    self.show_error_toast("No dictation selected", cx);
                    return DispatchOutcome::success();
                };

                cx.write_to_clipboard(gpui::ClipboardItem::new_string(entry.transcript));
                self.show_hud(
                    "Copied dictation to clipboard".to_string(),
                    Some(HUD_MEDIUM_MS),
                    cx,
                );
                DispatchOutcome::success()
            }
            "dictation_history_delete" => {
                let Some(entry) = selected_entry else {
                    self.show_error_toast("No dictation selected", cx);
                    return DispatchOutcome::success();
                };

                if let Err(error) = crate::dictation::delete_history_entry(&entry.id) {
                    return DispatchOutcome::error(
                        crate::action_helpers::ERROR_ACTION_FAILED,
                        format!("Failed to delete dictation: {error}"),
                    );
                }

                self.refresh_dictation_history_selection_after_delete();
                self.show_hud("Deleted dictation".to_string(), Some(HUD_MEDIUM_MS), cx);
                cx.notify();
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
