use gpui::{Context, Window};

use crate::{AppView, ScriptListApp};

impl ScriptListApp {
    pub(crate) fn menu_syntax_object_selector_owns_main_keyboard(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && self.menu_syntax_object_selector_state.owns_main_list()
    }

    pub(crate) fn set_menu_syntax_object_selector_selection(&mut self, row_id: String) {
        self.menu_syntax_object_selector_state.selected_row_id = Some(row_id);
    }

    pub(crate) fn selected_menu_syntax_object_selector_row_id_from_main_list(
        &mut self,
    ) -> Option<String> {
        let (grouped, flat) = self.get_grouped_results_cached();
        let crate::list_item::GroupedListItem::Item(flat_index) =
            grouped.get(self.selected_index)?
        else {
            return None;
        };
        let Some(crate::scripts::SearchResult::SpineProjection(row)) = flat.get(*flat_index) else {
            return None;
        };
        row.id
            .as_ref()
            .strip_prefix("menu-syntax-object:")
            .map(str::to_string)
    }

    pub(crate) fn accept_menu_syntax_object_selector_row(
        &mut self,
        row_id: &str,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self
            .menu_syntax_object_selector_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return false;
        };
        let Some(selected_index) = snapshot.rows.iter().position(|row| row.id == row_id) else {
            return false;
        };
        let raw_filter_text = self.filter_text.clone();
        let outcome = crate::menu_syntax::apply_object_selector_intent(
            crate::menu_syntax::InlinePickerKeyIntent::Accept,
            &snapshot,
            Some(selected_index),
            &raw_filter_text,
        );
        self.dispatch_menu_syntax_object_selector_outcome(outcome, window, cx);
        true
    }

    fn dispatch_menu_syntax_object_selector_outcome(
        &mut self,
        outcome: crate::menu_syntax::ObjectSelectorIntentOutcome,
        _window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        match outcome {
            crate::menu_syntax::ObjectSelectorIntentOutcome::Ignored
            | crate::menu_syntax::ObjectSelectorIntentOutcome::SelectionChanged { .. } => {}
            crate::menu_syntax::ObjectSelectorIntentOutcome::ReplaceInput { text } => {
                self.filter_text = text.clone();
                self.pending_filter_sync = true;
                self.computed_filter_text = text.clone();
                self.set_menu_syntax_mode_from_filter(&text);
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_replace",
                    cx,
                );
                self.menu_syntax_object_selector_state = Default::default();
                cx.notify();
            }
            crate::menu_syntax::ObjectSelectorIntentOutcome::Close => {
                self.menu_syntax_object_selector_state = Default::default();
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_close",
                    cx,
                );
                cx.notify();
            }
        }
    }

    pub(crate) fn run_menu_syntax_object_selector_state_machine(
        &mut self,
        raw_filter: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu_syntax_form_input_active && self.menu_syntax_capture_form_owns_input() {
            self.menu_syntax_object_selector_state = Default::default();
            self.invalidate_grouped_cache();
            self.reconcile_script_list_after_filter_change(
                "menu_syntax_object_selector_form_input",
                cx,
            );
            cx.notify();
            return;
        }
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        let ctx = crate::menu_syntax::ObjectSelectorContext {
            candidates: self.menu_syntax_object_candidates_for_filter(raw_filter),
        };
        let transition = crate::menu_syntax::plan_object_selector_transition(
            &self.menu_syntax_object_selector_state,
            raw_filter,
            &capture_targets,
            &ctx,
        );
        match transition {
            crate::menu_syntax::ObjectSelectorTransition::NoChange => {}
            crate::menu_syntax::ObjectSelectorTransition::Close => {
                self.menu_syntax_object_selector_state = Default::default();
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_close",
                    cx,
                );
                cx.notify();
            }
            crate::menu_syntax::ObjectSelectorTransition::Open {
                snapshot,
                selected_row_id,
            } => {
                self.menu_syntax_object_selector_state =
                    crate::menu_syntax::MenuSyntaxObjectSelectorState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start: 0,
                    };
                self.menu_syntax_trigger_picker_state = Default::default();
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_open",
                    cx,
                );
                cx.notify();
            }
            crate::menu_syntax::ObjectSelectorTransition::Update {
                snapshot,
                selected_row_id,
            } => {
                let selected_index = selected_row_id
                    .as_deref()
                    .and_then(|id| snapshot.rows.iter().position(|row| row.id == id))
                    .unwrap_or(0);
                let visible_start = crate::menu_syntax::object_selector_visible_start_for_selection(
                    self.menu_syntax_object_selector_state.visible_start,
                    selected_index,
                    snapshot.rows.len(),
                );
                self.menu_syntax_object_selector_state =
                    crate::menu_syntax::MenuSyntaxObjectSelectorState {
                        snapshot: Some(snapshot),
                        selected_row_id,
                        visible_start,
                    };
                self.menu_syntax_trigger_picker_state = Default::default();
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_update",
                    cx,
                );
                cx.notify();
            }
        }
    }

    pub(crate) fn menu_syntax_object_candidates_for_filter(
        &self,
        raw_filter: &str,
    ) -> Vec<crate::menu_syntax::ObjectSelectorCandidate> {
        let capture_targets =
            crate::menu_syntax::registered_capture_targets_from_scripts(&self.scripts);
        let Some(selector) = crate::menu_syntax::capture::active_object_selector_for_input(
            raw_filter,
            &capture_targets,
        ) else {
            return Vec::new();
        };
        let query = selector.query.trim();
        match selector.kind {
            crate::menu_syntax::CaptureObjectKind::Note => {
                crate::notes::search_root_notes_meta_direct(
                    query,
                    crate::notes::RootNotesSectionOptions {
                        enabled: true,
                        max_results: 10,
                        min_query_chars: 0,
                        search_content: true,
                    },
                )
                .into_iter()
                .map(|hit| crate::menu_syntax::ObjectSelectorCandidate {
                    kind: crate::menu_syntax::CaptureObjectKind::Note,
                    id: hit.id.to_string(),
                    label: if hit.title.trim().is_empty() {
                        "Untitled Note".to_string()
                    } else {
                        hit.title
                    },
                    subtitle: format!(
                        "Updated {} - {} chars",
                        crate::formatting::format_relative_time_short_dt(hit.updated_at),
                        hit.char_count
                    ),
                })
                .collect()
            }
            kind => crate::menu_syntax::search_root_object_candidates_direct(kind, query, 10),
        }
    }

    pub(crate) fn apply_menu_syntax_object_selector_intent(
        &mut self,
        intent: crate::menu_syntax::InlinePickerKeyIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self
            .menu_syntax_object_selector_state
            .snapshot
            .as_ref()
            .cloned()
        else {
            return false;
        };
        let selected_row_id = self
            .selected_menu_syntax_object_selector_row_id_from_main_list()
            .or_else(|| {
                self.menu_syntax_object_selector_state
                    .selected_row_id
                    .clone()
            });
        let selected_index = selected_row_id
            .as_deref()
            .and_then(|id| snapshot.rows.iter().position(|row| row.id == id));
        let raw_filter_text = self.filter_text.clone();
        let outcome = crate::menu_syntax::apply_object_selector_intent(
            intent,
            &snapshot,
            selected_index,
            &raw_filter_text,
        );
        match outcome {
            crate::menu_syntax::ObjectSelectorIntentOutcome::SelectionChanged { new_index } => {
                let next_row_id = snapshot.rows.get(new_index).map(|row| row.id.clone());
                self.menu_syntax_object_selector_state.visible_start =
                    crate::menu_syntax::object_selector_visible_start_for_selection(
                        self.menu_syntax_object_selector_state.visible_start,
                        new_index,
                        snapshot.rows.len(),
                    );
                self.menu_syntax_object_selector_state.selected_row_id = next_row_id;
                self.invalidate_grouped_cache();
                self.reconcile_script_list_after_filter_change(
                    "menu_syntax_object_selector_selection",
                    cx,
                );
                cx.notify();
                true
            }
            crate::menu_syntax::ObjectSelectorIntentOutcome::Ignored => false,
            other => {
                self.dispatch_menu_syntax_object_selector_outcome(other, Some(window), cx);
                true
            }
        }
    }
}
