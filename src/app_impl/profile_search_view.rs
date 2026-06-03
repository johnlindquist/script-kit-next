use super::*;

impl ScriptListApp {
    pub(crate) fn open_profile_search(&mut self, cx: &mut Context<Self>) {
        tracing::info!(
            target: "script_kit::spine",
            event = "profile_search_open",
            "Opening Profile Search"
        );
        self.filter_text.clear();
        self.pending_filter_sync = true;
        self.pending_placeholder = Some("Search profiles...".to_string());
        self.current_view = AppView::ProfileSearchView {
            filter: String::new(),
            selected_index: 0,
        };
        self.rekey_main_automation_surface_from_current_view();
        self.hovered_index = None;
        self.opened_from_main_menu = true;
        self.pending_focus = Some(FocusTarget::MainFilter);
        self.focused_input = FocusedInput::MainFilter;
        cx.notify();
    }

    pub(crate) fn profile_search_results_for_filter(
        &self,
        filter: &str,
    ) -> Vec<crate::profile_search::ProfileSearchResult> {
        let prefs = crate::config::load_user_preferences();
        let ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
        crate::profile_search::profile_search_results(&prefs.ai, &ctx, filter)
    }

    pub(crate) fn profile_search_visible_len(&self, filter: &str) -> usize {
        self.profile_search_results_for_filter(filter).len()
    }

    pub(crate) fn selected_profile_search_result_owned(
        &self,
    ) -> Option<crate::profile_search::ProfileSearchResult> {
        let AppView::ProfileSearchView {
            filter,
            selected_index,
        } = &self.current_view
        else {
            return None;
        };
        self.profile_search_results_for_filter(filter)
            .get(*selected_index)
            .cloned()
    }

    pub(crate) fn move_profile_search_selection(&mut self, is_up: bool, cx: &mut Context<Self>) {
        let (filter, current_index) = match &self.current_view {
            AppView::ProfileSearchView {
                filter,
                selected_index,
            } => (filter.clone(), *selected_index),
            _ => return,
        };
        let filtered_len = self.profile_search_visible_len(&filter);
        let mut next_index = current_index;
        if filtered_len == 0 {
            next_index = 0;
        } else {
            if next_index >= filtered_len {
                next_index = filtered_len - 1;
            }
            if is_up && next_index > 0 {
                next_index -= 1;
            } else if !is_up && next_index + 1 < filtered_len {
                next_index += 1;
            }
        }
        if let AppView::ProfileSearchView { selected_index, .. } = &mut self.current_view {
            *selected_index = next_index;
        }
        self.list_scroll_handle
            .scroll_to_item(next_index, ScrollStrategy::Nearest);
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        cx.notify();
    }

    pub(crate) fn select_profile_search_result(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(result) = self.selected_profile_search_result_owned() else {
            return false;
        };
        let persisted = crate::profile_search::persist_profile_search_selection(&result.profile.id);
        if persisted {
            self.refresh_agent_model_footer_labels();
            self.reset_to_script_list(cx);
        }
        cx.notify();
        persisted
    }
}
