use gpui::{div, prelude::*, px, rgb, rgba, IntoElement};

use super::types::{BUTTON_GAP, KEYCAP_GAP, KEYCAP_SIZE, KEY_DISPLAY_HEIGHT, KEY_DISPLAY_PADDING};
use super::ShortcutRecorder;

impl ShortcutRecorder {
    /// Render a single keycap
    pub(super) fn render_keycap(&self, key: &str) -> impl IntoElement {
        let colors = self.colors;
        div()
            .w(px(KEYCAP_SIZE))
            .h(px(KEYCAP_SIZE))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((colors.keycap_bg << 8) | 0xFF))
            .border_1()
            .border_color(rgba((colors.keycap_border << 8) | 0x80))
            .rounded(px(8.))
            .text_xl()
            .font_weight(gpui::FontWeight::MEDIUM)
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

    /// Render the key display area
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
            // Show placeholder when nothing is pressed
            key_row = key_row.child(
                div()
                    .text_base()
                    .text_color(rgb(colors.text_muted))
                    .child("Press any key combination..."),
            );
        } else {
            // Show keycaps (either live modifiers or recorded shortcut)
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
            .bg(rgba((colors.key_display_bg << 8) | 0x60))
            .rounded(px(8.))
            .border_1()
            .border_color(rgba((colors.border << 8) | 0x40))
            .child(key_row)
    }

    /// Render conflict warning if present
    pub(super) fn render_conflict_warning(&self) -> impl IntoElement {
        let colors = self.colors;

        if let Some(ref conflict) = self.conflict {
            div()
                .w_full()
                .mt(px(12.))
                .px(px(12.))
                .py(px(8.))
                .bg(rgba((colors.warning << 8) | 0x20))
                .border_1()
                .border_color(rgba((colors.warning << 8) | 0x40))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(div().text_sm().text_color(rgb(colors.warning)).child("⚠"))
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(rgb(colors.text_secondary))
                        .child(format!("Already used by \"{}\"", conflict.command_name)),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        }
    }

    pub(super) fn button_gap_px() -> f32 {
        BUTTON_GAP
    }
}
