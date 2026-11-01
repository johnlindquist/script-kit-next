#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreateAiPresetFormAction {
    Submit,
}

impl CreateAiPresetFormAction {
    fn success_hud(self, preset_name: &str) -> String {
        match self {
            Self::Submit => format!("Preset '{preset_name}' created"),
        }
    }

    fn failure_message(self, error: impl std::fmt::Display) -> String {
        match self {
            Self::Submit => format!("Failed to create preset: {error}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiPresetSearchEmptyState {
    NoPresetsAvailable,
    NoFilteredMatches,
}

impl AiPresetSearchEmptyState {
    pub(crate) fn from_filter(filter: &str) -> Self {
        if filter.is_empty() {
            Self::NoPresetsAvailable
        } else {
            Self::NoFilteredMatches
        }
    }

    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::NoPresetsAvailable => "No presets available",
            Self::NoFilteredMatches => "No presets match your filter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiPresetModelSelection<'a> {
    ProviderDefault,
    Explicit(&'a str),
}

impl<'a> AiPresetModelSelection<'a> {
    fn from_input(model: &'a str) -> Self {
        if model.trim().is_empty() {
            Self::ProviderDefault
        } else {
            Self::Explicit(model)
        }
    }

    fn as_create_arg(self) -> Option<&'a str> {
        match self {
            Self::ProviderDefault => None,
            Self::Explicit(model) => Some(model),
        }
    }
}

impl ScriptListApp {
    pub(crate) fn ai_preset_search_visible_row_labels(filter: &str) -> Vec<String> {
        let all_presets = crate::ai::presets::load_presets().unwrap_or_default();
        let default_presets: Vec<(&str, &str, &str)> = vec![
            (
                "general",
                "General Assistant",
                "Helpful AI assistant for any task",
            ),
            ("coder", "Code Assistant", "Expert programmer and debugger"),
            (
                "writer",
                "Writing Assistant",
                "Help with writing and editing",
            ),
            (
                "researcher",
                "Research Assistant",
                "Deep analysis and research",
            ),
            (
                "creative",
                "Creative Partner",
                "Brainstorming and creative ideas",
            ),
        ];

        let mut rows: Vec<(String, String, String)> = default_presets
            .iter()
            .map(|(id, name, desc)| ((*id).to_string(), (*name).to_string(), (*desc).to_string()))
            .collect();
        for preset in &all_presets {
            if !default_presets.iter().any(|(did, _, _)| *did == preset.id) {
                rows.push((
                    preset.id.clone(),
                    preset.name.clone(),
                    preset.description.clone(),
                ));
            }
        }

        let filter_lower = filter.to_lowercase();
        rows.into_iter()
            .filter(|(id, name, desc)| {
                filter.is_empty()
                    || name.to_lowercase().contains(&filter_lower)
                    || desc.to_lowercase().contains(&filter_lower)
                    || id.to_lowercase().contains(&filter_lower)
            })
            .map(|(_, name, desc)| {
                if desc.is_empty() {
                    name
                } else {
                    format!("{name} — {desc}")
                }
            })
            .collect()
    }

    /// Render the searchable AI presets list view.
    fn render_search_ai_presets(
        &mut self,
        filter: String,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Load all presets (defaults + user-saved)
        let all_presets = crate::ai::presets::load_presets().unwrap_or_default();
        let default_presets: Vec<(&str, &str, &str)> = vec![
            (
                "general",
                "General Assistant",
                "Helpful AI assistant for any task",
            ),
            ("coder", "Code Assistant", "Expert programmer and debugger"),
            (
                "writer",
                "Writing Assistant",
                "Help with writing and editing",
            ),
            (
                "researcher",
                "Research Assistant",
                "Deep analysis and research",
            ),
            (
                "creative",
                "Creative Partner",
                "Brainstorming and creative ideas",
            ),
        ];

        let mut items: Vec<(String, String, String, bool)> = Vec::new();
        for (id, name, desc) in &default_presets {
            items.push((id.to_string(), name.to_string(), desc.to_string(), true));
        }
        for preset in &all_presets {
            if !default_presets.iter().any(|(did, _, _)| *did == preset.id) {
                items.push((
                    preset.id.clone(),
                    preset.name.clone(),
                    preset.description.clone(),
                    false,
                ));
            }
        }

        let filter_lower = filter.to_lowercase();
        let filtered_items: Vec<_> = if filter.is_empty() {
            items.iter().enumerate().collect()
        } else {
            items
                .iter()
                .enumerate()
                .filter(|(_, (id, name, desc, _))| {
                    name.to_lowercase().contains(&filter_lower)
                        || desc.to_lowercase().contains(&filter_lower)
                        || id.to_lowercase().contains(&filter_lower)
                })
                .collect()
        };

        let count = filtered_items.len();
        let list_colors = ListItemColors::from_theme(&self.theme);
        let entity = cx.entity().downgrade();

        let list_items: Vec<AnyElement> = filtered_items
            .iter()
            .enumerate()
            .map(
                |(display_idx, (_original_idx, (id, name, desc, is_default)))| {
                    let is_selected = display_idx == selected_index;
                    let id_clone = id.clone();
                    let entity_clone = entity.clone();
                    let badge = if *is_default {
                        " (built-in)"
                    } else {
                        " (custom)"
                    };
                    let description = format!("{}{}", desc, badge);

                    div()
                        .id(display_idx)
                        .cursor_pointer()
                        .on_click(move |_event, window, cx| {
                            if let Some(app) = entity_clone.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.select_ai_preset(&id_clone, window, cx);
                                });
                            }
                        })
                        .child(
                            ListItem::new(name.clone(), list_colors)
                                .description_opt(Some(description))
                                .selected(is_selected),
                        )
                        .into_any_element()
                },
            )
            .collect();

        let list_element: AnyElement = if count == 0 {
            crate::components::render_simple_empty_state(
                "ai-presets-empty",
                AiPresetSearchEmptyState::from_filter(&filter).message(),
                "sparkles",
                None,
                &self.theme,
                cx,
            )
        } else {
            crate::components::scrollbar::render_tracked_scroll_column(
                "search-ai-presets-row-stack",
                &self.builtin_row_stack_scroll_handle,
                list_items,
            )
        };
        // Every list leads with a persistent section separator (POLISH.md
        // layout-stability bar; same rule as the main menu's "Results"
        // header, 4d76327b8): the label may swap but the row never appears
        // or disappears, so filtering can't shift the rows below it.
        let list_element = div()
            .flex()
            .flex_col()
            .h_full()
            .min_h(px(0.))
            .child(crate::list_item::render_section_header(
                if filter.trim().is_empty() {
                    "Presets"
                } else {
                    "Results"
                },
                None,
                list_colors,
                true,
            ))
            .child(div().relative().flex_1().min_h(px(0.)).child(list_element))
            .into_any_element();

        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            vec![
                gpui::SharedString::from("↵ Select"),
                gpui::SharedString::from("Esc Back"),
            ],
            None,
        ));

        self.render_generic_filterable_search_surface(
            "search_ai_presets",
            format!("{} presets", count),
            list_element,
            footer,
            cx,
        )
    }

    /// The preset has a domain-specific save, but editing is an ordinary native form.
    fn render_create_ai_preset(&self, entity: Entity<FormPromptState>, cx: &mut Context<Self>) -> AnyElement {
        let palette = crate::components::prompt_text_palette(&self.theme);
        let metrics = crate::components::FormFieldMetrics::from_theme_and_design(&self.theme, self.current_design);
        let fields = entity.clone();
        crate::components::render_inset_prompt_body("preset-body", div().size_full().flex().flex_col().gap(px(metrics.field_stack_gap_px))
            .child(crate::components::prompt_form_intro("Create AI preset", "Save instructions to use in future chats.", palette.primary, palette.help, metrics.field_gap_px))
            .child(div().flex_1().min_h(px(0.)).child(entity)), crate::components::PromptBodyInsets::MainMenu(self.current_design))
            .on_key_down(cx.listener(move |app, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if form_enter_behavior(key, event.keystroke.modifiers.platform, focused_form_field_is_textarea(fields.read(cx))) == FormEnterBehavior::Submit {
                    app.handle_create_ai_preset_key("enter", window, cx); cx.stop_propagation();
                } else if crate::ui_foundation::is_key_escape(key) {
                    app.handle_create_ai_preset_key("escape", window, cx); cx.stop_propagation();
                }
            })).into_any_element()
    }

    /// Select a preset from the search view and apply it in AI chat.
    fn select_ai_preset(&mut self, preset_id: &str, _window: &mut Window, cx: &mut Context<Self>) {
        tracing::info!(
            preset_fingerprint = %crate::ai::reliability::redacted_fingerprint(preset_id),
            action = "select_ai_preset",
            "User selected AI preset from search"
        );
        let trace_id = uuid::Uuid::new_v4().to_string();
        self.open_chat_for_action(
            "select_ai_preset",
            &trace_id,
            DeferredAgentChatAction::ApplyPreset {
                preset_id: preset_id.to_string(),
            },
            cx,
        );
    }

    /// Handle keyboard input for the Create AI Preset form.
    pub(crate) fn handle_create_ai_preset_key(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppView::CreateAiPresetView { entity } = &self.current_view {
            let entity = entity.clone();
            if crate::ui_foundation::is_key_tab(key) {
                entity.update(cx, |form,cx| form.focus_next(window,cx));
            } else if crate::ui_foundation::is_key_enter(key) {
                if self
                    .prompt_completion
                    .as_ref()
                    .is_some_and(|binding| binding.observation().completed)
                {
                    return;
                }
                let form_action = CreateAiPresetFormAction::Submit;
                if entity.update(cx, |form,cx| form.validated_submit_value(Some(window),cx)).is_none() { return; }
                let values: std::collections::HashMap<_,_> = entity.read(cx).named_values(cx).into_iter().collect();
                let name_val = &values["name"];
                let prompt_val = &values["systemPrompt"];
                let model_val = AiPresetModelSelection::from_input(&values["model"]).as_create_arg();
                match crate::ai::presets::create_preset(name_val, prompt_val, model_val) {
                    Ok(preset) => {
                        tracing::info!(
                            preset_fingerprint = %crate::ai::reliability::redacted_fingerprint(
                                &preset.id
                            ),
                            action = "create_preset_success",
                            "AI preset created"
                        );
                        if let Some(binding) = &self.prompt_completion {
                            let outcome = serde_json::to_value(&preset)
                                .map(crate::protocol::SubmitValue::Json)
                                .map(crate::prompt_completion::PromptOutcome::Submitted);
                            match outcome {
                                Ok(outcome) => {
                                    if let Err(error) = binding.try_complete(outcome) {
                                        self.notice_failed(form_action.failure_message(error), cx);
                                        return;
                                    }
                                }
                                Err(error) => {
                                    self.notice_failed(form_action.failure_message(error), cx);
                                    return;
                                }
                            }
                        }
                        if crate::runtime_policy::is_owned_evaluation() {
                            cx.notify();
                        } else {
                            self.one_window_notice(crate::line_notice::INFO, form_action.success_hud(&preset.name), cx);
                            self.go_back_or_close(window, cx);
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            diagnostic_fingerprint = %crate::ai::reliability::redacted_fingerprint(
                                &e.to_string()
                            ),
                            action = "create_preset_failed",
                            "Failed to create preset"
                        );
                        self.notice_failed(form_action.failure_message(e), cx);
                    }
                }
            } else if crate::ui_foundation::is_key_escape(key) {
                if let Some(binding) = &self.prompt_completion {
                    if let Err(error) = binding.try_complete(crate::prompt_completion::PromptOutcome::Cancelled) {
                        self.notice_failed(format!("Preset cancellation failed: {error}"), cx);
                        return;
                    }
                }
                self.go_back_or_close(window, cx);
            }
        }
    }
}

#[cfg(test)]
mod ai_presets_chrome_audit {
    #[test]
    fn ai_presets_search_uses_minimal_chrome_scaffold() {
        let source = include_str!("ai_presets.rs");
        assert!(
            source.contains("render_minimal_list_prompt_scaffold("),
            "ai_presets search should use render_minimal_list_prompt_scaffold"
        );
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert_eq!(
            source.matches(&legacy).count(),
            0,
            "ai_presets should not construct PromptFooter"
        );
    }
}
