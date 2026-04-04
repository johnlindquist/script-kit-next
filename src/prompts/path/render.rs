use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::ui_foundation::{
    is_key_backspace, is_key_down, is_key_enter, is_key_left, is_key_right, is_key_tab, is_key_up,
    printable_char,
};

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

        // Text colors from theme
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;

        // Minimal chrome header: path prefix (muted) + filter text (primary), no buttons
        let path_prefix = self.path_prefix.clone();
        let filter_text = self.filter_text.clone();
        let filter_is_empty = filter_text.is_empty();

        let header = div()
            .id(gpui::ElementId::Name("input:path-filter".into()))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(gpui::px(8.0))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .text_size(gpui::px(16.0))
                    .overflow_x_hidden()
                    .child(
                        div()
                            .text_color(gpui::rgba((text_muted << 8) | 0xCC))
                            .flex_shrink_0()
                            .max_w(gpui::px(200.0))
                            .overflow_x_hidden()
                            .child(gpui::SharedString::from(path_prefix)),
                    )
                    .child(
                        div()
                            .text_color(if filter_is_empty {
                                gpui::rgb(text_muted)
                            } else {
                                gpui::rgb(text_primary)
                            })
                            .child(if filter_is_empty {
                                gpui::SharedString::from("Type to filter...")
                            } else {
                                gpui::SharedString::from(filter_text)
                            }),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(gpui::rgba((text_muted << 8) | 0x99))
                    .child(format!("{filtered_count} items")),
            );

        // Content wrapper
        let content = div()
            .id(gpui::ElementId::Name("list:path-entries".into()))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .px(gpui::px(8.0))
            .child(list);

        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("prompts::path", &hints);

        let container =
            crate::components::render_minimal_list_prompt_scaffold(header, content, hints, None)
                .id(gpui::ElementId::Name("window:path".into()))
                .text_color(gpui::rgb(text_primary));

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

                    if is_key_up(key) {
                        this.move_up(cx);
                    } else if is_key_down(key) {
                        this.move_down(cx);
                    } else if is_key_left(key) {
                        this.navigate_to_parent(cx);
                    } else if is_key_right(key) {
                        this.navigate_into_selected(cx);
                    } else if is_key_tab(key) {
                        if event.keystroke.modifiers.shift {
                            this.navigate_to_parent(cx);
                        } else {
                            this.navigate_into_selected(cx);
                        }
                    } else if is_key_enter(key) {
                        this.handle_enter(cx);
                    } else if is_key_backspace(key) {
                        this.handle_backspace(cx);
                    } else if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
                        this.handle_char(ch, cx);
                    }
                },
            )
    }
}
