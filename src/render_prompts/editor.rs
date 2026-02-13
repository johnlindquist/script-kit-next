mod __render_prompts_editor_docs {
    //! Editor prompt rendering helpers and `ScriptListApp::render_editor_prompt`.
    //! Key routines include footer/status builders and shortcut filtering for editor-reserved bindings.
    //! This file depends on `editor`, `ui_foundation`, and actions-dialog utilities from the main app.
}

// Editor prompt render method - extracted from render_prompts.rs
// This file is included via include!() macro in main.rs

const EDITOR_PROMPT_KEY_CONTEXT: &str = "editor_prompt";
const EDITOR_PROMPT_SHORTCUT_HINT_SUFFIX: &str = "⌘↵/⌘S submit · ⌘K actions";

#[inline]
fn editor_footer_helper_text(snippet_helper_text: Option<&str>) -> String {
    match snippet_helper_text {
        Some(snippet_text) => format!("{snippet_text} · {EDITOR_PROMPT_SHORTCUT_HINT_SUFFIX}"),
        None => running_status_text("review input, then press ⌘↵ or ⌘S (⌘K for actions)"),
    }
}

#[inline]
fn editor_footer_config(
    has_actions: bool,
    helper_text: Option<String>,
    info_label: Option<String>,
) -> PromptFooterConfig {
    prompt_footer_config_with_status("Continue", has_actions, helper_text, info_label)
        .primary_shortcut("⌘↵")
        .secondary_shortcut("⌘K")
}

#[inline]
fn editor_reserved_shortcut_reason(key: &str, modifiers: &gpui::Modifiers) -> Option<&'static str> {
    // Keep native editor bindings in control of text editing, navigation, and submit.
    if !modifiers.platform || modifiers.control || modifiers.alt {
        return None;
    }

    match key {
        "enter" | "return" => Some("submit"),
        "s" => Some("save_submit"),
        "z" | "y" => Some("undo_redo"),
        "f" | "g" => Some("find"),
        "a" | "c" | "v" | "x" => Some("clipboard_selection"),
        "left" | "right" | "up" | "down" | "arrowleft" | "arrowright" | "arrowup" | "arrowdown" => {
            Some("cursor_navigation")
        }
        _ => None,
    }
}

impl ScriptListApp {
    fn render_editor_prompt(
        &mut self,
        entity: Entity<EditorPrompt>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let render_context = PromptRenderContext::new(self.theme.as_ref(), self.current_design);
        let theme = render_context.theme;
        let design_visual = render_context.design_visual;
        let actions_dialog_top = render_context.actions_dialog_top;
        let actions_dialog_right = render_context.actions_dialog_right;
        let has_actions =
            self.sdk_actions.is_some() && !self.sdk_actions.as_ref().unwrap().is_empty();

        // Sync suppress_keys with actions popup state so editor ignores keys when popup is open
        let show_actions = self.show_actions_popup;
        entity.update(cx, |editor, _| {
            editor.suppress_keys = show_actions;
        });

        // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
        // Shadows are handled by app_shell
        let _box_shadows = self.create_box_shadows();

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(theme);

        // Use explicit height from layout constants instead of h_full()
        // h_full() doesn't work at the root level because there's no parent to fill
        let content_height = window_resize::layout::MAX_HEIGHT;

        // Key handler for Cmd+K actions toggle (at parent level to intercept before editor)
        let has_actions_for_handler = has_actions;
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                if key_preamble(this, event, false, false, cx) {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let has_cmd = event.keystroke.modifiers.platform;

                // For ScratchPadView (built-in utility): ESC returns to main menu or closes window
                // This is different from EditorPrompt (SDK prompt) which doesn't respond to ESC
                if matches!(this.current_view, AppView::ScratchPadView { .. }) {
                    if key_str == "escape" && !this.show_actions_popup {
                        logging::log("KEY", "ESC in ScratchPadView");
                        this.go_back_or_close(window, cx);
                        return;
                    }

                    if has_cmd && key_str == "w" {
                        logging::log("KEY", "Cmd+W - closing window");
                        this.close_and_reset_window(cx);
                        return;
                    }
                }

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                // Check for Cmd+K to toggle actions popup (if actions are available)
                if has_cmd && ui_foundation::is_key_k(key) && has_actions_for_handler {
                    let correlation_id = logging::current_correlation_id();
                    logging::log(
                        "KEY",
                        &format!(
                            "{EDITOR_PROMPT_KEY_CONTEXT}: Cmd+K toggles actions (correlation_id={correlation_id})"
                        ),
                    );
                    this.toggle_arg_actions(cx, window);
                    return;
                }

                let modifiers = &event.keystroke.modifiers;

                // Route to shared actions dialog handler (modal when open)
                match this.route_key_to_actions_dialog(
                    key,
                    key_char,
                    modifiers,
                    ActionsDialogHost::EditorPrompt,
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
                if let Some(reason) =
                    editor_reserved_shortcut_reason(&key_lower, &event.keystroke.modifiers)
                {
                    let correlation_id = logging::current_correlation_id();
                    logging::log_debug(
                        "KEY",
                        &format!(
                            "{EDITOR_PROMPT_KEY_CONTEXT}: reserved shortcut preserved (reason={reason}, shortcut={shortcut_key}, correlation_id={correlation_id})"
                        ),
                    );
                    return;
                }
                if let Some(action_name) = this.action_shortcuts.get(&shortcut_key).cloned() {
                    let correlation_id = logging::current_correlation_id();
                    logging::log(
                        "KEY",
                        &format!(
                            "{EDITOR_PROMPT_KEY_CONTEXT}: SDK action shortcut matched (action={action_name}, shortcut={shortcut_key}, correlation_id={correlation_id})"
                        ),
                    );
                    this.trigger_action_by_name(&action_name, cx);
                }
                // Let other keys fall through to the editor
            },
        );

        // Clone entity for footer submit button
        let entity_for_footer = entity.clone();

        // Get the prompt ID for submit
        let prompt_id = entity.read(cx).id.clone();

        // Extract editor state for footer display
        let (snippet_helper_text, language_label) = {
            let editor = entity.read(cx);
            let language = editor.language().to_string();

            // Build snippet indicator if in snippet mode
            let snippet_text = editor.snippet_state().map(|state| {
                let current = state.current_tabstop_idx + 1; // 1-based for display
                let total = state.snippet.tabstops.len();

                // Get the current tabstop's display name (placeholder or index)
                let current_name = state
                    .snippet
                    .tabstops
                    .get(state.current_tabstop_idx)
                    .and_then(|ts| {
                        ts.placeholder.clone().or_else(|| {
                            ts.choices
                                .as_ref()
                                .and_then(|c: &Vec<String>| c.first().cloned())
                        })
                    })
                    .unwrap_or_else(|| format!("${}", current));

                format!(
                    "Tab {} of {} · \"{}\" · Tab to continue, Esc to exit",
                    current, total, current_name
                )
            });

            (snippet_text, language)
        };

        // NOTE: The EditorPrompt entity has its own track_focus and on_key_down in its render method.
        // We do NOT add track_focus here to avoid duplicate focus tracking on the same handle.
        //
        // Container with flex layout:
        // - Editor wrapper using flex_1 to fill remaining space above footer
        // - Footer as normal child at bottom (40px fixed height)
        div()
            .relative() // Needed for absolute positioned actions dialog overlay
            .flex()
            .flex_col()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // VIBRANCY: Only apply bg when vibrancy disabled
            // NOTE: No shadow - shadows on transparent elements cause gray fill with vibrancy
            .w_full()
            .h(content_height) // Explicit 700px height (window height for editor view)
            .overflow_hidden() // Clip content to rounded corners
            .rounded(px(design_visual.radius_lg))
            .key_context(EDITOR_PROMPT_KEY_CONTEXT)
            .on_key_down(handle_key)
            // Editor entity fills remaining space (flex_1)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.)) // Required for flex children to shrink properly
                    .w_full()
                    .overflow_hidden()
                    .child(entity),
            )
            // Unified footer with Submit + Actions
            .child({
                let handle_submit = cx.entity().downgrade();
                let handle_actions = cx.entity().downgrade();
                let entity_weak = entity_for_footer.downgrade();
                let prompt_id_for_submit = prompt_id.clone();

                let footer_colors = PromptFooterColors::from_theme(theme);

                // Snippet guidance stays first, with editor submit/actions hints appended.
                let helper_text = Some(editor_footer_helper_text(snippet_helper_text.as_deref()));
                let footer_config =
                    editor_footer_config(has_actions, helper_text, Some(language_label.clone()));

                let mut footer = PromptFooter::new(footer_config, footer_colors).on_primary_click(
                    Box::new(move |_, _window, cx| {
                        // Get editor content and submit
                        if let Some(editor_entity) = entity_weak.upgrade() {
                            let content = editor_entity.update(cx, |editor, cx| editor.content(cx));
                            if let Some(app) = handle_submit.upgrade() {
                                app.update(cx, |this, cx| {
                                    logging::log("EDITOR", "Footer Submit button clicked");
                                    this.submit_prompt_response(
                                        prompt_id_for_submit.clone(),
                                        Some(content),
                                        cx,
                                    );
                                });
                            }
                        }
                    }),
                );

                if has_actions {
                    footer = footer.on_secondary_click(Box::new(move |_, window, cx| {
                        if let Some(app) = handle_actions.upgrade() {
                            app.update(cx, |this, cx| {
                                this.toggle_arg_actions(cx, window);
                            });
                        }
                    }));
                }

                // Footer as normal flex child (not absolute positioned)
                // The flex_1 editor wrapper above takes remaining space
                footer
            })
            // Actions dialog overlay
            .when_some(
                render_actions_backdrop(
                    self.show_actions_popup,
                    self.actions_dialog.clone(),
                    actions_dialog_top,
                    actions_dialog_right,
                    ActionsBackdropConfig {
                        backdrop_id: "editor-actions-backdrop",
                        close_host: ActionsDialogHost::EditorPrompt,
                        backdrop_log_message:
                            "Editor actions backdrop clicked - dismissing dialog",
                        show_pointer_cursor: true,
                    },
                    cx,
                ),
                |d, backdrop_overlay| d.child(backdrop_overlay),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod editor_prompt_tests {
    use super::*;

    fn cmd_modifiers() -> gpui::Modifiers {
        gpui::Modifiers {
            platform: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_editor_footer_displays_cmd_enter_and_cmd_s_submit_hints() {
        let helper = editor_footer_helper_text(None);
        assert!(helper.contains("⌘↵"));
        assert!(helper.contains("⌘S"));
        assert!(helper.contains("⌘K"));

        let config = editor_footer_config(true, Some(helper), Some("typescript".to_string()));
        assert_eq!(config.primary_label, "Continue");
        assert_eq!(config.primary_shortcut, "⌘↵");
        assert_eq!(config.secondary_shortcut, "⌘K");
        assert!(config.show_secondary);
    }

    #[test]
    fn test_editor_action_shortcuts_do_not_override_reserved_editing_bindings() {
        let cmd = cmd_modifiers();

        for key in ["f", "z", "return", "s", "arrowleft", "c"] {
            assert!(
                editor_reserved_shortcut_reason(key, &cmd).is_some(),
                "{key} should stay editor-owned",
            );
        }

        assert!(
            editor_reserved_shortcut_reason("k", &cmd).is_none(),
            "Cmd+K remains available for actions",
        );

        let cmd_and_ctrl = gpui::Modifiers {
            platform: true,
            control: true,
            ..Default::default()
        };
        assert!(
            editor_reserved_shortcut_reason("f", &cmd_and_ctrl).is_none(),
            "Cmd+Ctrl+F is not treated as an editor-reserved shortcut",
        );
    }

    #[test]
    fn test_editor_footer_appends_shortcut_hints_when_snippet_guidance_is_present() {
        let helper =
            editor_footer_helper_text(Some("Tab 1 of 2 · \"$1\" · Tab to continue, Esc to exit"));
        assert!(helper.starts_with("Tab 1 of 2"));
        assert!(helper.contains(EDITOR_PROMPT_SHORTCUT_HINT_SUFFIX));
    }

    #[test]
    fn test_editor_actions_backdrop_uses_cursor_pointer_when_clickable() {
        const EDITOR_RENDER_SOURCE: &str = include_str!("editor.rs");

        assert!(
            EDITOR_RENDER_SOURCE.contains("render_actions_backdrop("),
            "editor render should delegate backdrop overlay creation to shared helper"
        );
        assert!(
            EDITOR_RENDER_SOURCE.contains("\"editor-actions-backdrop\""),
            "editor render should pass its backdrop id to shared helper"
        );
        assert!(
            EDITOR_RENDER_SOURCE.contains("ActionsDialogHost::EditorPrompt"),
            "editor render should preserve actions host routing when helper is used"
        );
        assert!(
            EDITOR_RENDER_SOURCE.contains("show_pointer_cursor: true"),
            "editor render should keep backdrop cursor pointer enabled"
        );
    }
}
