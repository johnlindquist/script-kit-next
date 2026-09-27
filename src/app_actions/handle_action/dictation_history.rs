#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DictationHistoryHandlerAction {
    Paste,
    AddToAgentChat,
    SaveNote,
    Copy,
    Delete,
}

impl DictationHistoryHandlerAction {
    fn from_action_id(action_id: &str) -> Option<Self> {
        match action_id {
            "dictation_history_paste" => Some(Self::Paste),
            "dictation_history_add_to_agent_chat" => Some(Self::AddToAgentChat),
            "dictation_history_save_note" => Some(Self::SaveNote),
            "dictation_history_copy" => Some(Self::Copy),
            "dictation_history_delete" => Some(Self::Delete),
            _ => None,
        }
    }

    fn selection_required_message(self) -> &'static str {
        match self {
            Self::Paste | Self::AddToAgentChat | Self::SaveNote | Self::Copy | Self::Delete => {
                "No dictation selected"
            }
        }
    }

    fn user_message(self) -> Option<&'static str> {
        match self {
            Self::Paste => Some("Pasting to frontmost app…"),
            Self::AddToAgentChat => Some("Asking the model about it"),
            Self::SaveNote | Self::Copy | Self::Delete => None,
        }
    }

    fn success_hud(self) -> Option<&'static str> {
        match self {
            Self::SaveNote => Some("Saved dictation as note"),
            Self::Copy => Some("Copied dictation to clipboard"),
            Self::Delete => Some("Deleted dictation"),
            Self::Paste | Self::AddToAgentChat => None,
        }
    }

    fn error_prefix(self) -> Option<&'static str> {
        match self {
            Self::SaveNote => Some("Failed to save note"),
            Self::Delete => Some("Failed to delete dictation"),
            Self::Paste | Self::AddToAgentChat | Self::Copy => None,
        }
    }

    fn failure_message(self, error: impl std::fmt::Display) -> String {
        let prefix = self
            .error_prefix()
            .unwrap_or("Failed to complete dictation action");
        format!("{prefix}: {error}")
    }
}

impl ScriptListApp {
    fn refresh_dictation_history_selection_after_delete(&mut self) {
        if let AppView::DictationHistoryView {
            filter,
            selected_index,
            visible_limit,
        } = &mut self.current_view
        {
            let filtered_len = crate::dictation::search_history_page(filter, 0, *visible_limit)
                .map(|page| page.rows.len())
                .unwrap_or(0);

            if filtered_len > 0 {
                *selected_index = (*selected_index).min(filtered_len.saturating_sub(1));
                self.dictation_history_scroll_handle
                    .scroll_to_item(*selected_index);
            } else {
                *selected_index = 0;
            }
        }
    }

    /// Whether the entry waits in the line to go with the chat's next turn.
    pub(crate) fn dictation_history_attachment_is_pending(
        &self,
        entry_id: &str,
        _cx: &Context<Self>,
    ) -> bool {
        let uri = format!("kit://dictation-history?id={entry_id}");
        self.one_window_host.work.parts.values().any(|part| {
            matches!(
                part,
                crate::ai::AiContextPart::ResourceUri { uri: pending_uri, .. }
                    if *pending_uri == uri
            )
        })
    }

    pub(crate) fn delete_dictation_history_entry_confirmed(
        &mut self,
        entry_id: &str,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = crate::dictation::delete_history_entry(entry_id) {
            tracing::warn!(
                category = "DICTATION",
                entry_id,
                error_fingerprint = %crate::dictation::redacted_transcript_fingerprint(&error.to_string()),
                "Failed to delete Dictation History entry",
            );
            self.show_error_toast("Failed to delete dictation", cx);
            return;
        }
        self.refresh_dictation_history_selection_after_delete();
        self.show_hud("Deleted dictation".to_string(), Some(HUD_MEDIUM_MS), cx);
        cx.notify();
    }

    fn handle_dictation_history_action(
        &mut self,
        action_id: &str,
        selected_entry: Option<crate::dictation::DictationHistoryEntry>,
        _dctx: &DispatchContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        match action_id {
            "dictation_history_paste" => {
                let Some(history_action) = DictationHistoryHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(entry) = selected_entry else {
                    self.show_error_toast(history_action.selection_required_message(), cx);
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
                outcome.user_message = history_action.user_message().map(String::from);
                outcome
            }
            "dictation_history_add_to_agent_chat" => {
                let Some(history_action) = DictationHistoryHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(entry) = selected_entry else {
                    self.show_error_toast(history_action.selection_required_message(), cx);
                    return DispatchOutcome::success();
                };

                self.one_window_open_chat(
                    crate::one_window::chat_entry::ChatEntry::quick().with(
                        crate::ai::AiContextPart::ResourceUri {
                            uri: entry.resource_uri(),
                            label: format!("Dictation: {}", entry.preview),
                        },
                    ),
                    cx,
                );

                let mut outcome = DispatchOutcome::success();
                outcome.user_message = history_action.user_message().map(String::from);
                outcome
            }
            "dictation_history_save_note" => {
                let Some(history_action) = DictationHistoryHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(entry) = selected_entry else {
                    self.show_error_toast(history_action.selection_required_message(), cx);
                    return DispatchOutcome::success();
                };

                match crate::notes::save_note_with_content(cx, entry.transcript) {
                    Ok(()) => {
                        if let Some(message) = history_action.success_hud() {
                            self.show_hud(message.to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    }
                    Err(error) => {
                        return DispatchOutcome::error(
                            crate::action_helpers::ERROR_ACTION_FAILED,
                            history_action.failure_message(error),
                        );
                    }
                }

                DispatchOutcome::success()
            }
            "dictation_history_copy" => {
                let Some(history_action) = DictationHistoryHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(entry) = selected_entry else {
                    self.show_error_toast(history_action.selection_required_message(), cx);
                    return DispatchOutcome::success();
                };

                cx.write_to_clipboard(gpui::ClipboardItem::new_string(entry.transcript));
                if let Some(message) = history_action.success_hud() {
                    self.show_hud(message.to_string(), Some(HUD_MEDIUM_MS), cx);
                }
                DispatchOutcome::success()
            }
            "dictation_history_delete" => {
                let Some(history_action) = DictationHistoryHandlerAction::from_action_id(action_id)
                else {
                    return DispatchOutcome::not_handled();
                };
                let Some(entry) = selected_entry else {
                    self.show_error_toast(history_action.selection_required_message(), cx);
                    return DispatchOutcome::success();
                };

                let pending = self.dictation_history_attachment_is_pending(&entry.id, cx);
                let body = crate::dictation::delete_history_confirmation_body(pending);
                let entry_id = entry.id.clone();
                let owner = cx.entity().downgrade();
                let owner_for_confirm = owner.clone();
                self.was_window_focused = true;
                crate::confirm::open_parent_confirm_dialog_for_entity(
                    window,
                    cx,
                    owner,
                    crate::confirm::ParentConfirmOptions::irreversible(
                        "Delete Dictation?",
                        body,
                        "Delete",
                    ),
                    move |_window, cx| {
                        if let Some(entity) = owner_for_confirm.upgrade() {
                            entity.update(cx, |this, cx| {
                                this.delete_dictation_history_entry_confirmed(&entry_id, cx);
                            });
                        }
                    },
                    |_window, _cx| {},
                );
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
