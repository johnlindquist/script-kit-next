// Element collection for getElements protocol support.
// Returns a bounded list of visible UI elements with semantic IDs.

/// Outcome of collecting visible UI elements, carrying receipt metadata
/// for the `elementsResult` protocol response.
#[derive(Debug, Clone)]
pub(crate) struct ElementCollectionOutcome {
    pub elements: Vec<protocol::ElementInfo>,
    pub total_count: usize,
    pub warnings: Vec<String>,
}

impl ElementCollectionOutcome {
    pub fn new(elements: Vec<protocol::ElementInfo>, total_count: usize) -> Self {
        Self {
            elements,
            total_count,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn focused_semantic_id(&self) -> Option<String> {
        self.elements
            .iter()
            .find(|element| element.focused == Some(true))
            .map(|element| element.semantic_id.clone())
    }

    pub fn selected_semantic_id(&self) -> Option<String> {
        self.elements
            .iter()
            .find(|element| element.selected == Some(true))
            .map(|element| element.semantic_id.clone())
    }
}

impl From<(Vec<protocol::ElementInfo>, usize)> for ElementCollectionOutcome {
    fn from((elements, total_count): (Vec<protocol::ElementInfo>, usize)) -> Self {
        Self::new(elements, total_count)
    }
}

impl ScriptListApp {
    /// Push an element into the vec only if it hasn't reached the limit.
    /// Returns true if the element was added, false if capped.
    #[inline]
    fn push_limited_element(
        elements: &mut Vec<protocol::ElementInfo>,
        limit: usize,
        element: protocol::ElementInfo,
    ) -> bool {
        if elements.len() >= limit {
            return false;
        }
        elements.push(element);
        true
    }

    /// Build an ElementInfo for a Choice, preferring its stable key for the semantic ID.
    #[inline]
    fn keyed_choice_element(
        display_index: usize,
        choice: &Choice,
        selected: bool,
    ) -> protocol::ElementInfo {
        protocol::ElementInfo {
            semantic_id: choice.generate_id(display_index),
            element_type: protocol::ElementType::Choice,
            text: Some(choice.name.clone()),
            value: Some(choice.value.clone()),
            selected: Some(selected),
            focused: None,
            index: Some(display_index),
        }
    }

    pub(crate) fn collect_visible_elements(
        &self,
        limit: usize,
        cx: &Context<Self>,
    ) -> ElementCollectionOutcome {
        match &self.current_view {
            AppView::ScriptList => {
                let (elements, total_count) = self.collect_script_list_elements(limit);
                let mut outcome = ElementCollectionOutcome::new(elements, total_count);
                let context_strip_present = outcome
                    .elements
                    .iter()
                    .any(|element| element.semantic_id == "panel:context-strip");
                // Structured warning if context strip elements were truncated by the limit
                if !context_strip_present && total_count > outcome.elements.len() {
                    tracing::info!(
                        event = "context_strip_truncated",
                        limit,
                        total_count,
                        "Context strip omitted from elements due to limit"
                    );
                    outcome = outcome.with_warning("context_strip_truncated_by_limit");
                }
                outcome
            }

            AppView::ArgPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ).into(),

            AppView::MiniPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ).into(),

            AppView::MicroPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ).into(),

            AppView::ClipboardHistoryView {
                filter,
                selected_index,
            } => {
                let rows: Vec<String> = self
                    .cached_clipboard_entries
                    .iter()
                    .map(|entry| entry.text_preview.clone())
                    .collect();
                self.collect_named_rows(
                    "clipboard-filter",
                    filter.clone(),
                    "clipboard-history",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::AppLauncherView {
                filter,
                selected_index,
            } => {
                let rows: Vec<String> =
                    self.apps.iter().map(|app| app.name.clone()).collect();
                self.collect_named_rows(
                    "app-filter",
                    filter.clone(),
                    "apps",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::WindowSwitcherView {
                filter,
                selected_index,
            } => {
                let rows: Vec<String> = self
                    .cached_windows
                    .iter()
                    .map(|w| format!("{} — {}", w.app, w.title))
                    .collect();
                self.collect_named_rows(
                    "window-filter",
                    filter.clone(),
                    "windows",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::FileSearchView {
                ref query,
                selected_index,
            } => {
                let rows: Vec<String> = self
                    .cached_file_results
                    .iter()
                    .map(|entry| entry.name.clone())
                    .collect();
                self.collect_named_rows(
                    "file-search-input",
                    query.clone(),
                    "file-results",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::ProcessManagerView {
                filter,
                selected_index,
            } => {
                let rows: Vec<String> = self
                    .cached_processes
                    .iter()
                    .map(|p| p.script_path.clone())
                    .collect();
                self.collect_named_rows(
                    "process-filter",
                    filter.clone(),
                    "processes",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::EmojiPickerView {
                ref filter,
                selected_index,
                selected_category,
            } => {
                let rows: Vec<String> = crate::emoji::search_emojis(filter.as_str())
                    .into_iter()
                    .filter(|emoji| {
                        selected_category
                            .map(|category| emoji.category == category)
                            .unwrap_or(true)
                    })
                    .map(|emoji| emoji.name.to_string())
                    .collect();
                self.collect_named_rows(
                    "emoji-filter",
                    filter.clone(),
                    "emoji-results",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::BrowseKitsView {
                query,
                selected_index,
                results,
            } => {
                let rows: Vec<String> =
                    results.iter().map(|r| r.full_name.clone()).collect();
                self.collect_named_rows(
                    "kit-search",
                    query.clone(),
                    "kit-results",
                    &rows,
                    *selected_index,
                    limit,
                ).into()
            }

            AppView::ThemeChooserView { filter, .. } => {
                let total_count = 2;
                let elements: Vec<protocol::ElementInfo> = vec![
                    protocol::ElementInfo::input(
                        "theme-filter",
                        Some(filter.as_str()),
                        true,
                    ),
                    protocol::ElementInfo::panel("theme-chooser"),
                ]
                .into_iter()
                .take(limit)
                .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_theme_chooser")
            }

            AppView::ActionsDialog => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("actions-dialog")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_actions_dialog")
            }

            AppView::DivPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("div-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_div_prompt")
            }

            AppView::FormPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("form-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_form_prompt")
            }

            AppView::TermPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("term-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_term_prompt")
            }

            AppView::EditorPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("editor-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_editor_prompt")
            }

            AppView::SelectPrompt { entity, .. } => {
                entity.read(cx).collect_elements(limit).into()
            }

            AppView::PathPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("path-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_path_prompt")
            }

            AppView::ChatPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("chat-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_chat_prompt")
            }

            AppView::EnvPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("env-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_env_prompt")
            }

            AppView::DropPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("drop-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_drop_prompt")
            }

            AppView::TemplatePrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("template-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_template_prompt")
            }

            AppView::NamingPrompt { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("naming-prompt")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_naming_prompt")
            }

            AppView::CreationFeedback { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("creation-feedback")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_creation_feedback")
            }

            AppView::WebcamView { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("webcam")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_webcam")
            }

            AppView::ScratchPadView { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("scratch-pad")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_scratch_pad")
            }

            AppView::QuickTerminalView { .. } => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("quick-terminal")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("panel_only_quick_terminal")
            }

            _ => {
                let total_count = 1;
                let elements: Vec<protocol::ElementInfo> =
                    vec![protocol::ElementInfo::panel("current-view")]
                        .into_iter()
                        .take(limit)
                        .collect();
                ElementCollectionOutcome::new(elements, total_count)
                    .with_warning("collector_used_current_view_fallback")
            }
        }
    }

    fn collect_choice_view_elements(
        &self,
        input_name: &str,
        input_value: String,
        choices: &[Choice],
        selected_index: usize,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let filtered = self.get_filtered_arg_choices(choices);
        let total_count = filtered.len() + 2;

        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::input(
                input_name,
                Some(input_value.as_str()),
                self.focused_input != FocusedInput::None,
            ),
        );

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("choices", filtered.len()),
        );

        for (display_index, choice) in filtered.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(Self::keyed_choice_element(
                display_index,
                choice,
                display_index == selected_index,
            ));
        }

        (elements, total_count)
    }

    fn collect_named_rows(
        &self,
        input_name: &str,
        input_value: String,
        list_name: &str,
        rows: &[String],
        selected_index: usize,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = rows.len() + 2;

        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::input(
                input_name,
                Some(input_value.as_str()),
                self.focused_input != FocusedInput::None,
            ),
        );

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list(list_name, rows.len()),
        );

        for (index, row) in rows.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(protocol::ElementInfo {
                semantic_id: protocol::generate_semantic_id("choice", index, row),
                element_type: protocol::ElementType::Choice,
                text: Some(row.clone()),
                value: Some(row.clone()),
                selected: Some(index == selected_index),
                focused: None,
                index: Some(index),
            });
        }

        (elements, total_count)
    }

    fn collect_script_list_elements(
        &self,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let row_names: Vec<String> = self
            .filtered_results()
            .into_iter()
            .map(|result| match result {
                scripts::SearchResult::Script(m) => m.script.name.clone(),
                scripts::SearchResult::Scriptlet(m) => m.scriptlet.name.clone(),
                scripts::SearchResult::BuiltIn(m) => m.entry.name.clone(),
                scripts::SearchResult::App(m) => m.app.name.clone(),
                scripts::SearchResult::Window(m) => m.window.title.clone(),
                scripts::SearchResult::Agent(m) => m.agent.name.clone(),
                scripts::SearchResult::Fallback(m) => m.fallback.name().to_string(),
            })
            .collect();

        let (mut elements, base_total) = self.collect_named_rows(
            "filter",
            self.filter_text.clone(),
            "results",
            &row_names,
            self.selected_index,
            limit,
        );

        // Append context strip elements
        let strip_labels = ["Current Context", "Selection", "Browser URL", "Focused Window"];
        let default_parts = Self::default_main_window_context_parts();
        // +1 for the panel, +4 chips, +1 "Ask AI with Context" button
        let strip_element_count = 1 + strip_labels.len() + 1;
        let total_count = base_total + strip_element_count;

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::panel("context-strip"),
        );

        for (i, (label, part)) in strip_labels.iter().zip(default_parts.iter()).enumerate() {
            let is_selected = self.main_window_context_parts.contains(part);
            let mut btn = protocol::ElementInfo::button(i, label);
            btn.selected = Some(is_selected);
            Self::push_limited_element(&mut elements, limit, btn);
        }

        let mut ai_btn = protocol::ElementInfo::button(4, "Ask AI with Context");
        ai_btn.selected = Some(!self.main_window_context_parts.is_empty());
        Self::push_limited_element(&mut elements, limit, ai_btn);

        // Emit JSON snapshot of all collected semantic IDs for agent introspection
        let semantic_ids: Vec<&str> = elements
            .iter()
            .map(|e| e.semantic_id.as_str())
            .collect();
        let active_context_labels: Vec<&str> = self
            .main_window_context_parts
            .iter()
            .map(|p| p.label())
            .collect();
        let context_strip_present = elements
            .iter()
            .any(|element| element.semantic_id == "panel:context-strip");
        tracing::debug!(
            event = "collect_script_list_elements",
            total_count,
            returned = elements.len(),
            limit,
            truncated = total_count > elements.len(),
            context_strip_present,
            active_context_count = self.main_window_context_parts.len(),
            active_context_labels = ?active_context_labels,
            semantic_ids = ?semantic_ids,
            "ScriptList element collection complete"
        );

        (elements, total_count)
    }
}
