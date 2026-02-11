use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::logging;

use super::types::{
    overlay_color_with_alpha, MODAL_PADDING, MODAL_WIDTH, OVERLAY_BACKDROP_ALPHA,
    OVERLAY_BACKDROP_HOVER_ALPHA,
};
use super::ShortcutRecorder;

impl Focusable for ShortcutRecorder {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutRecorder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
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

        // Build header with command info
        let header = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(colors.text_primary))
                    .child("Record Keyboard Shortcut"),
            )
            .when_some(self.command_name.clone(), |d, name| {
                d.child(
                    div()
                        .text_base()
                        .text_color(rgb(colors.text_secondary))
                        .child(format!("For: {}", name)),
                )
            })
            .when_some(self.command_description.clone(), |d, desc| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(rgb(colors.text_muted))
                        .child(desc),
                )
            });

        // Build button row
        let clear_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.clear(cx);
        });

        let cancel_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            this.cancel();
        });

        let save_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            this.save();
        });

        let buttons = div()
            .w_full()
            .mt(px(16.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                // Left side: Clear button
                Button::new("Clear", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .disabled(!can_clear)
                    .on_click(Box::new(move |event, window, cx| {
                        clear_handler(event, window, cx);
                    })),
            )
            .child(
                // Right side: Cancel and Save
                div()
                    .flex()
                    .flex_row()
                    .gap(px(Self::button_gap_px()))
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
                    ),
            );

        // Instructions
        let instructions = div()
            .w_full()
            .mt(px(12.))
            .text_xs()
            .text_color(rgb(colors.text_muted))
            .text_center()
            .child("Press a modifier (⌘⌃⌥⇧) + a key");

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
            if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc") {
                this.handle_escape(cx);
            } else if (key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return"))
                && this.shortcut.is_complete()
                && this.conflict.is_none()
            {
                this.save();
                cx.notify();
            } else {
                this.handle_key_down(key, mods, cx);
            }
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
        let backdrop_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, _cx| {
            logging::log("SHORTCUT", "Backdrop clicked - cancelling");
            this.cancel();
        });

        // Modal content - with stop propagation to prevent backdrop dismiss
        let modal = div()
            .id("shortcut-modal-content")
            .w(px(MODAL_WIDTH))
            .p(px(MODAL_PADDING))
            .bg(rgba((colors.modal_bg << 8) | 0xF0))
            .border_1()
            .border_color(rgba((colors.border << 8) | 0x80))
            .rounded(px(12.))
            .flex()
            .flex_col()
            // Stop propagation - clicks inside modal shouldn't dismiss it
            .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                // Empty handler stops propagation to backdrop
            })
            .child(header)
            .child(div().h(px(16.))) // Spacer
            .child(self.render_key_display())
            .child(self.render_conflict_warning())
            .child(instructions)
            .child(buttons);

        // Full-screen overlay with backdrop and centered modal
        // The overlay captures ALL keyboard and modifier events while open
        div()
            .id("shortcut-recorder-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key_down)
            .on_modifiers_changed(handle_modifiers_changed) // CRITICAL: Live modifier feedback
            // Backdrop layer - semi-transparent, captures clicks to dismiss
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
            // Modal container - centered on top of backdrop
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
