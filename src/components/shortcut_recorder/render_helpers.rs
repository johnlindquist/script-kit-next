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

    /// Render the key display area with the shared compact shortcut chrome.
    pub(super) fn render_key_display(&self) -> impl IntoElement {
        let colors = self.colors;
        let keycaps = self.get_display_keycaps();

        let content = if keycaps.is_empty() {
            div()
                .text_sm()
                .text_color(rgb(colors.text_muted))
                .child("Press any key combination...")
                .into_any_element()
        } else {
            crate::components::hint_strip::render_inline_shortcut_keys(
                keycaps.iter().map(String::as_str),
                crate::components::hint_strip::whisper_inline_shortcut_colors(
                    rgba((colors.text_primary << 8) | 0xD0).into(),
                    rgba((colors.border << 8) | 0xFF).into(),
                    true,
                ),
            )
        };

        div()
            .w_full()
            .h(px(KEY_DISPLAY_HEIGHT))
            .px(px(KEY_DISPLAY_PADDING))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.key_display_bg << 8) | 0x38))
            .rounded(px(6.))
            .border_1()
            .border_color(rgba((colors.border << 8) | 0x28))
            .child(content)
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
