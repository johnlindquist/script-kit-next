//! Dictation's lists as views under the input: which microphone it listens to, and what you
//! said before. Enter picks the microphone, or pastes the words at the front app's cursor.

use gpui::Context;
use sk_one_window::{Done, EnterTarget, Event, Row};

use super::learns::OwnView;
use crate::ScriptListApp;

impl ScriptListApp {
    fn one_window_microphone_rows(&self) -> Vec<Row> {
        let prefs = crate::config::load_user_preferences();
        let items = crate::dictation::list_input_device_menu_items(
            prefs.dictation.selected_device_id.as_deref(),
        )
        .unwrap_or_default();
        items
            .into_iter()
            .map(|item| {
                let value = crate::dictation::device_selection_value(&item.action);
                Row::new(format!("mic:{value}"), item.title)
                    .with_sub(if item.is_selected {
                        format!("Listening · {}", item.subtitle)
                    } else {
                        item.subtitle
                    })
                    .with_icon(if item.is_selected { "check" } else { "mic" })
                    .with_enter_label("Listen to this one")
            })
            .collect()
    }

    pub(super) fn one_window_open_microphones(&mut self, cx: &mut Context<Self>) {
        let rows = self.one_window_microphone_rows();
        let learns = &mut self.one_window_host.learns;
        learns.view_head = "Dictation listens to".to_string();
        learns.view_rows = rows;
        self.one_window_open_view(
            OwnView::Microphones,
            "Microphone".to_string(),
            Some(EnterTarget::new("mic", "Listen to this one")),
            true,
            cx,
        );
    }

    pub(super) fn one_window_microphone_enter(&mut self, row: &Row, cx: &mut Context<Self>) {
        let Some(value) = row.key.strip_prefix("mic:") else {
            return;
        };
        let action = crate::dictation::device_selection_action_from_value(value);
        if let Err(error) = crate::dictation::apply_device_selection(&action) {
            tracing::warn!(target: "script_kit::one_window", %error, "microphone_not_changed");
            return;
        }
        self.one_window_handle(
            Event::Done(Done {
                icon: "check".into(),
                title: format!("Dictation listens to {}", row.title),
                ..Done::default()
            }),
            cx,
        );
    }

    fn one_window_history_rows() -> Vec<Row> {
        crate::dictation::load_history()
            .into_iter()
            .rev()
            .map(|entry| {
                let when = crate::dictation::format_history_timestamp(&entry.timestamp);
                let target = if entry.target_label_snapshot.is_empty() {
                    when
                } else {
                    format!("{when} · {}", entry.target_label_snapshot)
                };
                Row::new(format!("history:{}", entry.id), entry.preview)
                    .with_sub(target)
                    .with_icon("mic")
                    .with_enter_label("Paste it at the cursor")
            })
            .collect()
    }

    pub(super) fn one_window_open_dictation_history(&mut self, cx: &mut Context<Self>) {
        let learns = &mut self.one_window_host.learns;
        learns.view_head = "What you said".to_string();
        learns.view_rows = Self::one_window_history_rows();
        self.one_window_open_view(
            OwnView::DictationHistory,
            "Dictation History".to_string(),
            Some(EnterTarget::new("history", "Paste it at the cursor")),
            true,
            cx,
        );
    }

    /// Pastes a transcript at the front app's cursor, the way letting go of the voice key does.
    pub(super) fn one_window_history_enter(&mut self, row: &Row, cx: &mut Context<Self>) {
        let Some(id) = row.key.strip_prefix("history:") else {
            return;
        };
        let Some(entry) = crate::dictation::get_history_entry(id) else {
            return;
        };
        let app = crate::ui_window::footer_frontmost_app_name().unwrap_or_default();
        self.one_window_paste(entry.transcript, app, cx);
        self.hide_main_and_reset(cx);
    }
}
