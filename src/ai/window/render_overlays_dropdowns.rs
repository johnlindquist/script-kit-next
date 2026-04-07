use super::*;
use crate::components::inline_dropdown::{
    inline_dropdown_clamp_selected_index, inline_dropdown_visible_range,
    render_dense_monoline_picker_row_with_leading_visual, InlineDropdown, InlineDropdownColors,
    InlineDropdownEmptyState, InlineDropdownSynopsis, GOLD, HINT,
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

        tracing::info!(
            target: "ai",
            event = "ai_presets_dropdown_rendered",
            preset_count = self.presets.len(),
            selected_index,
            visible_start = visible.start,
            visible_end = visible.end,
            has_synopsis = synopsis.is_some(),
            "Rendered AI presets inline dropdown"
        );

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
                        let model_meta = preset.preferred_model.clone().unwrap_or_default();

                        let leading_visual = div()
                            .w(px(14.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                svg()
                                    .external_path(preset.icon.external_path())
                                    .size(px(14.0))
                                    .text_color(if is_selected {
                                        GOLD
                                    } else {
                                        muted_fg.opacity(HINT)
                                    }),
                            )
                            .into_any_element();

                        render_dense_monoline_picker_row_with_leading_visual(
                            SharedString::from(format!("preset-{idx}")),
                            SharedString::from(preset.name.to_string()),
                            SharedString::from(model_meta),
                            &[],
                            &[],
                            is_selected,
                            fg,
                            muted_fg,
                            leading_visual,
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

        div()
            .id("presets-dropdown-overlay")
            .absolute()
            .inset_0()
            .bg(gpui::transparent_black())
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
                    .on_click(cx.listener(|_, _, _, _| {}))
                    .w(px(320.0))
                    .child(dropdown),
            )
    }
}
