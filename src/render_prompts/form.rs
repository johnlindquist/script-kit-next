// Form prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

use gpui_component::scroll::ScrollableElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormEnterBehavior {
    Submit,
    ForwardToField,
    Ignore,
}

#[inline]
fn form_enter_behavior(
    key: &str,
    has_cmd: bool,
    focused_field_is_textarea: bool,
) -> FormEnterBehavior {
    if !ui_foundation::is_key_enter(key) {
        return FormEnterBehavior::Ignore;
    }

    if focused_field_is_textarea && !has_cmd {
        return FormEnterBehavior::ForwardToField;
    }

    FormEnterBehavior::Submit
}

#[inline]
fn focused_form_field_is_textarea(form: &FormPromptState) -> bool {
    form.fields
        .get(form.focused_index)
        .and_then(|(field, _)| field.field_type.as_deref())
        .is_some_and(|field_type| field_type.eq_ignore_ascii_case("textarea"))
}

#[inline]
fn form_footer_status_text(focused_field_is_textarea: bool) -> String {
    if focused_field_is_textarea {
        running_status_text("press ⌘↵ to submit (Enter adds a new line)")
    } else {
        running_status_text("press Enter to submit")
    }
}

#[inline]
fn form_field_value_for_validation(
    field_entity: &crate::form_prompt::FormFieldEntity,
    cx: &App,
) -> String {
    match field_entity {
        crate::form_prompt::FormFieldEntity::TextField(entity) => {
            entity.read(cx).value().to_string()
        }
        crate::form_prompt::FormFieldEntity::TextArea(entity) => {
            entity.read(cx).value().to_string()
        }
        crate::form_prompt::FormFieldEntity::Checkbox(entity) => {
            if entity.read(cx).is_checked() {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
    }
}

#[inline]
fn is_valid_email_submit_value(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }

    if value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return false;
    }

    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();

    if local.is_empty() || domain.is_empty() || parts.next().is_some() {
        return false;
    }

    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }

    domain.contains('.')
}

#[inline]
fn is_valid_number_submit_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    trimmed.parse::<f64>().is_ok()
}

#[inline]
fn form_field_value_is_valid_for_submit(field_type: Option<&str>, value: &str) -> bool {
    match field_type {
        Some(field_type) if field_type.eq_ignore_ascii_case("email") => {
            is_valid_email_submit_value(value)
        }
        Some(field_type) if field_type.eq_ignore_ascii_case("number") => {
            is_valid_number_submit_value(value)
        }
        _ => true,
    }
}

#[inline]
fn collect_form_submit_validation_errors(form: &FormPromptState, cx: &App) -> Vec<String> {
    let mut invalid_fields = Vec::new();

    for (field_definition, field_entity) in &form.fields {
        let value = form_field_value_for_validation(field_entity, cx);
        if form_field_value_is_valid_for_submit(field_definition.field_type.as_deref(), &value) {
            continue;
        }

        let field_type = field_definition
            .field_type
            .as_deref()
            .unwrap_or("text")
            .to_string();
        invalid_fields.push(format!("{} ({})", field_definition.name, field_type));
    }

    invalid_fields
}

#[inline]
fn form_submit_validation_message(invalid_fields: &[String]) -> String {
    if invalid_fields.len() == 1 {
        format!("Fix invalid field before submitting: {}", invalid_fields[0])
    } else {
        format!(
            "Fix invalid fields before submitting: {}",
            invalid_fields.join(", ")
        )
    }
}

impl ScriptListApp {
    fn render_form_prompt(
        &mut self,
        entity: Entity<FormPromptState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();
        let (actions_dialog_top, actions_dialog_right) =
            prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin);

        // Get prompt ID and field count from entity
        let (prompt_id, field_count, focused_field_is_textarea) = {
            let form_state = entity.read(cx);
            (
                form_state.id.clone(),
                form_state.fields.len(),
                focused_form_field_is_textarea(form_state),
            )
        };

        // Clone entity for closures
        let entity_for_submit = entity.clone();
        let entity_for_tab = entity.clone();
        let entity_for_shift_tab = entity.clone();
        let entity_for_input = entity.clone();

        let prompt_id_for_key = prompt_id.clone();
        // Key handler for form navigation (Enter/Tab/Escape) and Cmd+K actions
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                // Global shortcuts (Cmd+W, ESC for dismissable prompts)
                // Note: Escape when actions popup is open should close the popup, not dismiss prompt
                if !this.show_actions_popup
                    && this.handle_global_shortcut_with_options(event, true, cx)
                {
                    return;
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_shift = event.keystroke.modifiers.shift;
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    logging::log("KEY", "Cmd+K in FormPrompt - calling toggle_arg_actions");
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::FormPrompt,
                    window,
                    cx,
                ) {
                    ActionsRoute::Execute { action_id } => {
                        this.trigger_action_by_name(&action_id, cx);
                        return;
                    }
                    ActionsRoute::Handled => {
                        // Key consumed by actions dialog
                        return;
                    }
                    ActionsRoute::NotHandled => {
                        // Actions popup not open - continue with normal handling
                    }
                }

                // Check for SDK action shortcuts (only when popup is NOT open)
                let key_lower = key.to_lowercase();
                let shortcut_key =
                    shortcuts::keystroke_to_shortcut(&key_lower, &event.keystroke.modifiers);
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    logging::log(
                        "KEY",
                        &format!("SDK action shortcut matched: {}", action_name),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                    return;
                }

                // Handle form-level keys (Enter, Escape, Tab) at this level
                // Forward all other keys to the focused form field for text input
                let focused_field_is_textarea = {
                    let form = entity_for_submit.read(cx);
                    focused_form_field_is_textarea(form)
                };
                match form_enter_behavior(key, has_cmd, focused_field_is_textarea) {
                    FormEnterBehavior::Submit => {
                        let validation_errors = {
                            let form = entity_for_submit.read(cx);
                            collect_form_submit_validation_errors(form, cx)
                        };
                        if !validation_errors.is_empty() {
                            let message = form_submit_validation_message(&validation_errors);
                            this.show_hud(message, Some(3000), cx);
                            return;
                        }

                        logging::log("KEY", "Enter in FormPrompt - submitting form");
                        let values = entity_for_submit.read(cx).collect_values(cx);
                        this.submit_prompt_response(prompt_id_for_key.clone(), Some(values), cx);
                        return;
                    }
                    FormEnterBehavior::ForwardToField => {
                        entity_for_input.update(cx, |form, cx| {
                            form.handle_key_input(event, cx);
                        });
                        return;
                    }
                    FormEnterBehavior::Ignore => {}
                }

                // Note: "escape" is handled by handle_global_shortcut_with_options above
                if key.eq_ignore_ascii_case("tab") {
                    // Tab navigation between fields
                    if has_shift {
                        entity_for_shift_tab.update(cx, |form, cx| {
                            form.focus_previous(cx);
                        });
                    } else {
                        entity_for_tab.update(cx, |form, cx| {
                            form.focus_next(cx);
                        });
                    }
                    return;
                }

                // Forward all other keys (characters, backspace, arrows, etc.) to the focused field
                // This is necessary because GPUI requires track_focus() to receive key events,
                // and we need the parent to have focus to handle Enter/Escape/Tab.
                // The form fields' individual on_key_down handlers don't fire when parent has focus.
                entity_for_input.update(cx, |form, cx| {
                    form.handle_key_input(event, cx);
                });
            },
        );

        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        // Dynamic height based on field count
        // Base height (150px) + per-field height (60px per field)
        // Minimum of calculated height and MAX_HEIGHT (700px)
        let base_height = 150.0;
        let field_height = 60.0;
        let calculated_height = base_height + (field_count as f32 * field_height);
        let max_height = 700.0; // Same as window_resize::layout::MAX_HEIGHT
        let content_height = px(calculated_height.min(max_height));

        // Form fields have their own focus handles and on_key_down handlers.
        // We DO NOT track_focus on the container - the fields handle their own focus.
        // Enter/Escape/Tab are handled by the handle_key listener above.
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h(content_height)
            .overflow_hidden()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("form_prompt")
            .on_key_down(handle_key)
            // Content area with form fields
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .min_h(px(0.))
                    .overflow_y_scrollbar()
                    .p(px(design_spacing.padding_xl))
                    // Render the form entity (contains all fields)
                    .child(entity.clone()),
            )
            // Unified footer with PromptFooter component
            .child({
                let footer_colors =
                    prompt_footer_colors_for_prompt(&design_colors, !self.theme.is_dark_mode());
                let footer_config = prompt_footer_config_with_status(
                    "Continue",
                    has_actions,
                    Some(form_footer_status_text(focused_field_is_textarea)),
                    Some(format!("{field_count} fields")),
                );

                let handle_actions = cx.entity().downgrade();

                PromptFooter::new(footer_config, footer_colors).on_secondary_click(Box::new(
                    move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    },
                ))
            })
            // Actions dialog overlay
            .when_some(
                if self.show_actions_popup {
                    self.actions_dialog.clone()
                } else {
                    None
                },
                |d, dialog| {
                    let backdrop_click = cx.listener(
                        |this: &mut Self,
                         _event: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            logging::log(
                                "FOCUS",
                                "Form actions backdrop clicked - dismissing dialog",
                            );
                            this.close_actions_popup(ActionsDialogHost::FormPrompt, window, cx);
                        },
                    );

                    d.child(
                        div()
                            .absolute()
                            .inset_0()
                            .child(
                                div()
                                    .id("form-actions-backdrop")
                                    .absolute()
                                    .inset_0()
                                    .on_click(backdrop_click),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top(px(actions_dialog_top))
                                    .right(px(actions_dialog_right))
                                    .child(dialog),
                            ),
                    )
                },
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod form_prompt_render_tests {
    use super::*;

    #[test]
    fn form_enter_behavior_submits_non_textarea_on_enter() {
        assert_eq!(
            form_enter_behavior("enter", false, false),
            FormEnterBehavior::Submit
        );
    }

    #[test]
    fn form_enter_behavior_forwards_textarea_enter_without_cmd() {
        assert_eq!(
            form_enter_behavior("enter", false, true),
            FormEnterBehavior::ForwardToField
        );
    }

    #[test]
    fn form_enter_behavior_submits_textarea_on_cmd_enter() {
        assert_eq!(
            form_enter_behavior("enter", true, true),
            FormEnterBehavior::Submit
        );
    }

    #[test]
    fn form_footer_status_text_mentions_cmd_enter_for_textarea() {
        assert_eq!(
            form_footer_status_text(true),
            running_status_text("press ⌘↵ to submit (Enter adds a new line)")
        );
    }

    #[test]
    fn form_footer_status_text_mentions_enter_for_non_textarea() {
        assert_eq!(
            form_footer_status_text(false),
            running_status_text("press Enter to submit")
        );
    }

    #[test]
    fn form_field_value_is_valid_for_submit_accepts_common_valid_inputs() {
        assert!(form_field_value_is_valid_for_submit(
            Some("email"),
            "user@example.com"
        ));
        assert!(form_field_value_is_valid_for_submit(Some("number"), "42.5"));
        assert!(form_field_value_is_valid_for_submit(Some("number"), ""));
    }

    #[test]
    fn form_field_value_is_valid_for_submit_rejects_invalid_email_and_number() {
        assert!(!form_field_value_is_valid_for_submit(
            Some("email"),
            "invalid-email"
        ));
        assert!(!form_field_value_is_valid_for_submit(
            Some("number"),
            "12abc"
        ));
    }
}
