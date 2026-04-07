use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::theme::AppChromeColors;
use crate::ui_foundation::{
    is_key_backspace, is_key_down, is_key_enter, is_key_space, is_key_up, printable_char,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SelectRowState {
    pub is_focused: bool,
    pub is_selected: bool,
    pub is_hovered: bool,
}

pub(super) fn compute_row_state(
    display_idx: usize,
    focused_index: usize,
    choice_idx: usize,
    selected: &HashSet<usize>,
    hovered_index: Option<usize>,
) -> SelectRowState {
    SelectRowState {
        is_focused: display_idx == focused_index,
        is_selected: selected.contains(&choice_idx),
        is_hovered: hovered_index == Some(display_idx),
    }
}

pub(super) fn visual_row_state_for_input_modality(
    row_state: SelectRowState,
    last_input_was_keyboard: bool,
) -> SelectRowState {
    if last_input_was_keyboard {
        SelectRowState {
            is_hovered: false,
            ..row_state
        }
    } else {
        row_state
    }
}

pub(super) fn extract_choice_icon_hint(description: Option<&str>) -> Option<&str> {
    description.and_then(|raw| {
        raw.split(['•', '|', '\n'])
            .map(str::trim)
            .find_map(|token| {
                let token_lower = token.to_ascii_lowercase();
                if token_lower == "icon"
                    || token_lower.starts_with("icon:")
                    || token_lower.starts_with("icon=")
                    || token_lower.starts_with("icon ")
                {
                    token
                        .split_once(':')
                        .or_else(|| token.split_once('='))
                        .map(|(_, value)| value.trim())
                        .or_else(|| token.split_whitespace().nth(1))
                } else {
                    None
                }
            })
            .filter(|value| !value.is_empty())
    })
}

pub(super) fn icon_kind_from_choice(choice: &Choice) -> IconKind {
    let metadata_icon =
        extract_choice_icon_hint(choice.description.as_deref()).and_then(IconKind::from_icon_hint);
    let name_prefix_icon = choice
        .name
        .split_whitespace()
        .next()
        .and_then(IconKind::from_icon_hint);

    metadata_icon
        .or(name_prefix_icon)
        .unwrap_or_else(|| IconKind::Svg("Code".to_string()))
}

fn leading_content_from_icon_kind(icon_kind: IconKind) -> LeadingContent {
    match icon_kind {
        IconKind::Emoji(emoji) => LeadingContent::Emoji(emoji.into()),
        IconKind::Image(render_image) => LeadingContent::AppIcon(render_image),
        IconKind::Svg(name) => LeadingContent::Icon {
            name: SharedString::from(name),
            color: None,
        },
    }
}

impl Focusable for SelectPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = AppChromeColors::from_theme(&self.theme);
        let tokens = get_tokens(self.design_variant);
        let spacing = tokens.spacing();

        let text_color = rgb(chrome.text_secondary_hex);
        let muted_color = rgb(chrome.text_muted_hex);

        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| "Search...".to_string());

        let input_display = if self.filter_text.is_empty() {
            SharedString::from(placeholder)
        } else {
            SharedString::from(self.filter_text.clone())
        };

        // Search input — minimal chrome: no bg, no border, scaffold owns padding
        let header = div()
            .id(gpui::ElementId::Name("input:select-filter".into()))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .child(
                div()
                    .flex_1()
                    .text_size(px(16.0))
                    .text_color(if self.filter_text.is_empty() {
                        muted_color
                    } else {
                        text_color
                    })
                    .child(input_display),
            )
            .when(self.multiple, |container| {
                container.child(
                    div()
                        .text_xs()
                        .text_color(muted_color)
                        .child(format!("{} selected", self.selected.len())),
                )
            });

        // Choices list
        let filtered_len = self.filtered_choices.len();
        let choices_content: AnyElement = if filtered_len == 0 {
            let empty_message = if self.filter_text.trim().is_empty() {
                "No choices available"
            } else {
                "No choices match your filter"
            };
            div()
                .w_full()
                .py(px(spacing.padding_xl))
                .px(px(spacing.item_padding_x))
                .text_color(muted_color)
                .child(empty_message)
                .into_any_element()
        } else {
            uniform_list(
                "select-choices",
                filtered_len,
                cx.processor(
                    move |this: &mut SelectPrompt,
                          visible_range: std::ops::Range<usize>,
                          window,
                          cx| {
                        let item_colors = UnifiedListItemColors::from_theme(&this.theme);
                        let last_input_was_keyboard = window.last_input_was_keyboard();
                        let mut rows = Vec::with_capacity(visible_range.len());

                        for display_idx in visible_range {
                            if let Some(&choice_idx) = this.filtered_choices.get(display_idx) {
                                if let Some(choice) = this.choices.get(choice_idx) {
                                    if let Some(indexed_choice) = this.choice_index.get(choice_idx)
                                    {
                                        let row_state = compute_row_state(
                                            display_idx,
                                            this.focused_index,
                                            choice_idx,
                                            &this.selected,
                                            this.hovered_index,
                                        );
                                        let visual_row_state = visual_row_state_for_input_modality(
                                            row_state,
                                            last_input_was_keyboard,
                                        );
                                        let is_focused = visual_row_state.is_focused;
                                        let is_selected = visual_row_state.is_selected;
                                        let is_hovered = visual_row_state.is_hovered;
                                        let semantic_id =
                                            choice.semantic_id.clone().unwrap_or_else(|| {
                                                indexed_choice.stable_semantic_id.clone()
                                            });
                                        let leading = if this.multiple {
                                            Some(LeadingContent::Emoji(
                                                choice_selection_indicator(true, is_selected)
                                                    .into(),
                                            ))
                                        } else {
                                            Some(leading_content_from_icon_kind(
                                                icon_kind_from_choice(choice),
                                            ))
                                        };
                                        let subtitle = if is_focused {
                                            indexed_choice
                                                .metadata
                                                .subtitle_text()
                                                .map(TextContent::plain)
                                        } else {
                                            None
                                        };
                                        let title = highlighted_choice_title(
                                            &choice.name,
                                            &this.filter_text,
                                        );
                                        let trailing =
                                            indexed_choice.metadata.shortcut.clone().map(
                                                |shortcut| {
                                                    TrailingContent::Shortcut(SharedString::from(
                                                        shortcut,
                                                    ))
                                                },
                                            );
                                        let hover_handler = cx.listener(
                                            move |this: &mut SelectPrompt,
                                                  hovered: &bool,
                                                  _window,
                                                  cx| {
                                                this.set_hover_from_mouse(
                                                    display_idx,
                                                    *hovered,
                                                    cx,
                                                );
                                            },
                                        );
                                        let click_handler = cx.listener(
                                            move |this: &mut SelectPrompt,
                                                  event: &gpui::ClickEvent,
                                                  window,
                                                  cx| {
                                                this.focus_handle.focus(window, cx);
                                                let click_count = event.click_count();
                                                this.activate_row_from_mouse(
                                                    display_idx,
                                                    click_count,
                                                    cx,
                                                );
                                            },
                                        );

                                        // UnifiedListItem owns row chrome. Keep the ghost
                                        // selected background for focused or selected rows,
                                        // but reserve the accent bar for the focused row so
                                        // multi-select does not imply active focus everywhere.
                                        let row = div()
                                            .id(display_idx)
                                            .w_full()
                                            .cursor_pointer()
                                            .on_hover(hover_handler)
                                            .on_click(click_handler)
                                            .child(
                                                UnifiedListItem::new(
                                                    gpui::ElementId::Name(semantic_id.into()),
                                                    title,
                                                )
                                                .subtitle_opt(subtitle)
                                                .leading_opt(leading)
                                                .trailing_opt(trailing)
                                                .state(ItemState {
                                                    is_selected: is_focused || is_selected,
                                                    is_hovered,
                                                    is_disabled: false,
                                                })
                                                .density(Density::Comfortable)
                                                .with_accent_bar(is_focused)
                                                .with_hover_effect(false)
                                                .colors(item_colors),
                                            );

                                        rows.push(row);
                                    }
                                }
                            }
                        }

                        rows
                    },
                ),
            )
            .h_full()
            .w_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };
        let content = div()
            .id(gpui::ElementId::Name("list:select-choices".into()))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .child(choices_content);

        let hints = crate::components::universal_prompt_hints();
        crate::components::emit_prompt_hint_audit("prompts::select", &hints);

        let container =
            crate::components::render_minimal_list_prompt_scaffold(header, content, hints, None)
                .id(gpui::ElementId::Name("window:select".into()))
                .text_color(text_color);

        FocusablePrompt::new(container)
            .key_context("select_prompt")
            .focus_handle(self.focus_handle.clone())
            .build(
                window,
                cx,
                |this, intercepted_key, _event, _window, _cx| match intercepted_key {
                    FocusablePromptInterceptedKey::Escape => {
                        this.submit_cancel();
                        true
                    }
                    _ => false,
                },
                |this, event, _window, cx| {
                    let key_str = event.keystroke.key.as_str();
                    let has_ctrl = event.keystroke.modifiers.platform; // Cmd on macOS, Ctrl on others
                    let is_up = is_key_up(key_str);
                    let is_down = is_key_down(key_str);
                    let is_space = is_key_space(key_str);
                    let is_enter = is_key_enter(key_str);
                    let is_backspace = is_key_backspace(key_str);

                    // Handle Ctrl/Cmd+A for select all
                    if has_ctrl && key_str.eq_ignore_ascii_case("a") {
                        this.toggle_select_all_filtered(cx);
                        return;
                    }

                    if is_up {
                        this.move_up(cx);
                    } else if is_down {
                        this.move_down(cx);
                    } else if is_space {
                        if has_ctrl {
                            this.toggle_selection(cx);
                        } else {
                            this.handle_char(' ', cx);
                        }
                    } else if is_enter {
                        this.submit();
                    } else if is_backspace {
                        this.handle_backspace(cx);
                    } else if let Some(ch) = printable_char(event.keystroke.key_char.as_deref()) {
                        if should_append_to_filter(ch) {
                            this.handle_char(ch, cx);
                        }
                    }
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{visual_row_state_for_input_modality, SelectRowState};

    /// Row background resolution — test-only helper retained for unit test coverage.
    fn resolve_row_bg_rgba(
        row_state: SelectRowState,
        focused_bg_rgba: u32,
        hovered_bg_rgba: u32,
    ) -> u32 {
        if row_state.is_focused || row_state.is_selected {
            focused_bg_rgba
        } else if row_state.is_hovered {
            hovered_bg_rgba
        } else {
            0x00000000
        }
    }

    #[test]
    fn test_resolve_row_bg_rgba_uses_focused_accent_for_selected_row() {
        let focused_bg_hex = 0x55AAFF3A;
        let hovered_bg_hex = 0x55AAFF26;

        let selected_row = SelectRowState {
            is_focused: false,
            is_selected: true,
            is_hovered: false,
        };

        assert_eq!(
            resolve_row_bg_rgba(selected_row, focused_bg_hex, hovered_bg_hex),
            focused_bg_hex
        );
    }

    #[test]
    fn test_resolve_row_bg_rgba_uses_hover_color_for_unselected_hovered_row() {
        let focused_bg_hex = 0x55AAFF3A;
        let hovered_bg_hex = 0x55AAFF26;

        let hovered_row = SelectRowState {
            is_focused: false,
            is_selected: false,
            is_hovered: true,
        };

        assert_eq!(
            resolve_row_bg_rgba(hovered_row, focused_bg_hex, hovered_bg_hex),
            hovered_bg_hex
        );
    }

    #[test]
    fn select_prompt_uses_shared_minimal_list_prompt_scaffold() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("render_minimal_list_prompt_scaffold("),
            "select prompt should use the shared minimal list prompt scaffold"
        );
    }

    #[test]
    fn select_prompt_does_not_use_prompt_footer() {
        let source = include_str!("render.rs");
        let needle = ["PromptFooter", "::new("].concat();
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];
        assert!(
            !render_code.contains(&needle),
            "select prompt render code should not use PromptFooter"
        );
    }

    #[test]
    fn visual_row_state_suppresses_hover_in_keyboard_modality() {
        let row_state = SelectRowState {
            is_focused: false,
            is_selected: false,
            is_hovered: true,
        };
        let visual = visual_row_state_for_input_modality(row_state, true);
        assert_eq!(
            visual,
            SelectRowState {
                is_focused: false,
                is_selected: false,
                is_hovered: false,
            }
        );
    }

    #[test]
    fn visual_row_state_preserves_focus_and_selection_when_hover_is_suppressed() {
        let row_state = SelectRowState {
            is_focused: true,
            is_selected: true,
            is_hovered: true,
        };
        let visual = visual_row_state_for_input_modality(row_state, true);
        assert_eq!(
            visual,
            SelectRowState {
                is_focused: true,
                is_selected: true,
                is_hovered: false,
            }
        );
    }

    #[test]
    fn visual_row_state_keeps_hover_in_mouse_modality() {
        let row_state = SelectRowState {
            is_focused: false,
            is_selected: false,
            is_hovered: true,
        };
        let visual = visual_row_state_for_input_modality(row_state, false);
        assert_eq!(visual, row_state);
    }

    #[test]
    fn select_render_drops_local_row_alpha_constants() {
        const SOURCE: &str = include_str!("render.rs");
        let render_fn_end = SOURCE.find("#[cfg(test)]").unwrap_or(SOURCE.len());
        let render_code = &SOURCE[..render_fn_end];
        assert!(
            !render_code.contains("ROW_FOCUSED_BG_ALPHA"),
            "select render should not define ROW_FOCUSED_BG_ALPHA"
        );
        assert!(
            !render_code.contains("ROW_HOVER_BG_ALPHA"),
            "select render should not define ROW_HOVER_BG_ALPHA"
        );
    }
}
