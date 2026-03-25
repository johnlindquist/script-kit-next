use super::*;
use crate::theme::opacity::{OPACITY_SELECTED, OPACITY_STRONG};

impl AiApp {
    pub(super) fn render_presets_dropdown(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let accent_fg = theme.accent_foreground;
        let fg = theme.foreground;

        // Build preset items
        let preset_items: Vec<_> = self
            .presets
            .iter()
            .enumerate()
            .map(|(idx, preset)| {
                let is_selected = idx == self.presets_selected_index;
                let icon = preset.icon;
                let name = preset.name.to_string();
                let description = preset.description.to_string();

                div()
                    .id(SharedString::from(format!("preset-{}", idx)))
                    .px(S3)
                    .py(S2)
                    .mx(S1)
                    .rounded(R_MD)
                    .flex()
                    .items_center()
                    .gap(S3)
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(accent))
                    .when(!is_selected, |el| {
                        el.hover(|el| el.bg(accent.opacity(OPACITY_SELECTED)))
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.presets_selected_index = idx;
                        this.create_chat_with_preset(window, cx);
                    }))
                    // Icon
                    .child(
                        svg()
                            .external_path(icon.external_path())
                            .size(ICON_MD)
                            .text_color(if is_selected { accent_fg } else { muted_fg }),
                    )
                    // Name and description
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_selected { accent_fg } else { fg })
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_selected {
                                        accent_fg.opacity(OPACITY_STRONG)
                                    } else {
                                        muted_fg
                                    })
                                    .child(description),
                            ),
                    )
            })
            .collect();

        // Overlay positioned near the new chat button
        // Theme-aware modal overlay: black for dark mode, white for light mode
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
                    .w(px(300.0))
                    .max_h(px(350.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded(R_LG)
                    // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px(S3)
                            .py(S2)
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("New Chat with Preset"),
                    )
                    // Preset list
                    .child(
                        div()
                            .id("preset-list")
                            .flex_1()
                            .overflow_y_scrollbar()
                            .p(S1)
                            .children(preset_items),
                    )
                    // Footer hint
                    .child(
                        div()
                            .px(S3)
                            .py(S2)
                            .border_t_1()
                            .border_color(border_color)
                            .text_xs()
                            .text_color(muted_fg)
                            .child("Select a preset to start a new chat"),
                    ),
            )
    }
}
