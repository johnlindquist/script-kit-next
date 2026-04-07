use super::super::*;
use super::types::ContextPickerItemKind;
use crate::ai::context_picker_row::{render_dense_monoline_picker_row, GHOST, HINT};
use crate::components::inline_dropdown::{
    InlineDropdown, InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis,
};
use crate::list_item::FONT_MONO;

impl AiApp {
    /// Render the inline context picker overlay.
    ///
    /// V05 Dense Monoline: text_xs, 3px pad, ghost bg on selected,
    /// gold bar 2x12, FONT_MONO /command at 0.30, vibrancy shell.
    pub(in crate::ai::window) fn render_context_picker(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = match &self.context_picker {
            Some(s) => s,
            None => return div().id("context-picker-empty").into_any_element(),
        };

        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::from_theme(&theme);
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;

        let items = state.items.clone();
        let selected_index = state.selected_index;
        let selected_item = items.get(selected_index).cloned();
        let entity = cx.entity().clone();

        let empty_state = state.items.is_empty().then(|| {
            let hints = super::empty_state_hints(state.trigger);

            tracing::info!(
                target: "ai",
                trigger = ?state.trigger,
                query = %state.query,
                hint_count = hints.len(),
                "ai_context_picker_empty_state"
            );

            let mut chips: Vec<gpui::AnyElement> = Vec::new();
            for hint in hints.iter() {
                let hint_display = SharedString::from(hint.display);
                let hint_display_for_click = hint_display.clone();
                let hint_insertion = hint.insertion.to_string();
                let hint_insertion_for_click = hint_insertion.clone();
                chips.push(
                    div()
                        .id(SharedString::from(format!("hint-{}", hint.display)))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(px(4.))
                        .bg(fg.opacity(GHOST))
                        .hover(|el| el.bg(fg.opacity(0.08)))
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            tracing::info!(
                                target: "ai",
                                display = %hint_display_for_click,
                                insertion = %hint_insertion_for_click,
                                "ai_context_picker_empty_hint_applied"
                            );
                            this.set_composer_value(hint_insertion_for_click.clone(), window, cx);
                        }))
                        .child(
                            div()
                                .text_xs()
                                .font_family(FONT_MONO)
                                .text_color(muted_fg.opacity(HINT))
                                .child(hint_display),
                        )
                        .into_any_element(),
                );
            }

            InlineDropdownEmptyState {
                message: SharedString::from(match state.trigger {
                    super::types::ContextPickerTrigger::Slash => "No matching commands",
                    super::types::ContextPickerTrigger::Mention => "No matching context",
                }),
                hints: chips,
            }
        });

        let picker_list = list(
            self.context_picker_list_state.clone(),
            move |ix, _window, _cx| {
                let item = match items.get(ix) {
                    Some(i) => i,
                    None => return div().into_any_element(),
                };
                let is_selected = ix == selected_index;
                let label: SharedString = item.label.clone();
                let meta: SharedString = item.meta.clone();

                let entity_click = entity.clone();

                render_dense_monoline_picker_row(
                    SharedString::from(format!("ctx-picker-{}", ix)),
                    label,
                    meta,
                    &item.label_highlight_indices,
                    &item.meta_highlight_indices,
                    is_selected,
                    fg,
                    muted_fg,
                )
                .cursor_pointer()
                .on_click(move |_, window, cx| {
                    entity_click.update(cx, |this, cx| {
                        if let Some(picker) = this.context_picker.as_mut() {
                            picker.selected_index = ix;
                        }
                        this.accept_context_picker_selection(window, cx);
                    });
                })
                .into_any_element()
            },
        )
        .with_sizing_behavior(ListSizingBehavior::Infer)
        .max_h(px(260.))
        .min_h(px(0.))
        .into_any_element();

        let synopsis = selected_item
            .filter(|item| !item.description.is_empty())
            .map(|item| InlineDropdownSynopsis {
                label: item.label.clone(),
                meta: item.meta.clone(),
                description: item.description.clone(),
            });

        tracing::info!(
            target: "ai",
            trigger = ?state.trigger,
            query = %state.query,
            item_count = state.items.len(),
            selected_index = state.selected_index,
            "inline_dropdown_inline_context_rendered"
        );

        InlineDropdown::new(
            SharedString::from("context-picker-overlay"),
            picker_list,
            colors,
        )
        .empty_state_opt(empty_state)
        .synopsis(synopsis)
        .vertical_padding(4.0)
        .into_any_element()
    }
}
