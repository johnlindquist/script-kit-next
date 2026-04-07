use super::*;
use crate::ai::context_picker_row::{render_dense_monoline_picker_row_with_accessory, GOLD, HINT};
use crate::components::inline_dropdown::{
    inline_dropdown_clamp_selected_index, inline_dropdown_visible_range, InlineDropdown,
    InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis,
};

impl AiApp {
    pub(super) fn render_presets_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::from_theme(&theme);
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;

        let selected_index =
            inline_dropdown_clamp_selected_index(self.presets_selected_index, self.presets.len());

        let visible = inline_dropdown_visible_range(selected_index, self.presets.len(), 8);

        let synopsis = self.presets.get(selected_index).and_then(|preset| {
            let meta = preset.preferred_model.clone().unwrap_or_default();
            let description = if preset.description.is_empty() {
                "Create a new chat from this preset".to_string()
            } else {
                preset.description.to_string()
            };
            if meta.is_empty() && description.is_empty() {
                None
            } else {
                Some(InlineDropdownSynopsis {
                    label: SharedString::from(preset.name.to_string()),
                    meta: SharedString::from(meta),
                    description: SharedString::from(description),
                })
            }
        });

        let body = div()
            .flex()
            .flex_col()
            .children(
                self.presets
                    .iter()
                    .enumerate()
                    .skip(visible.start)
                    .take(visible.len())
                    .map(|(idx, preset)| {
                        let is_selected = idx == selected_index;

                        let accessory = svg()
                            .external_path(preset.icon.external_path())
                            .size(px(14.))
                            .text_color(if is_selected {
                                GOLD
                            } else {
                                muted_fg.opacity(HINT)
                            })
                            .into_any_element();

                        render_dense_monoline_picker_row_with_accessory(
                            SharedString::from(format!("preset-{idx}")),
                            SharedString::from(preset.name.to_string()),
                            SharedString::default(),
                            &[],
                            &[],
                            is_selected,
                            fg,
                            muted_fg,
                            Some(accessory),
                        )
                        .cursor_pointer()
                        .on_click(cx.listener(
                            move |this, _, window, cx| {
                                this.presets_selected_index = idx;
                                this.confirm_presets_selection(window, cx);
                            },
                        ))
                    }),
            )
            .into_any_element();

        let dropdown = InlineDropdown::new(SharedString::from("presets-dropdown"), body, colors)
            .empty_state_opt(self.presets.is_empty().then(|| InlineDropdownEmptyState {
                message: SharedString::from("No presets saved"),
                hints: Vec::new(),
            }))
            .synopsis(synopsis);

        let overlay_bg = Self::get_modal_overlay_background();

        div()
            .id("presets-dropdown-overlay")
            .absolute()
            .inset_0()
            .bg(overlay_bg)
            .flex()
            .items_start()
            .justify_start()
            .pt(S9)
            .pl(S4)
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_presets_dropdown(cx);
            }))
            .child(
                div()
                    .id("presets-dropdown-container")
                    .w(px(320.0))
                    .on_click(cx.listener(|_, _, _, _| {}))
                    .child(dropdown),
            )
    }
}
