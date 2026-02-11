use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::logging;

use super::super::types::{
    is_clear_alias_shortcut, is_command_modifier, BUTTON_GAP, MODAL_PADDING, MODAL_WIDTH,
};
use super::AliasInput;

impl Focusable for AliasInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AliasInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let button_colors = ButtonColors::from_theme(&self.theme);
        let validation_feedback = self.validation_feedback();
        let overlay_appear = self.overlay_appear_style();
        let backdrop_hover_bg = rgba(Self::backdrop_hover_bg_token(colors));
        self.schedule_overlay_animation_tick_if_needed(overlay_appear.complete, cx);

        // Determine button states
        let can_save = validation_feedback.is_ok();
        let can_clear = self.current_alias.is_some();

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
                    .child(format!("Set Alias for \"{}\"", self.command_name)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(colors.text_muted))
                    .child("Type the alias + space in the main menu to run this command"),
            );

        // Build button row
        let clear_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.clear_alias();
            cx.notify();
        });

        let cancel_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.cancel();
            cx.notify();
        });

        let save_handler = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            this.save();
            cx.notify();
        });

        let buttons = div()
            .w_full()
            .mt(px(16.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                // Left side: Clear button (only if editing existing alias)
                div().when(can_clear, |d| {
                    d.child(
                        Button::new("Clear", button_colors)
                            .variant(ButtonVariant::Ghost)
                            .on_click(Box::new(move |event, window, cx| {
                                clear_handler(event, window, cx);
                            })),
                    )
                }),
            )
            .child(
                // Right side: Cancel and Save
                div()
                    .flex()
                    .flex_row()
                    .gap(px(BUTTON_GAP))
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

        let validation = match &validation_feedback {
            Ok(message) => div()
                .w_full()
                .mt(px(8.))
                .text_sm()
                .text_color(rgb(colors.text_muted))
                .child((*message).to_string()),
            Err(message) => div()
                .w_full()
                .mt(px(8.))
                .text_sm()
                .text_color(rgb(colors.text_error))
                .child(message.clone()),
        };

        // Key down event handler - captures all key events for text input
        let handle_key_down = cx.listener(move |this, event: &gpui::KeyDownEvent, _window, cx| {
            let key = event.keystroke.key.as_str();
            let mods = event.keystroke.modifiers;
            let key_char = event.keystroke.key_char.as_deref();
            let command_modifier = is_command_modifier(mods.platform, mods.control);

            logging::log_debug(
                "ALIAS_INPUT",
                &format!(
                    "KeyDown: key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, mods.platform, mods.control, mods.alt, mods.shift
                ),
            );

            // Handle special keys first
            if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc") {
                this.cancel();
                cx.notify();
                return;
            }

            if (key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return"))
                && !this.input.text().trim().is_empty()
            {
                this.save();
                cx.notify();
                return;
            }

            if is_clear_alias_shortcut(key, command_modifier, this.current_alias.is_some()) {
                this.clear_alias();
                cx.notify();
                return;
            }

            // Pass to text input handler for all other keys
            let handled =
                this.input
                    .handle_key(key, key_char, command_modifier, mods.alt, mods.shift, cx);

            if handled {
                cx.notify();
            }
        });

        // Cancel handler for backdrop clicks
        let backdrop_cancel = cx.listener(|this, _: &gpui::ClickEvent, _window, cx| {
            logging::log("ALIAS_INPUT", "Backdrop clicked - cancelling");
            this.cancel();
            cx.notify();
        });

        // Modal content - with stop propagation to prevent backdrop dismiss
        let modal = div()
            .id("alias-input-modal-content")
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
            .child(self.render_input_field(cx))
            .child(validation)
            .child(buttons);

        // Full-screen overlay with backdrop and centered modal
        // The overlay captures ALL keyboard events while open
        div()
            .id("alias-input-overlay")
            .absolute()
            .inset_0()
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key_down)
            // Backdrop layer - semi-transparent, captures clicks to dismiss
            .child(
                div()
                    .id("alias-input-backdrop")
                    .absolute()
                    .inset_0()
                    .bg(rgba((colors.overlay_bg << 8) | 0x80)) // 50% opacity
                    .opacity(overlay_appear.backdrop_opacity)
                    .cursor_pointer()
                    .hover(move |style| style.bg(backdrop_hover_bg))
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
