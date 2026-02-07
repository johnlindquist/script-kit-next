use super::*;

impl AiApp {
    pub(super) fn render_attachments_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let bg_color = theme.background;
        let border_color = theme.border;
        let muted_fg = theme.muted_foreground;
        let accent = theme.accent;
        let fg = theme.foreground;

        // Attachment options
        let options = [
            ("file", "Add File", LocalIconName::File, "Browse for a file"),
            ("image", "Add Image", LocalIconName::File, "Add an image"),
            (
                "clipboard",
                "Paste from Clipboard",
                LocalIconName::Copy,
                "⌘V",
            ),
        ];

        let option_items: Vec<_> = options
            .iter()
            .map(|(id, name, icon, hint)| {
                let id_str = *id;
                let name_str = name.to_string();
                let icon_name = *icon;
                let hint_str = hint.to_string();

                div()
                    .id(SharedString::from(format!("attach-{}", id_str)))
                    .px_3()
                    .py_2()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .hover(|el| el.bg(accent.opacity(0.5)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.hide_attachments_picker(cx);
                        match id_str {
                            "file" => {
                                info!("File picker not implemented yet");
                            }
                            "image" => {
                                info!("Image picker not implemented yet");
                            }
                            "clipboard" => {
                                this.paste_image_from_clipboard(cx);
                            }
                            _ => {}
                        }
                    }))
                    // Icon
                    .child(
                        svg()
                            .external_path(icon_name.external_path())
                            .size(px(16.))
                            .text_color(muted_fg),
                    )
                    // Name
                    .child(div().flex_1().text_sm().text_color(fg).child(name_str))
                    // Hint
                    .child(div().text_xs().text_color(muted_fg).child(hint_str))
            })
            .collect();

        // Show pending attachments if any
        let pending_items: Vec<_> = self
            .pending_attachments
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                div()
                    .id(SharedString::from(format!("pending-{}", idx)))
                    .px_3()
                    .py_1()
                    .mx_1()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(accent.opacity(0.2))
                    // File icon
                    .child(
                        svg()
                            .external_path(LocalIconName::File.external_path())
                            .size(px(14.))
                            .text_color(accent),
                    )
                    // Filename
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(filename),
                    )
                    // Remove button
                    .child(
                        div()
                            .id(SharedString::from(format!("remove-{}", idx)))
                            .cursor_pointer()
                            .hover(|el| el.text_color(theme.danger))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_attachment(idx, cx);
                            }))
                            .child(
                                svg()
                                    .external_path(LocalIconName::Close.external_path())
                                    .size(px(12.))
                                    .text_color(muted_fg),
                            ),
                    )
            })
            .collect();

        // Overlay
        // Theme-aware modal overlay: black for dark mode, white for light mode
        let overlay_bg = Self::get_modal_overlay_background();
        div()
            .id("attachments-picker-overlay")
            .absolute()
            .inset_0()
            .bg(overlay_bg)
            .flex()
            .items_end()
            .justify_start()
            .pb_20()
            .pl_4()
            .on_click(cx.listener(|this, _, _, cx| {
                this.hide_attachments_picker(cx);
            }))
            .child(
                div()
                    .id("attachments-picker-container")
                    .w(px(280.0))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_lg()
                    // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .on_click(cx.listener(|_, _, _, _| {}))
                    // Header
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border_color)
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(fg)
                            .child("Add Attachment"),
                    )
                    // Pending attachments (if any)
                    .when(!self.pending_attachments.is_empty(), |el| {
                        el.child(
                            div()
                                .px_2()
                                .py_1()
                                .border_b_1()
                                .border_color(border_color)
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(pending_items),
                        )
                    })
                    // Options
                    .child(div().p_1().children(option_items)),
            )
    }
}
