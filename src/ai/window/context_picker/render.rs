use super::super::*;
use super::types::{ContextPickerItemKind, ContextPickerSection};
use crate::theme::opacity::{OPACITY_BORDER, OPACITY_DISABLED, OPACITY_SELECTED, OPACITY_TEXT_MUTED};

/// Layout constants for the context picker overlay.
const PICKER_MAX_H: f32 = 260.0;

impl AiApp {
    /// Render the inline context picker overlay.
    ///
    /// Returns an empty element when the picker is closed.
    pub(in crate::ai::window) fn render_context_picker(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = match &self.context_picker {
            Some(s) if !s.items.is_empty() => s,
            _ => return div().id("context-picker-empty").into_any_element(),
        };

        let accent = cx.theme().accent;
        let muted_fg = cx.theme().muted_foreground;

        let mut rows: Vec<gpui::AnyElement> = Vec::new();
        let mut current_section: Option<ContextPickerSection> = None;

        for (idx, item) in state.items.iter().enumerate() {
            let section = match &item.kind {
                ContextPickerItemKind::BuiltIn(_) => ContextPickerSection::BuiltIn,
                ContextPickerItemKind::File(_) => ContextPickerSection::Files,
                ContextPickerItemKind::Folder(_) => ContextPickerSection::Folders,
            };

            if current_section != Some(section) {
                current_section = Some(section);
                let header: SharedString = section.label().into();
                rows.push(
                    div()
                        .px(S3)
                        .pt(S2)
                        .pb(S1)
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(muted_fg.opacity(OPACITY_TEXT_MUTED))
                        .child(header)
                        .into_any_element(),
                );
            }

            let is_selected = idx == state.selected_index;
            let bg = if is_selected {
                accent.opacity(OPACITY_SELECTED)
            } else {
                gpui::transparent_black()
            };

            let icon_name = match &item.kind {
                ContextPickerItemKind::BuiltIn(_) => LocalIconName::Code,
                ContextPickerItemKind::File(_) => LocalIconName::File,
                ContextPickerItemKind::Folder(_) => LocalIconName::Folder,
            };

            let label: SharedString = item.label.clone();
            let subtitle: SharedString = item.subtitle.clone();

            rows.push(
                div()
                    .id(SharedString::from(format!("ctx-picker-{}", idx)))
                    .flex()
                    .items_center()
                    .gap(S2)
                    .px(S3)
                    .py(S1)
                    .rounded(R_MD)
                    .bg(bg)
                    .cursor_pointer()
                    .hover(|el| el.bg(accent.opacity(OPACITY_DISABLED)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(picker) = this.context_picker.as_mut() {
                            picker.selected_index = idx;
                        }
                        this.accept_context_picker_selection(window, cx);
                    }))
                    .child(
                        svg()
                            .external_path(icon_name.external_path())
                            .size(ICON_XS)
                            .text_color(accent),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .text_ellipsis()
                                    .child(label),
                            )
                            .when(!subtitle.is_empty(), |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(muted_fg)
                                        .text_ellipsis()
                                        .child(subtitle),
                                )
                            }),
                    )
                    .into_any_element(),
            );
        }

        div()
            .id("context-picker-overlay")
            .w_full()
            .max_h(px(PICKER_MAX_H))
            .overflow_y_scroll()
            .rounded(R_LG)
            .border_1()
            .border_color(cx.theme().border.opacity(OPACITY_BORDER))
            .bg(cx.theme().background)
            .py(S1)
            .children(rows)
            .into_any_element()
    }
}
