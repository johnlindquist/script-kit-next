#[inline]
fn prompt_actions_dialog_offsets(padding_sm: f32, border_thin: f32) -> (f32, f32) {
    // Keep dialog anchored just below the shared header + divider.
    let top = crate::panel::HEADER_TOTAL_HEIGHT + padding_sm - border_thin;
    let right = padding_sm;
    (top, right)
}

#[inline]
fn running_status_text(context: &str) -> String {
    crate::panel::running_status_message(context)
}

#[inline]
fn prompt_footer_colors_for_prompt(
    design_colors: &crate::designs::DesignColors,
    is_light_mode: bool,
) -> PromptFooterColors {
    PromptFooterColors {
        accent: design_colors.accent,
        text_muted: design_colors.text_muted,
        border: design_colors.border,
        background: design_colors.background_selected,
        is_light_mode,
    }
}

#[inline]
fn prompt_footer_config_with_status(
    primary_label: &str,
    show_secondary: bool,
    helper_text: Option<String>,
    info_label: Option<String>,
) -> PromptFooterConfig {
    let mut config = PromptFooterConfig::new()
        .primary_label(primary_label)
        .primary_shortcut("↵")
        .secondary_label("Actions")
        .secondary_shortcut("⌘K")
        .show_secondary(show_secondary);

    if let Some(helper) = helper_text {
        config = config.helper_text(helper);
    }

    if let Some(info) = info_label {
        config = config.info_label(info);
    }

    config
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgSubmitOutcome {
    SubmitChoice(String),
    SubmitText(String),
    InvalidEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgHelperStatus {
    NavigateChoices,
    NoMatchesSubmitTypedValue,
    TypeValueToContinue,
    SubmitTypedValue,
}

#[inline]
fn resolve_arg_submit_outcome(
    selected_choice_value: Option<&str>,
    input_text: &str,
) -> ArgSubmitOutcome {
    if let Some(value) = selected_choice_value {
        return ArgSubmitOutcome::SubmitChoice(value.to_string());
    }

    if input_text.is_empty() {
        return ArgSubmitOutcome::InvalidEmpty;
    }

    ArgSubmitOutcome::SubmitText(input_text.to_string())
}

#[inline]
fn resolve_arg_helper_status(
    has_choices: bool,
    filtered_choices_len: usize,
    input_is_empty: bool,
) -> ArgHelperStatus {
    if has_choices && filtered_choices_len > 0 {
        return ArgHelperStatus::NavigateChoices;
    }

    if has_choices && !input_is_empty {
        return ArgHelperStatus::NoMatchesSubmitTypedValue;
    }

    if input_is_empty {
        return ArgHelperStatus::TypeValueToContinue;
    }

    ArgHelperStatus::SubmitTypedValue
}

#[inline]
fn arg_helper_status_text(status: ArgHelperStatus) -> String {
    match status {
        ArgHelperStatus::NavigateChoices => {
            running_status_text("use ↑/↓ to choose, Enter to continue")
        }
        ArgHelperStatus::NoMatchesSubmitTypedValue => {
            running_status_text("no matches · Enter submits typed value")
        }
        ArgHelperStatus::TypeValueToContinue => running_status_text("type a value and press Enter"),
        ArgHelperStatus::SubmitTypedValue => {
            running_status_text("press Enter to submit typed value")
        }
    }
}

#[inline]
fn resolve_arg_tab_completion(
    filtered: &[(usize, &Choice)],
    selected_index: usize,
) -> Option<String> {
    if filtered.is_empty() {
        return None;
    }

    if filtered.len() == 1 {
        return filtered.first().map(|(_, choice)| choice.name.clone());
    }

    filtered
        .get(selected_index)
        .or_else(|| filtered.first())
        .map(|(_, choice)| choice.name.clone())
}

impl ScriptListApp {
    #[inline]
    fn arg_prompt_has_choices(&self) -> bool {
        matches!(&self.current_view, AppView::ArgPrompt { choices, .. } if !choices.is_empty())
    }

    #[inline]
    fn sync_arg_prompt_after_text_change(
        &mut self,
        prev_original_idx: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let has_choices = self.arg_prompt_has_choices();
        let (new_selected_idx, filtered_len) = {
            let filtered = self.filtered_arg_choices();
            let new_idx = if let Some(prev_idx) = prev_original_idx {
                filtered
                    .iter()
                    .position(|(orig_idx, _)| *orig_idx == prev_idx)
                    .unwrap_or(0)
            } else {
                0
            };

            (new_idx, filtered.len())
        };

        self.arg_selected_index = new_selected_idx;

        // Defer resize through window_ops to avoid RefCell borrow conflicts during native callbacks.
        let (view_type, item_count) = if filtered_len == 0 {
            if has_choices {
                (ViewType::ArgPromptWithChoices, 0)
            } else {
                (ViewType::ArgPromptNoChoices, 0)
            }
        } else {
            (ViewType::ArgPromptWithChoices, filtered_len)
        };
        let target_height = crate::window_resize::height_for_view(view_type, item_count);
        crate::window_ops::queue_resize(f32::from(target_height), window, &mut *cx);
    }

    #[inline]
    fn resolve_current_arg_submit_outcome(&self) -> ArgSubmitOutcome {
        let filtered = self.filtered_arg_choices();
        let selected_choice_value = filtered
            .get(self.arg_selected_index)
            .map(|(_, choice)| choice.value.as_str());
        resolve_arg_submit_outcome(selected_choice_value, self.arg_input.text())
    }

    #[inline]
    fn submit_arg_prompt_from_current_state(&mut self, prompt_id: &str, cx: &mut Context<Self>) {
        match self.resolve_current_arg_submit_outcome() {
            ArgSubmitOutcome::SubmitChoice(value) | ArgSubmitOutcome::SubmitText(value) => {
                self.submit_prompt_response(prompt_id.to_string(), Some(value), cx);
            }
            ArgSubmitOutcome::InvalidEmpty => {
                self.show_hud("Type a value to continue".to_string(), Some(1500), cx);
            }
        }
    }

    #[inline]
    fn apply_arg_tab_completion(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let (completion_text, prev_original_idx) = {
            let filtered = self.filtered_arg_choices();
            (
                resolve_arg_tab_completion(&filtered, self.arg_selected_index),
                filtered
                    .get(self.arg_selected_index)
                    .map(|(original_idx, _)| *original_idx),
            )
        };

        let Some(completion_text) = completion_text else {
            return false;
        };

        if self.arg_input.text() == completion_text {
            return true;
        }

        self.arg_input.set_text(completion_text);
        self.arg_input.move_to_end(false);
        self.sync_arg_prompt_after_text_change(prev_original_idx, window, cx);
        cx.notify();
        true
    }
}
