// Element collection for getElements protocol support.
// Returns a bounded list of visible UI elements with semantic IDs.

impl ScriptListApp {
    fn collect_visible_elements(
        &self,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        match &self.current_view {
            AppView::ScriptList => self.collect_script_list_elements(limit),

            AppView::ArgPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ),

            AppView::MiniPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ),

            AppView::MicroPrompt { choices, .. } => self.collect_choice_view_elements(
                "filter",
                self.arg_input.text().to_string(),
                choices,
                self.arg_selected_index,
                limit,
            ),

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
                )
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
                )
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
                )
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
                )
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
                )
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
                )
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
                )
            }

            AppView::ThemeChooserView { filter, .. } => {
                let elements = vec![
                    protocol::ElementInfo::input(
                        "theme-filter",
                        Some(filter.as_str()),
                        true,
                    ),
                    protocol::ElementInfo::panel("theme-chooser"),
                ];
                (elements, 2)
            }

            AppView::ActionsDialog => {
                let elements = vec![protocol::ElementInfo::panel("actions-dialog")];
                (elements, 1)
            }

            AppView::DivPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("div-prompt")];
                (elements, 1)
            }

            AppView::FormPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("form-prompt")];
                (elements, 1)
            }

            AppView::TermPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("term-prompt")];
                (elements, 1)
            }

            AppView::EditorPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("editor-prompt")];
                (elements, 1)
            }

            AppView::SelectPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("select-prompt")];
                (elements, 1)
            }

            AppView::PathPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("path-prompt")];
                (elements, 1)
            }

            AppView::ChatPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("chat-prompt")];
                (elements, 1)
            }

            AppView::EnvPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("env-prompt")];
                (elements, 1)
            }

            AppView::DropPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("drop-prompt")];
                (elements, 1)
            }

            AppView::TemplatePrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("template-prompt")];
                (elements, 1)
            }

            AppView::NamingPrompt { .. } => {
                let elements = vec![protocol::ElementInfo::panel("naming-prompt")];
                (elements, 1)
            }

            AppView::CreationFeedback { .. } => {
                let elements =
                    vec![protocol::ElementInfo::panel("creation-feedback")];
                (elements, 1)
            }

            AppView::WebcamView { .. } => {
                let elements = vec![protocol::ElementInfo::panel("webcam")];
                (elements, 1)
            }

            AppView::ScratchPadView { .. } => {
                let elements = vec![protocol::ElementInfo::panel("scratch-pad")];
                (elements, 1)
            }

            AppView::QuickTerminalView { .. } => {
                let elements = vec![protocol::ElementInfo::panel("quick-terminal")];
                (elements, 1)
            }

            _ => {
                let elements = vec![protocol::ElementInfo::panel("current-view")];
                (elements, 1)
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
        elements.push(protocol::ElementInfo::input(
            input_name,
            Some(input_value.as_str()),
            self.focused_input != FocusedInput::None,
        ));
        elements.push(protocol::ElementInfo::list("choices", filtered.len()));

        for (index, choice) in filtered.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(protocol::ElementInfo::choice(
                index,
                &choice.name,
                &choice.value,
                index == selected_index,
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
        elements.push(protocol::ElementInfo::input(
            input_name,
            Some(input_value.as_str()),
            self.focused_input != FocusedInput::None,
        ));
        elements.push(protocol::ElementInfo::list(list_name, rows.len()));

        for (index, row) in rows.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(protocol::ElementInfo::choice(
                index,
                row,
                row,
                index == selected_index,
            ));
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

        self.collect_named_rows(
            "filter",
            self.filter_text.clone(),
            "results",
            &row_names,
            self.selected_index,
            limit,
        )
    }
}
