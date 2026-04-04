use gpui::{div, prelude::*, px, rgb, rgba, IntoElement};

use super::types::{KEY_DISPLAY_HEIGHT, KEY_DISPLAY_PADDING};
use super::ShortcutRecorder;

impl ShortcutRecorder {
    /// Get keycaps for live display - shows current modifiers while recording,
    /// or the final recorded shortcut when complete
    pub(super) fn get_display_keycaps(&self) -> Vec<String> {
        if self.shortcut.is_complete() {
            self.shortcut.to_keycaps()
        } else if self.is_recording {
            let mut keycaps = Vec::new();
            if self.current_modifiers.control {
                keycaps.push("⌃".to_string());
            }
            if self.current_modifiers.alt {
                keycaps.push("⌥".to_string());
            }
            if self.current_modifiers.shift {
                keycaps.push("⇧".to_string());
            }
            if self.current_modifiers.platform {
                keycaps.push("⌘".to_string());
            }
            keycaps
        } else {
            self.shortcut.to_keycaps()
        }
    }

    /// Render the live shortcut preview as an inline status row.
    /// This keeps the recorder aligned with the footer hint strip language instead of
    /// giving the shortcut a boxed "mini keyboard" treatment.
    pub(super) fn render_key_display(&self) -> impl IntoElement {
        let colors = self.colors;
        let keycaps = self.get_display_keycaps();

        crate::components::hint_strip::emit_shortcut_chrome_audit(
            "shortcut_recorder_display",
            "inline-preview",
        );

        let content = if keycaps.is_empty() {
            div()
                .text_xs()
                .text_color(rgba((colors.text_muted << 8) | 0xD0))
                .child("Hold modifiers, then press a key")
                .into_any_element()
        } else {
            let status = if self.shortcut.is_complete() {
                "Recorded"
            } else {
                "Listening"
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba((colors.text_muted << 8) | 0xC0))
                        .child(status),
                )
                .child(crate::components::hint_strip::render_inline_shortcut_keys(
                    keycaps.iter().map(String::as_str),
                    crate::components::hint_strip::whisper_inline_shortcut_colors(
                        rgba((colors.text_primary << 8) | 0xD0).into(),
                        rgba((colors.border << 8) | 0xFF).into(),
                        true,
                    ),
                ))
                .into_any_element()
        };

        div()
            .w_full()
            .h(px(KEY_DISPLAY_HEIGHT))
            .px(px(KEY_DISPLAY_PADDING))
            .flex()
            .items_center()
            .justify_center()
            .child(content)
    }

    /// Render a compact footer strip: ⎋ Cancel • ↵ Save
    pub(super) fn render_footer_shortcuts(&self) -> gpui::AnyElement {
        let colors = self.colors;

        crate::components::hint_strip::emit_shortcut_chrome_audit(
            "shortcut_recorder_footer",
            "compact-inline",
        );

        let hint = |keys: &[&str], label: &'static str| -> gpui::AnyElement {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .child(crate::components::hint_strip::render_inline_shortcut_keys(
                    keys.iter().copied(),
                    crate::components::hint_strip::whisper_inline_shortcut_colors(
                        rgb(colors.text_muted).into(),
                        rgba((colors.border << 8) | 0xFF).into(),
                        true,
                    ),
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(label),
                )
                .into_any_element()
        };

        div()
            .w_full()
            .mt(px(10.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(12.0))
            .child(hint(&["⎋"], "Cancel"))
            .child(
                div()
                    .text_xs()
                    .text_color(rgba((colors.text_muted << 8) | 0x72))
                    .child("•"),
            )
            .child(hint(&["↵"], "Save"))
            .into_any_element()
    }

    /// Render conflict warning if present
    pub(super) fn render_conflict_warning(&self) -> impl IntoElement {
        let colors = self.colors;

        if let Some(ref conflict) = self.conflict {
            div()
                .w_full()
                .mt(px(8.))
                .text_xs()
                .text_color(rgb(colors.warning))
                .text_center()
                .child(format!("Conflicts with \"{}\"", conflict.command_name))
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }
}
