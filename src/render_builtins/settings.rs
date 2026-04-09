/// Settings item definition for the hub view.
struct SettingsItem {
    name: &'static str,
    description: &'static str,
    icon: &'static str,
    action: SettingsAction,
}

/// Action to execute when a settings item is selected.
#[derive(Clone)]
enum SettingsAction {
    ChooseTheme,
    ConfigureVercelApiKey,
    ConfigureOpenAiApiKey,
    ConfigureAnthropicApiKey,
    ResetWindowPositions,
}

fn get_settings_items() -> Vec<SettingsItem> {
    let mut items = vec![
        SettingsItem {
            name: "Theme Designer",
            description: "Design your color theme with live preview",
            icon: "🎨",
            action: SettingsAction::ChooseTheme,
        },
        SettingsItem {
            name: "Configure Vercel AI Gateway",
            description: "Set up the Vercel AI Gateway API key for ACP Chat",
            icon: "🔑",
            action: SettingsAction::ConfigureVercelApiKey,
        },
        SettingsItem {
            name: "Configure OpenAI API Key",
            description: "Set up the OpenAI API key for ACP Chat",
            icon: "🔑",
            action: SettingsAction::ConfigureOpenAiApiKey,
        },
        SettingsItem {
            name: "Configure Anthropic API Key",
            description: "Set up the Anthropic API key for ACP Chat",
            icon: "🔑",
            action: SettingsAction::ConfigureAnthropicApiKey,
        },
    ];

    // Only show reset if there are saved positions
    if crate::window_state::has_custom_positions() {
        items.push(SettingsItem {
            name: "Reset Window Positions",
            description: "Restore all windows to default positions",
            icon: "🔄",
            action: SettingsAction::ResetWindowPositions,
        });
    }

    items
}

impl ScriptListApp {
    /// Execute a settings action selected from the settings hub.
    fn execute_settings_action(
        &mut self,
        action: &SettingsAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            SettingsAction::ChooseTheme => {
                tracing::info!(
                    correlation_id = "settings-hub",
                    action = "choose_theme",
                    "settings.action_executed"
                );
                self.open_theme_chooser_view(cx);
            }
            SettingsAction::ConfigureVercelApiKey => {
                tracing::info!(
                    correlation_id = "settings-hub",
                    action = "configure_vercel_api_key",
                    "settings.action_executed"
                );
                self.show_api_key_prompt(
                    "SCRIPT_KIT_VERCEL_API_KEY",
                    "Enter your Vercel AI Gateway API key",
                    "Vercel AI Gateway",
                    cx,
                );
            }
            SettingsAction::ConfigureOpenAiApiKey => {
                tracing::info!(
                    correlation_id = "settings-hub",
                    action = "configure_openai_api_key",
                    "settings.action_executed"
                );
                self.show_api_key_prompt(
                    "SCRIPT_KIT_OPENAI_API_KEY",
                    "Enter your OpenAI API key",
                    "OpenAI",
                    cx,
                );
            }
            SettingsAction::ConfigureAnthropicApiKey => {
                tracing::info!(
                    correlation_id = "settings-hub",
                    action = "configure_anthropic_api_key",
                    "settings.action_executed"
                );
                self.show_api_key_prompt(
                    "SCRIPT_KIT_ANTHROPIC_API_KEY",
                    "Enter your Anthropic API key",
                    "Anthropic",
                    cx,
                );
            }
            SettingsAction::ResetWindowPositions => {
                tracing::info!(
                    correlation_id = "settings-hub",
                    action = "reset_window_positions",
                    "settings.action_executed"
                );
                crate::window_state::suppress_save();
                crate::window_state::reset_all_positions();
                self.show_hud(
                    "Window positions reset - takes effect next open".to_string(),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                self.close_and_reset_window(cx);
            }
        }
    }

    /// Render the settings hub view with categorized configuration options.
    fn render_settings(
        &mut self,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::exception(
                "settings",
                "settings_hub_with_categorized_options",
            ),
        );
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let text_primary = self.theme.colors.text.primary;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;

        let items = get_settings_items();
        let item_count = items.len();
        let list_colors = ListItemColors::from_theme(&self.theme);

        // Key handler
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                this.hide_mouse_cursor(cx);

                let key = event.keystroke.key.as_str();
                let has_cmd = event.keystroke.modifiers.platform;

                // ESC: go back/close
                if is_key_escape(key) {
                    this.go_back_or_close(window, cx);
                    cx.stop_propagation();
                    return;
                }

                // Cmd+W always closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    this.close_and_reset_window(cx);
                    cx.stop_propagation();
                    return;
                }

                let current_selected =
                    if let AppView::SettingsView { selected_index } = &this.current_view {
                        *selected_index
                    } else {
                        return;
                    };

                let settings_items = get_settings_items();
                let settings_count = settings_items.len();

                if is_key_up(key) {
                    if current_selected > 0 {
                        if let AppView::SettingsView { selected_index } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected - 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_down(key) {
                    if current_selected < settings_count.saturating_sub(1) {
                        if let AppView::SettingsView { selected_index } =
                            &mut this.current_view
                        {
                            *selected_index = current_selected + 1;
                        }
                        cx.notify();
                    }
                    cx.stop_propagation();
                } else if is_key_enter(key) {
                    if let Some(item) = settings_items.get(current_selected) {
                        let action = item.action.clone();
                        this.execute_settings_action(&action, window, cx);
                    }
                    cx.stop_propagation();
                } else {
                    cx.propagate();
                }
            },
        );

        // Build list items
        let entity = cx.entity().downgrade();
        let hovered = self.hovered_index;

        let list_items: Vec<AnyElement> = items
            .iter()
            .enumerate()
            .map(|(ix, item)| {
                let is_selected = ix == selected_index;
                let is_hovered = hovered == Some(ix);
                let action = item.action.clone();
                let entity_click = entity.clone();
                let entity_hover = entity.clone();
                let name_str = format!("{} {}", item.icon, item.name);
                let desc = item.description.to_string();

                div()
                    .id(ix)
                    .cursor_pointer()
                    .on_click(move |_event, window, cx| {
                        if let Some(app) = entity_click.upgrade() {
                            app.update(cx, |this, cx| {
                                this.execute_settings_action(&action, window, cx);
                            });
                        }
                    })
                    .on_hover({
                        let entity_h = entity_hover;
                        move |is_hovered: &bool, _window: &mut Window, cx: &mut gpui::App| {
                            if let Some(app) = entity_h.upgrade() {
                                app.update(cx, |this, cx| {
                                    if *is_hovered {
                                        this.input_mode = InputMode::Mouse;
                                        if this.hovered_index != Some(ix) {
                                            this.hovered_index = Some(ix);
                                            cx.notify();
                                        }
                                    } else if this.hovered_index == Some(ix) {
                                        this.hovered_index = None;
                                        cx.notify();
                                    }
                                });
                            }
                        }
                    })
                    .child(
                        ListItem::new(name_str, list_colors)
                            .description_opt(Some(desc))
                            .selected(is_selected)
                            .hovered(is_hovered)
                            .with_accent_bar(is_selected),
                    )
                    .into_any_element()
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .rounded(px(design_visual.radius_lg))
            .text_color(rgb(text_primary))
            .font_family(design_typography.font_family)
            .key_context("settings")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header
            .child(
                div()
                    .w_full()
                    .px(px(design_spacing.padding_lg))
                    .py(px(design_spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(design_typography.font_size_xl))
                            .child("Settings"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(format!(
                                "{} option{}",
                                item_count,
                                if item_count == 1 { "" } else { "s" }
                            )),
                    ),
            )
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Settings list
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .overflow_hidden()
                    .py(px(design_spacing.padding_xs))
                    .flex()
                    .flex_col()
                    .children(list_items),
            )
            .child(if matches!(
                crate::footer_popup::active_main_window_footer_surface(),
                Some("settings")
            ) {
                crate::components::prompt_layout_shell::render_native_main_window_footer_spacer()
            } else {
                PromptFooter::new(
                    PromptFooterConfig::new()
                        .primary_label("Open")
                        .primary_shortcut("↵")
                        .show_secondary(false),
                    PromptFooterColors::from_theme(&self.theme),
                )
                .into_any_element()
            })
            .into_any_element()
    }
}
