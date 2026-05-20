use super::*;

impl ScriptListApp {
    pub(crate) fn update_main_input_tab_navigation_for_menu_syntax_form(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let handler_form_owns_input = self.menu_syntax_capture_form_owns_input();
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_tab_navigation(handler_form_owns_input, window, cx);
            state.set_tab_navigation_space_as_tab(handler_form_owns_input, window, cx);
        });
    }

    fn menu_syntax_form_signature(form: &crate::menu_syntax::MenuSyntaxFormSnapshot) -> String {
        let mut signature = format!("{}:", form.target);
        for field in &form.fields {
            signature.push_str(&field.id);
            signature.push('|');
        }
        signature
    }

    pub(crate) fn sync_menu_syntax_form_inputs_from_filter(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_main_input_tab_navigation_for_menu_syntax_form(window, cx);
        if !self.menu_syntax_capture_form_owns_input() {
            self.clear_menu_syntax_form_inputs();
            return;
        }

        let Some(snapshot) = self.menu_syntax_main_hint_snapshot(&self.filter_text, false) else {
            self.clear_menu_syntax_form_inputs();
            return;
        };
        let Some(form) = snapshot.form else {
            self.clear_menu_syntax_form_inputs();
            return;
        };
        self.ensure_menu_syntax_form_inputs(&form, window, cx);
    }

    fn clear_menu_syntax_form_inputs(&mut self) {
        self.menu_syntax_form_signature = None;
        self.menu_syntax_form_inputs.clear();
        self.menu_syntax_form_input_subscriptions.clear();
        self.menu_syntax_form_syncing_from_input = false;
    }

    fn menu_syntax_form_input_for(
        &self,
        field_id: &str,
    ) -> Option<Entity<gpui_component::input::InputState>> {
        self.menu_syntax_form_inputs
            .iter()
            .find_map(|(id, input)| (id == field_id).then(|| input.clone()))
    }

    fn new_menu_syntax_form_input(
        placeholder: String,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<gpui_component::input::InputState> {
        cx.new(|cx| {
            gpui_component::input::InputState::new(window, cx)
                .tab_navigation(true)
                .placeholder(placeholder)
                .default_value(value)
        })
    }

    fn ensure_menu_syntax_form_inputs(
        &mut self,
        form: &crate::menu_syntax::MenuSyntaxFormSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let signature = Self::menu_syntax_form_signature(form);
        if self.menu_syntax_form_signature.as_deref() != Some(signature.as_str()) {
            self.menu_syntax_form_signature = Some(signature);
            self.menu_syntax_form_inputs.clear();
            self.menu_syntax_form_input_subscriptions.clear();
            for field in &form.fields {
                let input = Self::new_menu_syntax_form_input(
                    field.placeholder.clone(),
                    field.value.clone(),
                    window,
                    cx,
                );
                let field_id = field.id.clone();
                let subscription = cx.subscribe_in(&input, window, {
                    let field_id = field_id.clone();
                    move |this,
                          input,
                          event: &gpui_component::input::InputEvent,
                          window,
                          cx| {
                        match event {
                            gpui_component::input::InputEvent::Change => {
                                if this.menu_syntax_form_syncing_from_input {
                                    return;
                                }
                                let value = input.read(cx).value().to_string();
                                this.menu_syntax_form_syncing_from_input = true;
                                let _ = this.update_menu_syntax_form_field(
                                    Some(&field_id),
                                    value,
                                    window,
                                    cx,
                                );
                                this.menu_syntax_form_syncing_from_input = false;
                            }
                            gpui_component::input::InputEvent::PressTab { secondary } => {
                                if *secondary {
                                    this.focus_previous_menu_syntax_form_field(window, cx);
                                } else {
                                    this.focus_next_menu_syntax_form_field(window, cx);
                                }
                            }
                            _ => {}
                        }
                    }
                });
                self.menu_syntax_form_inputs.push((field_id, input));
                self.menu_syntax_form_input_subscriptions.push(subscription);
            }
        }

        for field in &form.fields {
            let Some(input) = self.menu_syntax_form_input_for(&field.id) else {
                continue;
            };
            let current = input.read(cx).value().to_string();
            if current == field.value {
                continue;
            }
            let focused = input.read(cx).focus_handle(cx).is_focused(window);
            if focused {
                continue;
            }
            self.menu_syntax_form_syncing_from_input = true;
            input.update(cx, |state, cx| {
                state.set_value(field.value.clone(), window, cx);
                state.set_placeholder(field.placeholder.clone(), window, cx);
            });
            self.menu_syntax_form_syncing_from_input = false;
        }
    }

    fn actual_menu_syntax_form_focused_index(
        &self,
        window: &Window,
        cx: &App,
    ) -> Option<usize> {
        self.menu_syntax_form_inputs
            .iter()
            .enumerate()
            .find_map(|(index, (_, input))| {
                input
                    .read(cx)
                    .focus_handle(cx)
                    .is_focused(window)
                    .then_some(index)
            })
    }

    fn focus_menu_syntax_form_input_at(
        &mut self,
        index: usize,
        form: &crate::menu_syntax::MenuSyntaxFormSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((field_id, input)) = self.menu_syntax_form_inputs.get(index).cloned() else {
            return;
        };
        self.menu_syntax_form_input_active = true;
        self.menu_syntax_form_focused_index = index;
        if let Some(field) = form.fields.get(index) {
            self.menu_syntax_form_draft_field_id = Some(field.id.clone());
            self.menu_syntax_form_draft_value = field.value.clone();
        } else {
            self.menu_syntax_form_draft_field_id = Some(field_id);
            self.menu_syntax_form_draft_value.clear();
        }
        input.update(cx, |state, cx| state.focus(window, cx));
        cx.notify();
    }

    fn focus_menu_syntax_main_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.menu_syntax_form_input_active = false;
        self.menu_syntax_form_draft_field_id = None;
        self.menu_syntax_form_draft_value.clear();
        self.gpui_input_state
            .update(cx, |state, cx| state.focus(window, cx));
        cx.notify();
    }

    pub(crate) fn menu_syntax_capture_form_owns_input(&self) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && self
                .menu_syntax_mode
                .capture_composer_owns_input_for(&self.filter_text)
    }

    pub(crate) fn focus_next_menu_syntax_form_field(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_menu_syntax_form_focus(1, window, cx);
    }

    pub(crate) fn focus_previous_menu_syntax_form_field(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_menu_syntax_form_focus(-1, window, cx);
    }

    pub(crate) fn update_menu_syntax_form_field(
        &mut self,
        field_id: Option<&str>,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.menu_syntax_capture_form_owns_input() {
            return false;
        }
        let Some(snapshot) = self.menu_syntax_main_hint_snapshot(&self.filter_text, false) else {
            return false;
        };
        let Some(form) = snapshot.form else {
            return false;
        };
        let Some(invocation) = self.menu_syntax_capture_form_invocation(&self.filter_text) else {
            return false;
        };
        let resolved_field_id = field_id
            .map(str::to_string)
            .or_else(|| {
                form.fields
                    .get(form.focused_index.min(form.fields.len().saturating_sub(1)))
                    .map(|field| field.id.clone())
            })
            .unwrap_or_default();
        if resolved_field_id.is_empty() {
            return false;
        }
        if self.menu_syntax_form_input_active
            && self
                .menu_syntax_form_draft_field_id
                .as_deref()
                .is_some_and(|id| id == resolved_field_id)
        {
            self.menu_syntax_form_draft_value = value.clone();
        }
        let Some(next_text) = crate::menu_syntax::apply_capture_form_field_edit(
            &invocation,
            &resolved_field_id,
            &value,
        ) else {
            return false;
        };
        self.set_filter_text_immediate(next_text, window, cx);
        tracing::info!(
            target: "script_kit::menu_syntax_form",
            event = "menu_syntax_form_field_updated",
            field_id = %resolved_field_id,
            sync_source = "formField",
        );
        true
    }

    pub(crate) fn handle_menu_syntax_form_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.menu_syntax_form_input_active || !self.menu_syntax_capture_form_owns_input() {
            return false;
        }
        if modifiers.platform || modifiers.alt || modifiers.control {
            return false;
        }
        let Some(snapshot) = self.menu_syntax_main_hint_snapshot(&self.filter_text, false) else {
            return false;
        };
        let Some(form) = snapshot.form else {
            return false;
        };
        let Some(field) = form
            .fields
            .get(form.focused_index.min(form.fields.len().saturating_sub(1)))
        else {
            return false;
        };
        let field_id = field.id.clone();
        let mut value = self
            .menu_syntax_form_draft_field_id
            .as_deref()
            .filter(|id| *id == field_id)
            .map(|_| self.menu_syntax_form_draft_value.clone())
            .unwrap_or_else(|| field.value.clone());

        let key_lower = key.to_ascii_lowercase();
        if key_lower == "escape" {
            self.menu_syntax_form_input_active = false;
            self.menu_syntax_form_draft_field_id = None;
            self.menu_syntax_form_draft_value.clear();
            cx.notify();
            return true;
        }

        if key_lower == "backspace" {
            if value.pop().is_none() {
                return true;
            }
        } else if key_lower == "space" {
            value.push(' ');
        } else if let Some(ch) = key_char.filter(|ch| ch.chars().count() == 1) {
            value.push_str(ch);
        } else {
            return false;
        }

        self.menu_syntax_form_draft_field_id = Some(field_id.clone());
        self.menu_syntax_form_draft_value = value.clone();
        self.update_menu_syntax_form_field(Some(&field_id), value, window, cx)
    }

    fn move_menu_syntax_form_focus(
        &mut self,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = self.menu_syntax_main_hint_snapshot(&self.filter_text, false) else {
            return;
        };
        let Some(form) = snapshot.form else {
            return;
        };
        self.ensure_menu_syntax_form_inputs(&form, window, cx);
        let field_count = form.fields.len();
        if field_count == 0 {
            self.menu_syntax_form_focused_index = 0;
            self.menu_syntax_form_input_active = false;
            return;
        }
        let current = self
            .actual_menu_syntax_form_focused_index(window, cx)
            .or_else(|| self.menu_syntax_form_input_active.then_some(
                self.menu_syntax_form_focused_index.min(field_count - 1),
            ));
        let next = match (current, delta < 0) {
            (None, false) => Some(0),
            (None, true) => Some(field_count - 1),
            (Some(0), true) => None,
            (Some(index), true) => Some(index - 1),
            (Some(index), false) if index + 1 < field_count => Some(index + 1),
            (Some(_), false) => None,
        };
        if let Some(next_index) = next {
            self.focus_menu_syntax_form_input_at(next_index, &form, window, cx);
        } else {
            self.focus_menu_syntax_main_input(window, cx);
        }
        tracing::info!(
            target: "script_kit::menu_syntax_form",
            event = "menu_syntax_form_focus_changed",
            target = %form.target,
            focused_index = self.menu_syntax_form_focused_index,
            field_count,
        );
        cx.notify();
    }

    fn sync_menu_syntax_form_draft_from_form(&mut self, form: &crate::menu_syntax::MenuSyntaxFormSnapshot) {
        let Some(field) = form
            .fields
            .get(self.menu_syntax_form_focused_index.min(form.fields.len().saturating_sub(1)))
        else {
            self.menu_syntax_form_draft_field_id = None;
            self.menu_syntax_form_draft_value.clear();
            return;
        };
        self.menu_syntax_form_draft_field_id = Some(field.id.clone());
        self.menu_syntax_form_draft_value = field.value.clone();
    }

    pub(crate) fn scroll_menu_syntax_main_hint(&mut self, direction: f32) {
        let line_delta = gpui::px(crate::scrolling::free_scroll::FREE_SCROLL_LINE_DELTA_PX);
        let current = self.menu_syntax_main_hint_scroll_handle.offset();
        let max = self.menu_syntax_main_hint_scroll_handle.max_offset();
        let next_y = (current.y - (line_delta * direction)).clamp(-max.y, gpui::px(0.0));
        self.menu_syntax_main_hint_scroll_handle
            .set_offset(gpui::point(current.x, next_y));
    }

    pub(crate) fn menu_syntax_main_hint_snapshot(
        &self,
        raw_filter_text: &str,
        advanced_query_results_empty: bool,
    ) -> Option<crate::menu_syntax::MenuSyntaxMainHintSnapshot> {
        let menu_syntax_ai_proposal = self
            .pending_menu_syntax_ai_proposal
            .as_ref()
            .filter(|pending| pending.is_current_for(raw_filter_text))
            .map(|pending| &pending.proposal);
        let mut snapshot = crate::menu_syntax::build_menu_syntax_main_hint(
            crate::menu_syntax::MenuSyntaxMainHintContext {
                raw_filter_text,
                mode: &self.menu_syntax_mode,
                popup_snapshot: self.menu_syntax_trigger_popup_state.snapshot.as_ref(),
                popup_selected_row_id: self
                    .menu_syntax_trigger_popup_state
                    .selected_row_id
                    .as_deref(),
                scripts: &self.scripts,
                scriptlets: &self.scriptlets,
                advanced_query_results_empty,
                menu_syntax_ai_proposal,
            },
        )?;

        if matches!(
            snapshot.kind,
            crate::menu_syntax::MenuSyntaxMainHintKind::CaptureComposer
        ) {
            if let Some(target) = self.capture_target_for(raw_filter_text) {
                let store = crate::menu_syntax::history::HistoryStore::from_env();
                let tag_pool = store.try_read_tag_pool(&target).unwrap_or_default();
                if let Ok(pool) = store.try_read_tag_pool(&target) {
                    let recent: Vec<String> = pool
                        .iter()
                        .take(5)
                        .map(|tf| format!("#{}", tf.tag))
                        .collect();
                    if !recent.is_empty() {
                        let value = recent.join(" ");
                        if let Some(existing) =
                            snapshot.rows.iter_mut().find(|row| row.label == "Tags")
                        {
                            existing.value = value;
                        } else {
                            snapshot
                                .rows
                                .push(crate::menu_syntax::MenuSyntaxMainHintRow {
                                    label: "Recent tags".to_string(),
                                    value,
                                    chips: Vec::new(),
                                });
                        }
                    }
                }

                let invocation_for_form =
                    self.menu_syntax_capture_form_invocation(raw_filter_text);

                if let Some(invocation) = invocation_for_form.as_ref() {
                    let mut seen: std::collections::HashSet<&str> =
                        std::collections::HashSet::new();
                    for (key, _) in invocation.kv.iter().take(3) {
                        if !seen.insert(key.as_str()) {
                            continue;
                        }
                        if let Ok(pool) = store.try_read_key_pool(&target, key) {
                            let recent: Vec<String> =
                                pool.iter().take(3).map(|vf| vf.value.clone()).collect();
                            if !recent.is_empty() {
                                snapshot
                                    .rows
                                    .push(crate::menu_syntax::MenuSyntaxMainHintRow {
                                        label: format!("Recent {key}"),
                                        value: recent.join(", "),
                                        chips: Vec::new(),
                                    });
                            }
                        }
                    }
                    if let Some(schema) = crate::menu_syntax::builtin_schema(&target) {
                        let validation = crate::menu_syntax::capture_schema::validate(
                            invocation,
                            &schema,
                        );
                        let priority_values = schema
                            .optional
                            .iter()
                            .chain(schema.required.iter())
                            .find(|req| {
                                matches!(
                                    req,
                                    crate::menu_syntax::capture_schema::FieldRequirement::Priority
                                )
                            })
                            .map(|req| {
                                req.enum_values()
                                    .iter()
                                    .map(|value| value.to_string())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        let mut form = crate::menu_syntax::build_capture_form_snapshot(
                            &schema,
                            invocation,
                            self.menu_syntax_form_focused_index,
                            &validation,
                            crate::menu_syntax::MenuSyntaxFormSuggestionPools {
                                tags: tag_pool,
                                priority_values,
                                date_values: store
                                    .try_read_key_pool(&target, "date")
                                    .unwrap_or_default(),
                                url_values: store
                                    .try_read_key_pool(&target, "url")
                                    .unwrap_or_default(),
                            },
                        );
                        if self.menu_syntax_form_input_active {
                            for field in &mut form.fields {
                                if field.focused
                                    && self
                                        .menu_syntax_form_draft_field_id
                                        .as_deref()
                                        .is_some_and(|id| id == field.id)
                                {
                                    field.value = self.menu_syntax_form_draft_value.clone();
                                }
                            }
                        } else {
                            for field in &mut form.fields {
                                field.focused = false;
                            }
                        }
                        snapshot.form = Some(form);
                    }
                }
            }
        }

        Some(snapshot)
    }

    fn capture_target_for(&self, raw_filter_text: &str) -> Option<String> {
        if let Some(invocation) = self.menu_syntax_mode.capture_for(raw_filter_text) {
            return Some(invocation.target.clone());
        }
        if let Some(incomplete) = self.menu_syntax_mode.incomplete_for(raw_filter_text) {
            if let crate::menu_syntax::payload::IncompleteKind::MissingCaptureBody(target) =
                &incomplete.kind
            {
                return Some(target.clone());
            }
        }
        None
    }

    fn menu_syntax_capture_form_invocation(
        &self,
        raw_filter_text: &str,
    ) -> Option<crate::menu_syntax::payload::CaptureInvocation> {
        if let Some(invocation) = self.menu_syntax_mode.capture_for(raw_filter_text) {
            return Some(invocation.clone());
        }
        let incomplete = self.menu_syntax_mode.incomplete_for(raw_filter_text)?;
        if let crate::menu_syntax::payload::IncompleteKind::MissingCaptureBody(target) =
            &incomplete.kind
        {
            return Some(crate::menu_syntax::empty_capture_invocation(
                target,
                raw_filter_text,
            ));
        }
        None
    }
}
