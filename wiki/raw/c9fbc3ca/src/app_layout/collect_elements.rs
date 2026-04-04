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
                ElementCollectionOutcome::new(elements, total_count)
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
                ..
            } => {
                let rows: Vec<String> = self
                    .file_search_display_indices
                    .iter()
                    .filter_map(|&result_index| self.cached_file_results.get(result_index))
                    .map(|entry| format!("{} — {}", entry.name, entry.path))
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

            AppView::CurrentAppCommandsView {
                filter,
                selected_index,
            } => {
                let rows: Vec<String> = self
                    .cached_current_app_entries
                    .iter()
                    .map(|e| e.name.clone())
                    .collect();
                self.collect_named_rows(
                    "current-app-commands-filter",
                    filter.clone(),
                    "menu-commands",
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

            AppView::FormPrompt { entity, .. } => {
                let form = entity.read(cx);
                let (elements, total_count) =
                    self.collect_form_prompt_elements(form, limit, cx);
                Self::finalize_surface_outcome(
                    "form-prompt",
                    "form-prompt",
                    "panel_only_form_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::TermPrompt { entity, .. } => {
                let term = entity.read(cx);
                let (elements, total_count) =
                    self.collect_term_prompt_elements(term, "term", limit);
                Self::finalize_surface_outcome(
                    "term-prompt",
                    "term-prompt",
                    "panel_only_term_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::EditorPrompt { entity, .. } => {
                let editor = entity.read(cx);
                let (elements, total_count) =
                    self.collect_editor_prompt_elements(editor, "editor", limit);
                Self::finalize_surface_outcome(
                    "editor-prompt",
                    "editor-prompt",
                    "panel_only_editor_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::SelectPrompt { entity, .. } => {
                entity.read(cx).collect_elements(limit).into()
            }

            AppView::PathPrompt { entity, .. } => {
                let path_prompt = entity.read(cx);
                let (elements, total_count) =
                    self.collect_path_prompt_elements(path_prompt, limit);
                Self::finalize_surface_outcome(
                    "path-prompt",
                    "path-prompt",
                    "panel_only_path_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::ChatPrompt { entity, .. } => {
                let chat = entity.read(cx);
                let (elements, total_count) =
                    self.collect_chat_prompt_elements(chat, limit);
                Self::finalize_surface_outcome(
                    "chat-prompt",
                    "chat-prompt",
                    "panel_only_chat_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::EnvPrompt { entity, .. } => {
                let env_prompt = entity.read(cx);
                let (elements, total_count) =
                    self.collect_env_prompt_elements(env_prompt, limit);
                Self::finalize_surface_outcome(
                    "env-prompt",
                    "env-prompt",
                    "panel_only_env_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::DropPrompt { entity, .. } => {
                let drop_prompt = entity.read(cx);
                let (elements, total_count) =
                    self.collect_drop_prompt_elements(drop_prompt, limit);
                Self::finalize_surface_outcome(
                    "drop-prompt",
                    "drop-prompt",
                    "panel_only_drop_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::TemplatePrompt { entity, .. } => {
                let template_prompt = entity.read(cx);
                let (elements, total_count) =
                    self.collect_template_prompt_elements(template_prompt, limit);
                Self::finalize_surface_outcome(
                    "template-prompt",
                    "template-prompt",
                    "panel_only_template_prompt",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::NamingPrompt { entity, .. } => {
                let naming_prompt = entity.read(cx);
                let (elements, total_count) =
                    self.collect_naming_prompt_elements(naming_prompt, limit);
                Self::finalize_surface_outcome(
                    "naming-prompt",
                    "naming-prompt",
                    "panel_only_naming_prompt",
                    limit,
                    elements,
                    total_count,
                )
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

            AppView::ScratchPadView { entity, .. } => {
                let editor = entity.read(cx);
                let (elements, total_count) =
                    self.collect_editor_prompt_elements(editor, "scratch-pad", limit);
                Self::finalize_surface_outcome(
                    "scratch-pad",
                    "scratch-pad",
                    "panel_only_scratch_pad",
                    limit,
                    elements,
                    total_count,
                )
            }

            AppView::QuickTerminalView { entity } => {
                let term = entity.read(cx);
                let (elements, total_count) =
                    self.collect_term_prompt_elements(term, "quick-terminal", limit);
                Self::finalize_surface_outcome(
                    "quick-terminal",
                    "quick-terminal",
                    "panel_only_quick_terminal",
                    limit,
                    elements,
                    total_count,
                )
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

    fn finalize_surface_outcome(
        surface: &str,
        panel_name: &str,
        warning: &str,
        limit: usize,
        elements: Vec<protocol::ElementInfo>,
        total_count: usize,
    ) -> ElementCollectionOutcome {
        if !elements.is_empty() {
            let elements: Vec<protocol::ElementInfo> =
                elements.into_iter().take(limit).collect();
            tracing::info!(
                surface = surface,
                element_count = elements.len(),
                total_count,
                used_panel_fallback = false,
                "Collected semantic elements for inspectable surface"
            );
            return ElementCollectionOutcome::new(elements, total_count);
        }

        let total_count = 1;
        let elements: Vec<protocol::ElementInfo> =
            vec![protocol::ElementInfo::panel(panel_name)]
                .into_iter()
                .take(limit)
                .collect();
        tracing::info!(
            surface = surface,
            element_count = elements.len(),
            total_count,
            used_panel_fallback = true,
            "Collected semantic elements for inspectable surface"
        );
        ElementCollectionOutcome::new(elements, total_count).with_warning(warning)
    }

    fn preview_value(value: &str, max_chars: usize) -> String {
        let char_count = value.chars().count();
        if char_count <= max_chars {
            return value.to_string();
        }

        let mut preview: String = value.chars().take(max_chars).collect();
        preview.push_str("...");
        preview
    }

    fn input_element(
        semantic_name: &str,
        label: impl Into<String>,
        value: Option<String>,
        focused: bool,
        index: Option<usize>,
    ) -> protocol::ElementInfo {
        protocol::ElementInfo {
            semantic_id: protocol::generate_semantic_id_named("input", semantic_name),
            element_type: protocol::ElementType::Input,
            text: Some(label.into()),
            value,
            selected: None,
            focused: Some(focused),
            index,
        }
    }

    fn choice_element(
        index: usize,
        text: String,
        value: String,
        selected: bool,
    ) -> protocol::ElementInfo {
        protocol::ElementInfo {
            semantic_id: protocol::generate_semantic_id("choice", index, value.as_str()),
            element_type: protocol::ElementType::Choice,
            text: Some(text),
            value: Some(value),
            selected: Some(selected),
            focused: None,
            index: Some(index),
        }
    }

    fn collect_form_prompt_elements(
        &self,
        form: &FormPromptState,
        limit: usize,
        cx: &Context<Self>,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = form.fields.len() + 1;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("form-fields", form.fields.len()),
        );

        for (index, (field, entity)) in form.fields.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }

            let field_name = format!("form-{}", field.name);
            let field_label = field
                .label
                .clone()
                .unwrap_or_else(|| field.name.clone());
            let focused = index == form.focused_index;

            let element = match entity {
                crate::form_prompt::FormFieldEntity::TextField(text_field) => {
                    let text_field = text_field.read(cx);
                    Self::input_element(
                        field_name.as_str(),
                        field_label,
                        Some(Self::preview_value(text_field.value(), 240)),
                        focused,
                        Some(index),
                    )
                }
                crate::form_prompt::FormFieldEntity::TextArea(text_area) => {
                    let text_area = text_area.read(cx);
                    Self::input_element(
                        field_name.as_str(),
                        field_label,
                        Some(Self::preview_value(text_area.value(), 240)),
                        focused,
                        Some(index),
                    )
                }
                crate::form_prompt::FormFieldEntity::Checkbox(checkbox) => {
                    let checkbox = checkbox.read(cx);
                    let value = if checkbox.is_checked() {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    };
                    protocol::ElementInfo {
                        semantic_id: protocol::generate_semantic_id_named(
                            "choice",
                            field_name.as_str(),
                        ),
                        element_type: protocol::ElementType::Choice,
                        text: Some(field_label),
                        value: Some(value),
                        selected: Some(checkbox.is_checked()),
                        focused: Some(focused),
                        index: Some(index),
                    }
                }
            };

            elements.push(element);
        }

        (elements, total_count)
    }

    fn collect_term_prompt_elements(
        &self,
        term: &term_prompt::TermPrompt,
        semantic_prefix: &str,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let content = term.terminal.content();
        let visible_lines: Vec<(usize, String)> = content
            .lines_plain()
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some((index, Self::preview_value(trimmed, 240)))
                }
            })
            .collect();

        let total_count = visible_lines.len() + 1;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list(
                format!("{semantic_prefix}-lines").as_str(),
                visible_lines.len(),
            ),
        );

        for (index, (line_index, line)) in visible_lines.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(Self::choice_element(
                index,
                format!("Line {}", line_index + 1),
                line.clone(),
                *line_index == content.cursor_line,
            ));
        }

        (elements, total_count)
    }

    fn collect_editor_prompt_elements(
        &self,
        editor: &crate::editor::EditorPrompt,
        semantic_prefix: &str,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let mut total_count = 1;
        let mut elements = Vec::with_capacity(limit.min(8));

        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                format!("{semantic_prefix}-language").as_str(),
                "Language",
                Some(editor.language().to_string()),
                true,
                Some(0),
            ),
        );

        if let Some(snippet_state) = editor.snippet_state() {
            total_count += snippet_state.current_values.len() + 1;
            Self::push_limited_element(
                &mut elements,
                limit,
                protocol::ElementInfo::list(
                    format!("{semantic_prefix}-tabstops").as_str(),
                    snippet_state.current_values.len(),
                ),
            );

            for (index, value) in snippet_state.current_values.iter().enumerate() {
                if elements.len() >= limit {
                    break;
                }
                elements.push(Self::choice_element(
                    index,
                    format!("Tabstop {}", index + 1),
                    Self::preview_value(value.as_str(), 120),
                    index == snippet_state.current_tabstop_idx,
                ));
            }
        }

        (elements, total_count)
    }

    fn collect_path_prompt_elements(
        &self,
        path_prompt: &PathPrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = path_prompt.filtered_entries.len() + 3;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "path-current-directory",
                "Current Directory",
                Some(path_prompt.current_path.clone()),
                false,
                Some(0),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "path-filter",
                "Filter",
                Some(path_prompt.filter_text.clone()),
                true,
                Some(1),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("path-entries", path_prompt.filtered_entries.len()),
        );

        for (index, entry) in path_prompt.filtered_entries.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            let label = if entry.is_dir {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };
            elements.push(Self::choice_element(
                index,
                label,
                entry.path.clone(),
                index == path_prompt.selected_index,
            ));
        }

        (elements, total_count)
    }

    fn collect_env_prompt_elements(
        &self,
        env_prompt: &EnvPrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let input_text = env_prompt.input_text();
        let display_value = if env_prompt.secret {
            if input_text.is_empty() {
                String::new()
            } else {
                "*".repeat(input_text.chars().count().clamp(1, 8))
            }
        } else {
            Self::preview_value(input_text, 240)
        };

        let mut total_count = 2;
        if env_prompt.exists_in_keyring {
            total_count += 1;
        }

        let mut elements = Vec::with_capacity(limit.min(total_count));
        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "env-key",
                env_prompt.title.clone().unwrap_or_else(|| env_prompt.key.clone()),
                Some(env_prompt.key.clone()),
                false,
                Some(0),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "env-value",
                env_prompt
                    .prompt
                    .clone()
                    .unwrap_or_else(|| "Value".to_string()),
                Some(display_value),
                true,
                Some(1),
            ),
        );

        if env_prompt.exists_in_keyring {
            Self::push_limited_element(
                &mut elements,
                limit,
                protocol::ElementInfo {
                    semantic_id: protocol::generate_semantic_id_named(
                        "choice",
                        "env-keyring-status",
                    ),
                    element_type: protocol::ElementType::Choice,
                    text: Some("Stored Secret".to_string()),
                    value: Some("present".to_string()),
                    selected: Some(true),
                    focused: None,
                    index: Some(2),
                },
            );
        }

        (elements, total_count)
    }

    fn collect_drop_prompt_elements(
        &self,
        drop_prompt: &DropPrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        if drop_prompt.dropped_files.is_empty() {
            return (Vec::new(), 0);
        }

        let total_count = drop_prompt.dropped_files.len() + 1;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("dropped-files", drop_prompt.dropped_files.len()),
        );

        for (index, file) in drop_prompt.dropped_files.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(Self::choice_element(
                index,
                file.name.clone(),
                file.path.clone(),
                false,
            ));
        }

        (elements, total_count)
    }

    fn collect_template_prompt_elements(
        &self,
        template_prompt: &TemplatePrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = template_prompt.inputs.len() + 2;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "template-source",
                "Template",
                Some(Self::preview_value(template_prompt.template.as_str(), 240)),
                false,
                Some(0),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("template-inputs", template_prompt.inputs.len()),
        );

        for (index, input) in template_prompt.inputs.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            let value = template_prompt
                .values
                .get(index)
                .cloned()
                .unwrap_or_default();
            elements.push(Self::input_element(
                format!("template-{}", input.name).as_str(),
                input.label.clone(),
                Some(Self::preview_value(value.as_str(), 180)),
                index == template_prompt.current_input,
                Some(index),
            ));
        }

        (elements, total_count)
    }

    fn collect_naming_prompt_elements(
        &self,
        naming_prompt: &prompts::NamingPrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = 2;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "naming-friendly-name",
                naming_prompt
                    .placeholder
                    .clone()
                    .unwrap_or_else(|| "Name".to_string()),
                Some(Self::preview_value(
                    naming_prompt.friendly_name.as_str(),
                    180,
                )),
                true,
                Some(0),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "naming-filename",
                "Filename",
                Some(Self::preview_value(naming_prompt.filename.as_str(), 180)),
                false,
                Some(1),
            ),
        );

        (elements, total_count)
    }

    fn collect_chat_prompt_elements(
        &self,
        chat_prompt: &prompts::ChatPrompt,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = chat_prompt.messages.len() + 3;
        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "chat-model",
                "Model",
                chat_prompt.model.clone(),
                false,
                Some(0),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            Self::input_element(
                "chat-input",
                chat_prompt
                    .placeholder
                    .clone()
                    .unwrap_or_else(|| "Message".to_string()),
                Some(Self::preview_value(chat_prompt.input.text(), 240)),
                true,
                Some(1),
            ),
        );
        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("chat-messages", chat_prompt.messages.len()),
        );

        for (index, message) in chat_prompt.messages.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            let sender = if message.is_user() { "User" } else { "Assistant" };
            let content = message.get_content();
            let text = if content.is_empty() {
                sender.to_string()
            } else {
                format!("{sender}: {}", Self::preview_value(content, 180))
            };
            elements.push(Self::choice_element(
                index,
                text,
                Self::preview_value(content, 180),
                index + 1 == chat_prompt.messages.len(),
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

        let (elements, total_count) = self.collect_named_rows(
            "filter",
            self.filter_text.clone(),
            "results",
            &row_names,
            self.selected_index,
            limit,
        );

        // Emit JSON snapshot of all collected semantic IDs for agent introspection
        let semantic_ids: Vec<&str> = elements
            .iter()
            .map(|e| e.semantic_id.as_str())
            .collect();
        tracing::debug!(
            event = "collect_script_list_elements",
            total_count,
            returned = elements.len(),
            limit,
            truncated = total_count > elements.len(),
            semantic_ids = ?semantic_ids,
            "ScriptList element collection complete"
        );

        (elements, total_count)
    }
}
