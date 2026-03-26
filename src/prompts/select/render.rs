use super::*;
use crate::components::{FocusablePrompt, FocusablePromptInterceptedKey};
use crate::ui_foundation::{
    is_key_backspace, is_key_down, is_key_enter, is_key_space, is_key_up, printable_char,
};

const ROW_FOCUSED_BG_ALPHA: u32 = 0x3A;
const ROW_HOVER_BG_ALPHA: u32 = 0x26;

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

pub(super) fn resolve_row_bg_hex(
    row_state: SelectRowState,
    focused_bg_hex: u32,
    hovered_bg_hex: u32,
) -> u32 {
    if row_state.is_focused || row_state.is_selected {
        focused_bg_hex
    } else if row_state.is_hovered {
        hovered_bg_hex
    } else {
        0x00000000
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
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let (text_color, muted_color) = if self.design_variant == DesignVariant::Default {
            (
                rgb(self.theme.colors.text.secondary),
                rgb(self.theme.colors.text.muted),
            )
        } else {
            (rgb(colors.text_secondary), rgb(colors.text_muted))
        };
        let row_accent_color = if self.design_variant == DesignVariant::Default {
            self.theme.colors.accent.selected
        } else {
            colors.accent
        };
        let focused_row_bg_hex = (row_accent_color << 8) | ROW_FOCUSED_BG_ALPHA;
        let hovered_row_bg_hex = (row_accent_color << 8) | ROW_HOVER_BG_ALPHA;
        let hovered_row_bg = rgba(hovered_row_bg_hex);

        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| "Search...".to_string());

        let input_display = if self.filter_text.is_empty() {
            SharedString::from(placeholder)
        } else {
            SharedString::from(self.filter_text.clone())
        };

        // Search input — minimal chrome: no bg, no border, chrome-token padding
        let input_container = div()
            .id(gpui::ElementId::Name("input:select-filter".into()))
            .w_full()
            .px(px(crate::ui::chrome::HEADER_PADDING_X))
            .py(px(crate::ui::chrome::HEADER_PADDING_Y))
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
                          _window,
                          cx| {
                        let item_colors = UnifiedListItemColors {
                            selected_opacity: 0.0,
                            hover_opacity: 0.0,
                            ..UnifiedListItemColors::from_theme(&this.theme)
                        };
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
                                        let is_focused = row_state.is_focused;
                                        let is_selected = row_state.is_selected;
                                        let is_hovered = row_state.is_hovered;
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
                                        let subtitle = indexed_choice
                                            .metadata
                                            .subtitle_text()
                                            .map(TextContent::plain);
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
                                        let row_bg = rgba(resolve_row_bg_hex(
                                            row_state,
                                            focused_row_bg_hex,
                                            hovered_row_bg_hex,
                                        ));
                                        let hover_handler = cx.listener(
                                            move |this: &mut SelectPrompt,
                                                  hovered: &bool,
                                                  _window,
                                                  cx| {
                                                if *hovered {
                                                    if this.hovered_index != Some(display_idx) {
                                                        this.hovered_index = Some(display_idx);
                                                        cx.notify();
                                                    }
                                                } else if this.hovered_index == Some(display_idx) {
                                                    this.hovered_index = None;
                                                    cx.notify();
                                                }
                                            },
                                        );

                                        let mut row = div()
                                            .id(display_idx)
                                            .w_full()
                                            .h(px(LIST_ITEM_HEIGHT))
                                            .rounded(px(8.0))
                                            .bg(row_bg)
                                            .cursor_pointer()
                                            .on_hover(hover_handler)
                                            .child(
                                                UnifiedListItem::new(
                                                    gpui::ElementId::Name(semantic_id.into()),
                                                    title,
                                                )
                                                .subtitle_opt(subtitle)
                                                .leading_opt(leading)
                                                .trailing_opt(trailing)
                                                .state(ItemState {
                                                    is_selected,
                                                    is_hovered,
                                                    is_disabled: false,
                                                })
                                                .density(Density::Comfortable)
                                                .colors(item_colors),
                                            );

                                        if !is_focused && !is_selected {
                                            row = row.hover(move |s| s.bg(hovered_row_bg));
                                        }

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

        let choices_container = div()
            .id(gpui::ElementId::Name("list:select-choices".into()))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .px(px(8.0))
            .child(choices_content);

        let hints: Vec<SharedString> = if self.multiple {
            vec![
                SharedString::from("↵ Select"),
                SharedString::from("⌘Space Toggle"),
                SharedString::from("Esc Back"),
            ]
        } else {
            vec![
                SharedString::from("↵ Select"),
                SharedString::from("Esc Back"),
            ]
        };

        let container = div()
            .id(gpui::ElementId::Name("window:select".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .text_color(text_color)
            .child(input_container)
            .child(crate::components::SectionDivider::new())
            .child(choices_container)
            .child(crate::components::render_simple_hint_strip(hints, None));

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
    use super::{resolve_row_bg_hex, SelectRowState};

    #[test]
    fn test_resolve_row_bg_hex_uses_focused_accent_for_selected_row() {
        let focused_bg_hex = 0x55AAFF3A;
        let hovered_bg_hex = 0x55AAFF26;

        let selected_row = SelectRowState {
            is_focused: false,
            is_selected: true,
            is_hovered: false,
        };

        assert_eq!(
            resolve_row_bg_hex(selected_row, focused_bg_hex, hovered_bg_hex),
            focused_bg_hex
        );
    }

    #[test]
    fn test_resolve_row_bg_hex_uses_hover_color_for_unselected_hovered_row() {
        let focused_bg_hex = 0x55AAFF3A;
        let hovered_bg_hex = 0x55AAFF26;

        let hovered_row = SelectRowState {
            is_focused: false,
            is_selected: false,
            is_hovered: true,
        };

        assert_eq!(
            resolve_row_bg_hex(hovered_row, focused_bg_hex, hovered_bg_hex),
            hovered_bg_hex
        );
    }

    #[test]
    fn select_prompt_render_uses_chrome_token_padding() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("crate::ui::chrome::HEADER_PADDING_X"),
            "select input should use chrome-token HEADER_PADDING_X"
        );
        assert!(
            source.contains("crate::ui::chrome::HEADER_PADDING_Y"),
            "select input should use chrome-token HEADER_PADDING_Y"
        );
    }

    #[test]
    fn select_prompt_container_uses_section_divider() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("SectionDivider::new()"),
            "select container should use SectionDivider between input and list"
        );
    }

    #[test]
    fn select_prompt_uses_hint_strip_footer() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("render_simple_hint_strip("),
            "select prompt should render a minimal hint strip footer"
        );
        // Verify no PromptFooter usage (split string to avoid self-match)
        let needle = ["PromptFooter", "::new("].concat();
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];
        assert!(
            !render_code.contains(&needle),
            "select prompt render code should not use PromptFooter"
        );
    }
}
