#[inline]
fn prompt_actions_dialog_offsets(padding_sm: f32, border_thin: f32) -> (f32, f32) {
    // Keep dialog anchored just below the shared header + divider.
    let top = crate::panel::HEADER_TOTAL_HEIGHT + padding_sm - border_thin;
    let right = padding_sm;
    (top, right)
}

#[derive(Clone, Copy)]
struct PromptRenderContext<'a> {
    theme: &'a crate::theme::Theme,
    design_colors: crate::designs::DesignColors,
    design_spacing: crate::designs::DesignSpacing,
    design_typography: crate::designs::DesignTypography,
    design_visual: crate::designs::DesignVisual,
    actions_dialog_top: f32,
    actions_dialog_right: f32,
}

impl<'a> PromptRenderContext<'a> {
    #[inline]
    fn new(theme: &'a crate::theme::Theme, current_design: DesignVariant) -> Self {
        let tokens = get_tokens(current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let (actions_dialog_top, actions_dialog_right) =
            prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin);

        Self {
            theme,
            design_colors,
            design_spacing,
            design_typography,
            design_visual,
            actions_dialog_top,
            actions_dialog_right,
        }
    }
}

#[derive(Clone, Copy)]
struct ActionsBackdropConfig {
    backdrop_id: &'static str,
    close_host: ActionsDialogHost,
    backdrop_log_message: &'static str,
    show_pointer_cursor: bool,
}

#[derive(Clone, Copy)]
enum ActionsBackdropVerticalAnchor {
    Top(f32),
    Bottom(f32),
}

#[inline]
fn render_actions_backdrop_with_vertical_anchor(
    show_actions_popup: bool,
    actions_dialog: Option<Entity<ActionsDialog>>,
    actions_dialog_right: f32,
    vertical_anchor: ActionsBackdropVerticalAnchor,
    config: ActionsBackdropConfig,
    cx: &mut Context<ScriptListApp>,
) -> Option<gpui::Div> {
    if !show_actions_popup {
        return None;
    }

    let dialog = actions_dialog?;
    let backdrop_click = cx.listener(
        move |this: &mut ScriptListApp,
              _event: &gpui::ClickEvent,
              window: &mut Window,
              cx: &mut Context<ScriptListApp>| {
            logging::log("FOCUS", config.backdrop_log_message);
            this.close_actions_popup(config.close_host, window, cx);
        },
    );

    let dialog_container = match vertical_anchor {
        ActionsBackdropVerticalAnchor::Top(actions_dialog_top) => div()
            .absolute()
            .top(px(actions_dialog_top))
            .right(px(actions_dialog_right)),
        ActionsBackdropVerticalAnchor::Bottom(actions_dialog_bottom) => div()
            .absolute()
            .bottom(px(actions_dialog_bottom))
            .right(px(actions_dialog_right)),
    };

    Some(
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .id(config.backdrop_id)
                    .absolute()
                    .inset_0()
                    .when(config.show_pointer_cursor, |d| d.cursor_pointer())
                    .on_click(backdrop_click),
            )
            .child(dialog_container.child(dialog)),
    )
}

#[inline]
fn render_actions_backdrop(
    show_actions_popup: bool,
    actions_dialog: Option<Entity<ActionsDialog>>,
    actions_dialog_top: f32,
    actions_dialog_right: f32,
    config: ActionsBackdropConfig,
    cx: &mut Context<ScriptListApp>,
) -> Option<gpui::Div> {
    render_actions_backdrop_with_vertical_anchor(
        show_actions_popup,
        actions_dialog,
        actions_dialog_right,
        ActionsBackdropVerticalAnchor::Top(actions_dialog_top),
        config,
        cx,
    )
}

#[inline]
fn render_actions_backdrop_bottom_anchored(
    show_actions_popup: bool,
    actions_dialog: Option<Entity<ActionsDialog>>,
    actions_dialog_bottom: f32,
    actions_dialog_right: f32,
    config: ActionsBackdropConfig,
    cx: &mut Context<ScriptListApp>,
) -> Option<gpui::Div> {
    render_actions_backdrop_with_vertical_anchor(
        show_actions_popup,
        actions_dialog,
        actions_dialog_right,
        ActionsBackdropVerticalAnchor::Bottom(actions_dialog_bottom),
        config,
        cx,
    )
}

#[allow(dead_code)]
#[inline]
fn running_status_text(context: &str) -> String {
    crate::panel::running_status_message(context)
}

#[allow(dead_code)]
#[inline]
fn prompt_footer_colors_for_prompt(
    design_colors: &crate::designs::DesignColors,
    _is_light_mode: bool,
) -> PromptFooterColors {
    PromptFooterColors::from_design(design_colors)
}

#[allow(dead_code)]
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

#[inline]
fn key_preamble(
    app: &mut ScriptListApp,
    event: &gpui::KeyDownEvent,
    is_dismissable: bool,
    stop_propagation_on_global_shortcut: bool,
    cx: &mut Context<ScriptListApp>,
) -> bool {
    // When active, the shortcut recorder owns key handling for the prompt.
    if app.shortcut_recorder_state.is_some() {
        return true;
    }

    if !app.show_actions_popup && app.handle_global_shortcut_with_options(event, is_dismissable, cx) {
        if stop_propagation_on_global_shortcut {
            cx.stop_propagation();
        }
        return true;
    }

    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SdkActionShortcutMatch {
    action_name: String,
    shortcut_key: String,
}

#[inline]
fn check_sdk_action_shortcut(
    action_shortcuts: &std::collections::HashMap<String, String>,
    key: &str,
    modifiers: &gpui::Modifiers,
) -> Option<SdkActionShortcutMatch> {
    let shortcut_key = shortcuts::keystroke_to_shortcut(key, modifiers);
    action_shortcuts
        .get(&shortcut_key)
        .cloned()
        .map(|action_name| SdkActionShortcutMatch {
            action_name,
            shortcut_key,
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgSubmitOutcome {
    SubmitChoice(String),
    SubmitText(String),
    InvalidEmpty,
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

#[inline]
#[cfg(test)]
fn arg_prompt_hints(has_actions: bool) -> Vec<gpui::SharedString> {
    let mut hints = vec![
        gpui::SharedString::from("↵ Continue"),
        gpui::SharedString::from("Esc Back"),
    ];

    if has_actions {
        hints.insert(1, gpui::SharedString::from("⌘K Actions"));
    }

    hints
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
                self.show_hud("Type a value to continue".to_string(), Some(HUD_SHORT_MS), cx);
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
