// Day Page contextual Cmd+K actions ("Today" section) and their executor.
//
// The Today section leads the shared ActionsDialog row list via
// `ActionsDialog::set_host_section`. Every row here must execute through
// `ScriptListApp::execute_day_page_action` with a visible state change —
// rows that cannot execute in the current state are omitted instead of
// rendered disabled.

pub(crate) const DAY_PAGE_ACTIONS_SECTION_TITLE: &str = "Today";

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

    actions.push(
        Action::new(
            "day_page:open_past_day",
            "Open Past Day…",
            Some("Search and swap to a previous day page".to_string()),
            ActionCategory::ScriptContext,
        )
        .with_shortcut("cmd+p")
        .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
    );

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

    if !view.current_line_text(cx).trim().is_empty() {
        actions.push(
            Action::new(
                "day_page:handoff_line",
                "Send Line to Agent Chat",
                Some("Hand off the current line plus accepted context".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut("cmd+enter")
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    for (id, title, prefix_suffix, shortcut) in [
        ("day_page:format_bold", "Bold", ("**", "**"), Some("cmd+b")),
        ("day_page:format_italic", "Italic", ("_", "_"), Some("cmd+i")),
        ("day_page:format_code", "Code", ("`", "`"), Some("cmd+e")),
        (
            "day_page:format_strikethrough",
            "Strikethrough",
            ("~~", "~~"),
            Some("cmd+shift+x"),
        ),
    ] {
        let _ = prefix_suffix;
        actions.push(
            Action::new(
                id,
                title,
                Some("Markdown formatting (same as Notes)".to_string()),
                ActionCategory::ScriptContext,
            )
            .with_shortcut_opt(shortcut.map(str::to_string))
            .with_section(DAY_PAGE_ACTIONS_SECTION_TITLE),
        );
    }

    actions
}

impl DayPageView {
    /// Current line under the cursor (used for Agent Chat handoff gating).
    pub(crate) fn current_line_text(&self, cx: &App) -> String {
        let content = self.notes_editor.read(cx).content(cx);
        let selection = self.notes_editor.read(cx).selection(cx);
        let cursor = clamp_to_char_boundary(&content, selection.end.min(content.len()));
        let line_range = current_line_range(&content, cursor);
        content[line_range].to_string()
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

    pub(crate) fn insert_markdown_formatting(
        &mut self,
        prefix: &str,
        suffix: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.insert_formatting(prefix, suffix, window, cx);
        });
        cx.notify();
    }

    /// Explicit Agent Chat handoff for the current line. Falls back to a plain
    /// prompt submission when the line carries no spine sigils/mentions.
    pub(crate) fn handoff_current_line_to_agent_chat(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.submit_day_page_spine_prompt_from_current_line(window, cx) {
            return;
        }
        let line = self.current_line_text(cx).trim().to_string();
        if line.is_empty() {
            return;
        }
        let Some(app) = self.app.upgrade() else {
            return;
        };
        window.defer(cx, move |_window, cx| {
            app.update(cx, |app, cx| {
                app.day_page_handoff_plain_line(line, cx);
            });
        });
    }
}

impl ScriptListApp {
    /// Hand a plain prose line (no sigils) from the Day Page to Agent Chat.
    pub(crate) fn day_page_handoff_plain_line(&mut self, prompt: String, cx: &mut Context<Self>) {
        self.embedded_agent_chat = None;
        self.open_tab_ai_agent_chat_with_entry_intent_suppressing_focused_part(None, cx);
        if let AppView::AgentChatView { entity } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |chat, cx| {
                if let Err(error) = chat.submit_reused_entry_intent_with_host_context(
                    prompt,
                    Vec::new(),
                    "day_page_line_handoff",
                    cx,
                ) {
                    tracing::warn!(
                        target: "script_kit::day_page",
                        event = "day_page_line_handoff_failed",
                        error = %error,
                    );
                }
            });
        }
        cx.notify();
    }

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
                entity.update(cx, |view, cx| view.open_day_switcher(window, cx));
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
            "day_page:handoff_line" => {
                entity.update(cx, |view, cx| {
                    view.handoff_current_line_to_agent_chat(window, cx)
                });
                true
            }
            "day_page:format_bold" => {
                entity.update(cx, |view, cx| {
                    view.insert_markdown_formatting("**", "**", window, cx)
                });
                true
            }
            "day_page:format_italic" => {
                entity.update(cx, |view, cx| {
                    view.insert_markdown_formatting("_", "_", window, cx)
                });
                true
            }
            "day_page:format_code" => {
                entity.update(cx, |view, cx| {
                    view.insert_markdown_formatting("`", "`", window, cx)
                });
                true
            }
            "day_page:format_strikethrough" => {
                entity.update(cx, |view, cx| {
                    view.insert_markdown_formatting("~~", "~~", window, cx)
                });
                true
            }
            _ => false,
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
