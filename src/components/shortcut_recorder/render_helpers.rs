use gpui::{div, prelude::*, px, rgb, rgba, IntoElement};

use super::types::{KEYCAP_GAP, KEYCAP_SIZE, KEY_DISPLAY_HEIGHT, KEY_DISPLAY_PADDING};
use super::ShortcutRecorder;

impl ShortcutRecorder {
    /// Render a single keycap — lightweight, reduced chrome
    pub(super) fn render_keycap(&self, key: &str) -> impl IntoElement {
        let colors = self.colors;
        div()
            .w(px(KEYCAP_SIZE))
            .h(px(KEYCAP_SIZE))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.keycap_bg << 8) | 0xCC))
            .border_1()
            .border_color(rgba((colors.keycap_border << 8) | 0x40))
            .rounded(px(6.))
            .text_base()
            .font_weight(gpui::FontWeight::NORMAL)
            .text_color(rgb(colors.text_primary))
            .child(key.to_string())
    }

    /// Get keycaps for live display - shows current modifiers while recording,
    /// or the final recorded shortcut when complete
    pub(super) fn get_display_keycaps(&self) -> Vec<String> {
        if self.shortcut.is_complete() {
            // Show the final recorded shortcut
            self.shortcut.to_keycaps()
        } else if self.is_recording {
            // Show currently held modifiers (live feedback)
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
            // Recording complete but no final key - show what we have
            self.shortcut.to_keycaps()
        }
    }

    /// Render the key display area — subtle container, no heavy border
    pub(super) fn render_key_display(&self) -> impl IntoElement {
        let colors = self.colors;
        let keycaps = self.get_display_keycaps();

        let mut key_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(KEYCAP_GAP));

        if keycaps.is_empty() {
            // Quiet placeholder
            key_row = key_row.child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_muted))
                    .child("Waiting for input…"),
            );
        } else {
            for keycap in keycaps {
                key_row = key_row.child(self.render_keycap(&keycap));
            }
        }

        div()
            .w_full()
            .h(px(KEY_DISPLAY_HEIGHT))
            .px(px(KEY_DISPLAY_PADDING))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.key_display_bg << 8) | 0x40))
            .rounded(px(6.))
            .child(key_row)
    }

    /// Render conflict warning — inline text, no bordered box
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
