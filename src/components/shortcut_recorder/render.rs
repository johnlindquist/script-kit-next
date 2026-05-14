use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::components::overlay_modal::OverlayAnimation;
use crate::logging;
use crate::ui_foundation::{is_key_enter, is_key_escape};

use super::types::{
    overlay_color_with_alpha, BUTTON_GAP, OVERLAY_BACKDROP_ALPHA, OVERLAY_BACKDROP_HOVER_ALPHA,
    RECORDER_MODAL_PADDING, RECORDER_MODAL_WIDTH,
};
use super::ShortcutRecorder;

impl Focusable for ShortcutRecorder {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutRecorder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        crate::components::hint_strip::emit_shortcut_chrome_audit(
            "shortcut_recorder",
            "compact-modal",
        );

        let colors = self.colors;
        let chrome = crate::theme::AppChromeColors::from_theme(&self.theme);
        let button_colors = ButtonColors::from_theme(&self.theme);
        let overlay_appear = self.overlay_appear_style();
        let backdrop_bg = rgba(overlay_color_with_alpha(
            colors.overlay_bg,
            OVERLAY_BACKDROP_ALPHA,
        ));
        let backdrop_hover_bg = rgba(overlay_color_with_alpha(
            colors.overlay_bg,
            OVERLAY_BACKDROP_HOVER_ALPHA,
        ));
        self.schedule_overlay_animation_tick_if_needed(overlay_appear.complete, cx);

        // Determine button states
        let can_save = self.shortcut.is_complete() && self.conflict.is_none();
        let can_clear = !self.shortcut.is_empty();

        let title = self
            .command_name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("Shortcut")
            .to_string();
        let header = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .w(px(2.0))
                    .h(px(14.0))
                    .rounded(px(1.0))
                    .bg(rgb(chrome.accent_hex)),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .truncate()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(chrome.text_primary_hex))
                    .child(title),
            );

        // Build button row
        let clear_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.clear(cx);
        });

        let cancel_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.cancel();
            cx.notify();
        });

        let save_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.save();
            cx.notify();
        });

        let mut buttons = div()
            .w_full()
            .mt(px(12.))
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap(px(BUTTON_GAP));

        if can_clear {
            buttons = buttons.child(
                Button::new("Clear", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .on_click(Box::new(move |event, window, cx| {
                        clear_handler(event, window, cx);
                    })),
            );
        }

        let buttons = buttons
            .child(
                Button::new("Cancel", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .shortcut("Esc")
                    .on_click(Box::new(move |event, window, cx| {
                        cancel_handler(event, window, cx);
                    })),
            )
            .child(
                Button::new("Save", button_colors)
                    .variant(ButtonVariant::Primary)
                    .shortcut("↵")
                    .disabled(!can_save)
                    .on_click(Box::new(move |event, window, cx| {
                        save_handler(event, window, cx);
                    })),
            );

        // Key down event handler - captures modifiers and keys
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;

            logging::log(
                "SHORTCUT",
                &format!(
                    "KeyDown: key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, mods.platform, mods.control, mods.alt, mods.shift
                ),
            );

            // Handle special keys
            if (mods.platform && key.eq_ignore_ascii_case("w")) || is_key_escape(key) {
                this.cancel();
                cx.notify();
            } else if is_key_enter(key) && this.shortcut.is_complete() && this.conflict.is_none() {
                this.save();
                cx.notify();
            } else {
                this.handle_key_down(key, mods, cx);
            }
            cx.stop_propagation();
        });

        // Modifiers changed handler - CRITICAL for live modifier feedback
        // This fires whenever ANY modifier key is pressed or released (e.g., pressing Cmd alone)
        let handle_modifiers_changed = cx.listener(
            move |this, event: &gpui::ModifiersChangedEvent, _window, cx| {
                // Only update if we're still recording (haven't captured a complete shortcut yet)
                if this.is_recording {
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "ModifiersChanged: cmd={} ctrl={} alt={} shift={}",
                            event.modifiers.platform,
                            event.modifiers.control,
                            event.modifiers.alt,
                            event.modifiers.shift
                        ),
                    );
                    // Update current modifiers for live display
                    this.current_modifiers = event.modifiers;
                    cx.notify(); // Trigger re-render to show keycaps
                }
            },
        );

        // Cancel handler for backdrop clicks
        let backdrop_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            logging::log("SHORTCUT", "Backdrop clicked - cancelling");
            this.cancel();
            cx.notify();
        });
        let detached_surface_cancel = cx.listener(|this, _: &gpui::MouseDownEvent, _window, cx| {
            logging::log(
                "SHORTCUT",
                "Shortcut recorder popup margin clicked - cancelling",
            );
            this.cancel();
            cx.notify();
            cx.stop_propagation();
        });

        // Modal content - with stop propagation to prevent backdrop dismiss
        let modal = div()
            .id("shortcut-modal-content")
            .w(px(RECORDER_MODAL_WIDTH))
            .p(px(RECORDER_MODAL_PADDING))
            .bg(rgba(chrome.popup_surface_rgba))
            .border_1()
            .border_color(rgba(chrome.border_rgba))
            .rounded(px(8.))
            .flex()
            .flex_col()
            // Stop propagation - clicks inside modal shouldn't dismiss it
            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                // Empty handler stops propagation to backdrop
            })
            .child(header)
            .child(div().h(px(10.)))
            .child(self.render_key_display())
            .child(self.render_conflict_warning())
            .child(buttons);

        let recorder_surface = div()
            .id("shortcut-recorder-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key_down)
            .on_modifiers_changed(handle_modifiers_changed); // CRITICAL: Live modifier feedback

        if self.detached_window {
            recorder_surface.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .mt(px(overlay_appear.modal_offset_y))
                    .opacity(overlay_appear.modal_opacity)
                    .on_mouse_down(gpui::MouseButton::Left, detached_surface_cancel)
                    .child(modal),
            )
        } else {
            // Full-screen overlay with backdrop and centered modal.
            recorder_surface
                .child(
                    div()
                        .id("shortcut-backdrop")
                        .absolute()
                        .inset_0()
                        .bg(backdrop_bg)
                        .cursor_pointer()
                        .hover(move |style| style.bg(backdrop_hover_bg))
                        .opacity(overlay_appear.backdrop_opacity)
                        .on_click(backdrop_cancel),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .mt(px(overlay_appear.modal_offset_y))
                        .opacity(overlay_appear.modal_opacity)
                        .child(modal),
                )
        }
    }
}
