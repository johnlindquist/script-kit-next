//! Dictation's lists as views under the input: which microphone it listens to, and what you
//! said before. Enter picks the microphone, or pastes the words at the front app's cursor.

use gpui::Context;
use sk_one_window::{Done, EnterTarget, Event, Row};

use super::learns::OwnView;
use crate::scripts::SearchResult;
use crate::ScriptListApp;

/// The microphone picker's items as rows; the one dictation listens to is checked.
pub(crate) fn microphone_rows(items: Vec<crate::dictation::DictationDeviceMenuItem>) -> Vec<Row> {
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

impl ScriptListApp {
    pub(crate) fn one_window_open_microphones(&mut self, cx: &mut Context<Self>) {
        let prefs = crate::config::load_user_preferences();
        let selected = prefs.dictation.selected_device_id.as_deref();
        let items = match &self.one_window_host.learns.fixture_microphones {
            Some(devices) => crate::dictation::build_device_menu_items(devices, selected),
            None => crate::dictation::list_input_device_menu_items(selected).unwrap_or_default(),
        };
        let rows = microphone_rows(items);
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
            self.one_window_notice(
                crate::line_notice::FAILED,
                format!("Couldn't switch Dictation to {}.", row.title),
                cx,
            );
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

    /// What you said, newest first, as rows under the view's header.
    fn one_window_history_rows(&mut self) -> Vec<Row> {
        let entries = crate::dictation::load_history();
        let rows = entries
            .iter()
            .map(|entry| Row::new(format!("history:{}", entry.id), entry.preview.clone()))
            .collect();
        self.one_window_host.learns.history = entries;
        rows
    }

    /// The saved dictation a history row lists, as the transcript row root search shows.
    pub(super) fn one_window_history_result(&self, row: &Row) -> Option<SearchResult> {
        let id = row.key.strip_prefix("history:")?;
        let entry = self
            .one_window_host
            .learns
            .history
            .iter()
            .find(|entry| entry.id == id)?;
        let time = crate::dictation::format_history_timestamp(&entry.timestamp);
        let duration = crate::dictation::format_history_duration_ms(entry.audio_duration_ms);
        Some(SearchResult::DictationHistory(
            crate::scripts::DictationHistoryMatch {
                id: entry.id.clone(),
                preview: entry.preview.clone(),
                target: entry.target_label_snapshot.clone(),
                timestamp: entry.timestamp.clone(),
                audio_duration_ms: entry.audio_duration_ms,
                subtitle: format!("{} · {duration} · {time}", row.sub.as_deref().unwrap_or(&entry.target_label_snapshot)),
                score: 0,
                matched_field: if row.sub.as_deref()==Some("Matched original words"){crate::dictation::DictationHistorySearchField::RawTranscript}else{crate::dictation::DictationHistorySearchField::Transcript},
                evidence: None,
            },
        ))
    }

    pub(crate) fn one_window_open_dictation_history(&mut self, cx: &mut Context<Self>) {
        let rows = self.one_window_history_rows();
        let learns = &mut self.one_window_host.learns;
        learns.view_head = "What you said".to_string();
        learns.view_rows = rows;
        self.one_window_open_view(
            OwnView::DictationHistory,
            "Dictation History".to_string(),
            Some(EnterTarget::new("history", "Open for review")),
            true,
            cx,
        );
    }

    /// The highlighted saved dictation: its id and preview.
    fn one_window_selected_dictation(&mut self) -> Option<(String, String)> {
        match self.selected_main_list_search_result_owned()? {
            SearchResult::DictationHistory(m) => Some((m.id, m.preview)),
            _ => None,
        }
    }

    /// Enter pastes the transcript at the front app's cursor; picking one for Agent Chat hands
    /// it back to the chat instead.
    pub(super) fn one_window_history_enter(&mut self, cx: &mut Context<Self>) {
        let Some((id, preview)) = self.one_window_selected_dictation() else {
            return;
        };
        if self.active_attachment_portal_kind()
            == Some(crate::ai::context_selector::types::ContextPortalKind::DictationHistory)
        {
            let Some(entry) = crate::dictation::get_history_entry(&id) else {
                return;
            };
            self.one_window_handle(Event::View(sk_one_window::ViewOp::Park), cx);
            self.close_attachment_portal_with_part(
                crate::ai::message_parts::AiContextPart::ResourceUri {
                    uri: entry.resource_uri(),
                    label: format!("Dictation: {preview}"),
                },
                cx,
            );
            return;
        }
        self.restore_saved_take(&id,cx);
    }

    /// ⌘K puts the highlighted transcript in the line and lists what can be done with it.
    pub(super) fn one_window_history_stage(&mut self, cx: &mut Context<Self>) {
        let Some((id, preview)) = self.one_window_selected_dictation() else {
            return;
        };
        self.bind_history_paste(&id,cx);
        self.one_window_handle(
            Event::Stage(sk_one_window::Subject {
                kind: sk_one_window::SubjectKind::Thing,
                label: sk_one_window::contract::words::ACTING_ON.to_string(),
                title: preview,
                icon: "mic".to_string(),
                row_key: Some(format!("history:{id}")),
                ..sk_one_window::Subject::default()
            }),
            cx,
        );
    }

    /// The actions for a staged transcript: the ones root search offers for it.
    pub(crate) fn one_window_history_actions(
        &mut self,
        subject: &sk_one_window::Subject,
    ) -> Option<Event> {
        let id = subject.row_key.as_deref()?.strip_prefix("history:")?;
        let owned = crate::root_unified_result_actions::RootUnifiedActionSubject::Dictation {
            id: id.to_string(),
            preview: subject.title.clone(),
        };
        let actions = crate::root_unified_result_actions::root_unified_actions_for_subject(&owned);
        self.pending_root_unified_actions_subject = Some(owned);
        Some(Event::Actions(
            actions
                .iter()
                .filter(|action| action.is_enabled())
                .filter_map(|action|{let mut row=super::staging::action_row(action);if action.id=="root_dictation_paste"{row.title=format!("Paste into {}",self.history_paste_label(id)?);}Some(row)})
                .collect(),
        ))
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

    /// Deletes a saved dictation; the history view, when open, lists what's left.
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
            self.one_window_notice(
                crate::line_notice::FAILED,
                "Couldn't delete that dictation",
                cx,
            );
            return;
        }
        self.clear_forgotten_draft(entry_id,cx);
        if self.one_window_own_view() == Some(OwnView::DictationHistory) {
            self.one_window_host.learns.view_rows = self.one_window_history_rows();
            self.one_window_republish_own(cx);
        }
        self.one_window_handle(
            Event::Done(Done {
                icon: "check".into(),
                title: "Deleted the dictation".into(),
                ..Done::default()
            }),
            cx,
        );
    }
}
