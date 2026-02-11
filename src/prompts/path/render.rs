use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};

impl Focusable for PathPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PathPromptEvent> for PathPrompt {}

impl Render for PathPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Use ListItemColors for consistent theming - always use theme
        let list_colors = ListItemColors::from_theme(&self.theme);

        // Clone values needed for the closure
        let filtered_count = self.filtered_entries.len();
        let selected_index = self.selected_index;

        // TODO(codex-audit): This Vec snapshot is rebuilt to move data into the list closure.
        // Consider Arc-backed list data to avoid per-render allocation/copy churn.
        let entries_for_list: Vec<(String, bool)> = self
            .filtered_entries
            .iter()
            .map(|e| (e.name.clone(), e.is_dir))
            .collect();

        // Build list items using ListItem component for consistent styling
        let list = uniform_list(
            "path-list",
            filtered_count,
            move |visible_range: std::ops::Range<usize>, _window, _cx| {
                visible_range
                    .map(|ix| {
                        let (name, is_dir) = &entries_for_list[ix];
                        let is_selected = ix == selected_index;

                        // Choose icon based on entry type
                        let icon = if *is_dir {
                            IconKind::Emoji("📁".to_string())
                        } else {
                            IconKind::Emoji("📄".to_string())
                        };

                        // No description needed - folder icon 📁 is sufficient
                        let description: Option<String> = None;

                        // Use ListItem component for consistent styling with main menu
                        ListItem::new(name.clone(), list_colors)
                            .index(ix)
                            .icon_kind(icon)
                            .description_opt(description)
                            .selected(is_selected)
                            .with_accent_bar(true)
                            .into_any_element()
                    })
                    .collect()
            },
        )
        .track_scroll(&self.list_scroll_handle)
        .flex_1()
        .w_full();

        // Get entity handles for click callbacks
        let handle_select = cx.entity().downgrade();
        let handle_actions = cx.entity().downgrade();

        // Check if actions are currently showing (for CLS-free toggle)
        let show_actions = match self.actions_showing.lock() {
            Ok(guard) => *guard,
            Err(poison) => {
                tracing::error!("path_prompt_actions_showing_mutex_poisoned_in_render");
                *poison.into_inner()
            }
        };

        // Get actions search text from shared state
        let actions_search_text = match self.actions_search_text.lock() {
            Ok(guard) => guard.clone(),
            Err(poison) => {
                tracing::error!("path_prompt_actions_search_text_mutex_poisoned_in_render");
                poison.into_inner().clone()
            }
        };

        // Create path prefix for display in search input
        let path_prefix = self.path_prefix.clone();

        // Create header colors and config using shared components - always use theme
        let header_colors = PromptHeaderColors::from_theme(&self.theme);

        let header_config = PromptHeaderConfig::new()
            .filter_text(self.filter_text.clone())
            .placeholder("Type to filter...")
            .path_prefix(Some(path_prefix))
            .primary_button_label("Select")
            .primary_button_shortcut("↵")
            .show_actions_button(true)
            .cursor_visible(self.cursor_visible)
            .actions_mode(show_actions)
            .actions_search_text(actions_search_text)
            .focused(!show_actions);

        let header = PromptHeader::new(header_config, header_colors)
            .on_primary_click(Box::new(move |_, _window, cx| {
                logging::log("CLICK", "PathPrompt primary button (Select) clicked");
                if let Some(prompt) = handle_select.upgrade() {
                    prompt.update(cx, |this, cx| {
                        this.submit_selected(cx);
                    });
                }
            }))
            .on_actions_click(Box::new(move |_, _window, cx| {
                logging::log("CLICK", "PathPrompt actions button clicked");
                if let Some(prompt) = handle_actions.upgrade() {
                    prompt.update(cx, |this, cx| {
                        this.toggle_actions(cx);
                    });
                }
            }));

        // Create hint text for footer
        let hint_text = self.hint.clone().unwrap_or_else(|| {
            format!("{} items • ↑↓ navigate • ←→ in/out • Enter open • Tab into • ⌘K actions • Esc cancel", filtered_count)
        });

        // Create container colors and config - always use theme
        let container_colors = PromptContainerColors::from_theme(&self.theme);

        let container_config = PromptContainerConfig::new()
            .show_divider(true)
            .hint(hint_text);

        let container = div()
            .id(gpui::ElementId::Name("window:path".into()))
            .w_full()
            .h_full()
            .child(
                PromptContainer::new(container_colors)
                    .config(container_config)
                    .header(header)
                    .content(list),
            );

        FocusablePrompt::new(container)
            .key_context("path_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        let actions_showing = match this.actions_showing.lock() {
                            Ok(guard) => *guard,
                            Err(poison) => {
                                tracing::error!(
                                    "path_prompt_actions_showing_mutex_poisoned_in_app_key_handler"
                                );
                                *poison.into_inner()
                            }
                        };

                        if actions_showing {
                            return false;
                        }

                        logging::log(
                            "PROMPTS",
                            "PathPrompt: Escape key pressed - calling submit_cancel()",
                        );
                        this.submit_cancel();
                        true
                    }
                    FocusablePromptInterceptedKey::CmdK => {
                        this.toggle_actions(cx);
                        true
                    }
                    FocusablePromptInterceptedKey::CmdW => false,
                },
                |this, event, _window, cx| {
                    let key = event.keystroke.key.as_str();

                    // When actions are showing, let the ActionsDialog handle keys in parent routing.
                    let actions_showing = match this.actions_showing.lock() {
                        Ok(guard) => *guard,
                        Err(poison) => {
                            tracing::error!(
                                "path_prompt_actions_showing_mutex_poisoned_in_entity_key_handler"
                            );
                            *poison.into_inner()
                        }
                    };
                    if actions_showing {
                        return;
                    }

                    match key {
                        "up" | "Up" | "ArrowUp" | "arrowup" => this.move_up(cx),
                        "down" | "Down" | "ArrowDown" | "arrowdown" => this.move_down(cx),
                        "left" | "Left" | "ArrowLeft" | "arrowleft" => this.navigate_to_parent(cx),
                        "right" | "Right" | "ArrowRight" | "arrowright" => {
                            this.navigate_into_selected(cx)
                        }
                        "tab" | "Tab" => {
                            if event.keystroke.modifiers.shift {
                                this.navigate_to_parent(cx);
                            } else {
                                this.navigate_into_selected(cx);
                            }
                        }
                        "enter" | "Enter" | "return" | "Return" => this.handle_enter(cx),
                        "backspace" | "Backspace" => this.handle_backspace(cx),
                        _ => {
                            if let Some(ref key_char) = event.keystroke.key_char {
                                if let Some(ch) = key_char.chars().next() {
                                    if !ch.is_control() {
                                        this.handle_char(ch, cx);
                                    }
                                }
                            }
                        }
                    }
                },
            )
    }
}
