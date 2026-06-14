// Day Page contextual Cmd+K actions ("Today" section) and their executor.
//
// The Today section leads the shared ActionsDialog row list via
// `ActionsDialog::set_host_section`. Every row here must execute through
// `ScriptListApp::execute_day_page_action` with a visible state change —
// rows that cannot execute in the current state are omitted instead of
// rendered disabled.

pub(crate) const DAY_PAGE_ACTIONS_SECTION_TITLE: &str = "Today";

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
