use super::*;
use crate::components::inline_dropdown::{
    inline_dropdown_clamp_selected_index, inline_dropdown_visible_range, InlineDropdown,
    InlineDropdownColors, InlineDropdownEmptyState, InlineDropdownSynopsis,
};

impl AiApp {
    pub(super) fn render_presets_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let colors = InlineDropdownColors::from_theme(&theme);
        let list_item_colors = crate::list_item::ListItemColors::from_theme(&theme);
        let main_menu_theme = crate::designs::current_main_menu_theme();
        let row_height = crate::list_item::effective_list_item_height_for_theme(main_menu_theme);

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
                        let source_hint = (!model_meta.is_empty()).then_some(model_meta);
                        let description = if preset.description.is_empty() {
                            Some("Create a new chat from this preset".to_string())
                        } else {
                            Some(preset.description.to_string())
                        };

                        let row = crate::list_item::ListItem::new(
                            SharedString::from(preset.name.to_string()),
                            list_item_colors.clone(),
                        )
                        .index(idx)
                        .selected(is_selected)
                        .main_menu_theme(main_menu_theme)
                        .semantic_id(format!("preset-{idx}"))
                        .description_opt(description)
                        .source_hint_opt(source_hint)
                        .icon_kind_opt(Some(crate::list_item::IconKind::Svg(
                            preset.icon.external_path().to_string(),
                        )))
                        .type_accessory_opt(Some(
                            crate::list_item::TypeAccessory {
                                label: "Preset",
                                icon_name: "sparkles",
                            },
                        ));

                        div()
                            .id(SharedString::from(format!("preset-{idx}")))
                            .w_full()
                            .h(px(row_height))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.presets_selected_index = idx;
                                this.confirm_presets_selection(window, cx);
                            }))
                            .child(row)
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
