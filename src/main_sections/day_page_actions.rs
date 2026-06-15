// Day Page contextual Cmd+K actions ("Today" section) and their executor.
//
// The Today section leads the shared ActionsDialog row list via
// `ActionsDialog::set_host_section`. Every row here must execute through
// `ScriptListApp::execute_day_page_action` with a visible state change —
// rows that cannot execute in the current state are omitted instead of
// rendered disabled.

pub(crate) const DAY_PAGE_ACTIONS_SECTION_TITLE: &str = "Today";
pub(crate) const DAY_PAGE_ASK_AGENT_CHAT_ACTION_ID: &str = "day_page:ask_agent_chat";
pub(crate) const DAY_PAGE_AGENT_CHAT_CONTEXT_SOURCE: &str = "day_page_today";

fn day_page_editor_action_id(toolbar_id: &str) -> String {
    format!("day_page:format_{}", toolbar_id.replace('-', "_"))
}

fn day_page_toolbar_id_from_action_id(action_id: &str) -> Option<String> {
    action_id
        .strip_prefix("day_page:format_")
        .map(|id| id.replace('_', "-"))
}

fn day_page_editor_action_shortcut(toolbar_id: &str) -> Option<&'static str> {
    match toolbar_id {
        "bold" => Some("cmd+b"),
        "italic" => Some("cmd+i"),
        "code" => Some("cmd+e"),
        "strikethrough" => Some("cmd+shift+x"),
        _ => None,
    }
}

/// Build the Today section rows for the current Day Page state.
pub(crate) fn day_page_host_actions_section(
    view: &DayPageView,
    cx: &App,
) -> Vec<crate::actions::Action> {
    use crate::actions::{Action, ActionCategory};

    let mut actions = Vec::new();
    let viewing_fragment = view.session.is_viewing_fragment();

    if viewing_fragment {
        actions.push(
            Action::new(
                "day_page:back_to_today",
                "Back to Today",
                Some("Return from the fragment to the day page".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("escape")
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    if view.session.is_viewing_note()
        || (!view.session.is_viewing_fragment() && view.session.bound_date().is_some())
    {
        actions.push(
            Action::new(
                "day_page:open_in_notes_window",
                "Open in Notes Window",
                Some("Move this note from the Day Page surface to the Notes window".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    if view.is_dirty() {
        actions.push(
            Action::new(
                "day_page:save",
                "Save Today",
                Some("Write the day page to disk now".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("cmd+s")
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    let viewing_today = view
        .session
        .bound_date()
        .is_some_and(|date| date == view.local_today());
    if !view.session.is_viewing_fragment() && viewing_today {
        actions.push(
            Action::new(
                DAY_PAGE_ASK_AGENT_CHAT_ACTION_ID,
                "Ask Agent Chat About Today",
                Some("Attach Today's brain to Agent Chat and start a question".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("cmd+enter")
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    let clipboard_text = cx
        .read_from_clipboard()
        .and_then(|item| item.text().map(|text| text.to_string()))
        .filter(|text| !text.trim().is_empty());
    if clipboard_text.is_some() {
        actions.push(
            Action::new(
                "day_page:insert_clipboard",
                "Insert Clipboard Text",
                Some("Paste the current clipboard text at the cursor".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    for item in crate::components::notes_editor::NOTES_EDITOR_TOOLBAR_ACTIONS {
        let toolbar_id = item.spec.id;
        actions.push(
            Action::new(
                day_page_editor_action_id(toolbar_id),
                crate::components::notes_editor::notes_editor_toolbar_action_title(toolbar_id),
                Some("Apply shared Notes editor Markdown formatting".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut_opt(day_page_editor_action_shortcut(toolbar_id).map(str::to_string))
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    actions
}

impl DayPageView {
    fn today_agent_chat_context_part(&self, cx: &App) -> crate::ai::message_parts::AiContextPart {
        let content = self.notes_editor.read(cx).content(cx);
        let date_label = self
            .session
            .bound_date()
            .map(|date| date.to_string())
            .unwrap_or_else(|| "Today".to_string());
        let source = self
            .session
            .path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| format!("brain://days/{date_label}"));

        crate::ai::message_parts::AiContextPart::TextBlock {
            label: format!("Today's brain - {date_label}"),
            source,
            text: if content.trim().is_empty() {
                "(Today's brain is empty.)".to_string()
            } else {
                content
            },
            mime_type: Some("text/markdown".to_string()),
        }
    }

    pub(crate) fn open_agent_chat_about_today(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save(cx);
        let part = self.today_agent_chat_context_part(cx);
        let Some(app) = self.app.upgrade() else {
            return;
        };

        window.defer(cx, move |_window, cx| {
            app.update(cx, |app, cx| {
                if app.has_day_page_context_round_trip_pending() {
                    tracing::warn!(
                        target: "script_kit::day_page",
                        event = "day_page_agent_chat_open_blocked",
                        reason = "context_round_trip_pending",
                    );
                    return;
                }
                app.clear_actions_popup_state();
                if crate::actions::is_actions_window_open() {
                    crate::actions::close_actions_window(cx);
                }
                app.open_tab_ai_agent_chat_with_context_part(
                    part,
                    DAY_PAGE_AGENT_CHAT_CONTEXT_SOURCE,
                    cx,
                );
                if let AppView::AgentChatView { entity } = &app.current_view {
                    let entity = entity.clone();
                    entity.update(cx, |chat, cx| {
                        chat.set_input("Ask about Today's brain: ".to_string(), cx);
                    });
                }
            });
        });
    }

    pub(crate) fn append_agent_chat_response_to_today_file(
        &mut self,
        response: &str,
    ) -> anyhow::Result<()> {
        let response = response.trim();
        if response.is_empty() {
            anyhow::bail!("empty Agent Chat response");
        }
        let local = Utc::now().with_timezone(&self.session.substrate().timezone());
        let timestamp = local.format("%H:%M").to_string();
        self.session
            .append_external_line_to_bound_file(&format!("{timestamp} Agent Chat\n\n{response}"))?;
        script_kit_gpui::brain::wake_indexer();
        Ok(())
    }

    pub(crate) fn insert_clipboard_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = cx
            .read_from_clipboard()
            .and_then(|item| item.text().map(|text| text.to_string()))
        else {
            return;
        };
        if text.is_empty() {
            return;
        }
        self.notes_editor.update(cx, |editor, cx| {
            let input = editor.input_state();
            input.update(cx, |state, cx| {
                state.replace(&text, window, cx);
            });
        });
        cx.notify();
    }

    pub(crate) fn run_shared_markdown_toolbar_action(
        &mut self,
        toolbar_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(action) =
            crate::components::notes_editor::notes_editor_toolbar_action_by_id(toolbar_id)
        else {
            return false;
        };
        self.notes_editor.update(cx, |editor, cx| {
            (action.run)(editor, window, cx);
        });
        self.sync_footer(window, cx);
        cx.notify();
        true
    }

}

impl ScriptListApp {
    /// Execute a `day_page:*` actions-dialog row. Returns true when handled.
    pub(crate) fn execute_day_page_action(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let AppView::DayPage { entity } = &self.current_view else {
            return false;
        };
        let entity = entity.clone();
        let handled = match action_id {
            "day_page:save" => {
                entity.update(cx, |view, cx| {
                    view.save_and_sync_footer(window, cx);
                });
                true
            }
            "day_page:open_past_day" => {
                let window_handle = window.window_handle();
                window.defer(cx, move |_window, cx| {
                    cx.defer(move |cx| {
                        let _ = window_handle.update(cx, |_root, window, cx| {
                            entity.update(cx, |view, cx| view.open_note_switcher(window, cx));
                        });
                    });
                });
                true
            }
            "day_page:open_in_notes_window" => {
                let (note_id, day_date) = {
                    let view = entity.read(cx);
                    (
                        view.session.viewing_note_id().map(str::to_string),
                        view.session.bound_date(),
                    )
                };
                if let Some(note_id) = note_id.and_then(|id| crate::notes::NoteId::parse(&id)) {
                    if let Err(error) = crate::notes::open_note_in_notes_window(cx, note_id) {
                        tracing::warn!(
                            target: "script_kit::day_page",
                            error = %error,
                            "day_page_open_in_notes_window_failed"
                        );
                    }
                    true
                } else if let Some(date) = day_date {
                    if let Err(error) = crate::notes::open_day_note_in_notes_window(cx, date) {
                        tracing::warn!(
                            target: "script_kit::day_page",
                            error = %error,
                            "day_page_open_day_in_notes_window_failed"
                        );
                    }
                    true
                } else {
                    false
                }
            }
            DAY_PAGE_ASK_AGENT_CHAT_ACTION_ID => {
                entity.update(cx, |view, cx| view.open_agent_chat_about_today(window, cx));
                true
            }
            "day_page:back_to_today" => {
                entity.update(cx, |view, cx| view.return_to_day_page(window, cx));
                true
            }
            "day_page:insert_clipboard" => {
                entity.update(cx, |view, cx| view.insert_clipboard_text(window, cx));
                true
            }
            _ => day_page_toolbar_id_from_action_id(action_id)
                .map(|toolbar_id| {
                    entity.update(cx, |view, cx| {
                        view.run_shared_markdown_toolbar_action(&toolbar_id, window, cx)
                    })
                })
                .unwrap_or(false),
        };
        if handled {
            tracing::info!(
                target: "script_kit::day_page",
                event = "day_page_action_executed",
                action_id = %action_id,
            );
        }
        handled
    }
}

#[cfg(test)]
mod day_page_markdown_action_tests {
    use super::*;
    use crate::components::notes_editor::NOTES_EDITOR_TOOLBAR_ACTIONS;

    #[test]
    fn day_page_markdown_action_catalog_covers_notes_toolbar_actions() {
        for item in NOTES_EDITOR_TOOLBAR_ACTIONS {
            let action_id = day_page_editor_action_id(item.spec.id);
            let round_trip = day_page_toolbar_id_from_action_id(&action_id)
                .expect("day page action id should map back to toolbar id");
            assert_eq!(round_trip, item.spec.id);
        }
    }

    #[test]
    fn day_page_legacy_format_action_ids_are_preserved() {
        assert_eq!(day_page_editor_action_id("bold"), "day_page:format_bold");
        assert_eq!(
            day_page_editor_action_id("italic"),
            "day_page:format_italic"
        );
        assert_eq!(day_page_editor_action_id("code"), "day_page:format_code");
        assert_eq!(
            day_page_editor_action_id("strikethrough"),
            "day_page:format_strikethrough"
        );
    }

    #[test]
    fn day_page_hyphenated_toolbar_ids_round_trip() {
        assert_eq!(
            day_page_toolbar_id_from_action_id("day_page:format_numbered_list").as_deref(),
            Some("numbered-list")
        );
    }
}
