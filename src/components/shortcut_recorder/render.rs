use gpui::{
    div, prelude::*, px, rgb, rgba, App, Context, FocusHandle, Focusable, IntoElement, Render,
    Window,
};

use crate::components::confirm_modal_shell::{
    confirm_modal_header, confirm_modal_number_override, confirm_modal_shell,
    ConfirmModalShellConfig, CONFIRM_MODAL_RADIUS,
};
use crate::components::footer_chrome::{
    current_main_menu_footer_height, current_main_menu_footer_metrics, footer_action_slot_width,
    footer_button_height, render_footer_hint_action_button_frame, FooterActionSlot,
    FooterHintActionButtonFrameSpec, FooterHintButtonLayoutOverrides, FooterHintContentJustify,
};
use crate::components::overlay_modal::OverlayAnimation;
use crate::dev_style_tool::{
    ConfirmModalKnobId, CONFIRM_MODAL_ACTIONS_BUTTON_HEIGHT_KNOB_ID,
    CONFIRM_MODAL_ACTIONS_BUTTON_RADIUS_KNOB_ID, CONFIRM_MODAL_ACTIONS_CONTENT_GAP_KNOB_ID,
    CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID, CONFIRM_MODAL_ACTIONS_GAP_KNOB_ID,
    CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID, CONFIRM_MODAL_ACTIONS_PADDING_Y_KNOB_ID,
};
use crate::logging;
use crate::ui_foundation::{is_key_enter, is_key_escape};

use super::types::{
    overlay_color_with_alpha, OVERLAY_BACKDROP_ALPHA, OVERLAY_BACKDROP_HOVER_ALPHA,
    RECORDER_MODAL_PADDING, RECORDER_MODAL_WIDTH,
};
use super::ShortcutRecorder;

fn recorder_modal_number(id: ConfirmModalKnobId, fallback: f32) -> f32 {
    confirm_modal_number_override(id, fallback)
}

fn recorder_action_button_height() -> f32 {
    recorder_modal_number(
        CONFIRM_MODAL_ACTIONS_BUTTON_HEIGHT_KNOB_ID,
        footer_button_height(current_main_menu_footer_height()),
    )
}

fn recorder_action_button_gap() -> f32 {
    recorder_modal_number(
        CONFIRM_MODAL_ACTIONS_GAP_KNOB_ID,
        current_main_menu_footer_metrics().item_gap_px,
    )
}

fn recorder_action_button_layout() -> FooterHintButtonLayoutOverrides {
    let metrics = current_main_menu_footer_metrics();
    FooterHintButtonLayoutOverrides {
        button_padding_x_px: Some(recorder_modal_number(
            CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID,
            metrics.button_padding_x,
        )),
        button_padding_y_px: Some(recorder_modal_number(
            CONFIRM_MODAL_ACTIONS_PADDING_Y_KNOB_ID,
            metrics.button_padding_y,
        )),
        content_gap_px: Some(recorder_modal_number(
            CONFIRM_MODAL_ACTIONS_CONTENT_GAP_KNOB_ID,
            metrics.content_gap,
        )),
        button_radius_px: Some(recorder_modal_number(
            CONFIRM_MODAL_ACTIONS_BUTTON_RADIUS_KNOB_ID,
            metrics.button_radius,
        )),
        edge_padding_x_px: Some(recorder_modal_number(
            CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID,
            metrics.button_padding_x,
        )),
        shrink_frame_to_content_px: true,
    }
}

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
        let header =
            confirm_modal_header(title, rgb(chrome.accent_hex), rgb(chrome.text_primary_hex));

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

        let action_button_height = recorder_action_button_height();
        let action_button_layout = recorder_action_button_layout();
        let close_slot_width = footer_action_slot_width(FooterActionSlot::Close);
        let run_slot_width = footer_action_slot_width(FooterActionSlot::Run);

        let mut buttons = div()
            .w_full()
            .mt(px(12.))
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap(px(recorder_action_button_gap()));

        if can_clear {
            buttons = buttons.child(
                render_footer_hint_action_button_frame(
                    FooterHintActionButtonFrameSpec {
                        id: "shortcut-clear-button",
                        label: "Clear".into(),
                        key: "".into(),
                        slot_width_px: close_slot_width,
                        height_px: action_button_height,
                        selected: false,
                        key_first: false,
                        justify: FooterHintContentJustify::Center,
                        layout: action_button_layout,
                    },
                    &self.theme,
                )
                .on_click(move |event, window, cx| {
                    clear_handler(event, window, cx);
                }),
            );
        }

        let buttons = buttons
            .child(
                render_footer_hint_action_button_frame(
                    FooterHintActionButtonFrameSpec {
                        id: "shortcut-cancel-button",
                        label: "Cancel".into(),
                        key: "Esc".into(),
                        slot_width_px: close_slot_width,
                        height_px: action_button_height,
                        selected: false,
                        key_first: false,
                        justify: FooterHintContentJustify::Center,
                        layout: action_button_layout,
                    },
                    &self.theme,
                )
                .on_click(move |event, window, cx| {
                    cancel_handler(event, window, cx);
                }),
            )
            .child(
                render_footer_hint_action_button_frame(
                    FooterHintActionButtonFrameSpec {
                        id: "shortcut-save-button",
                        label: "Save".into(),
                        key: "↵".into(),
                        slot_width_px: run_slot_width,
                        height_px: action_button_height,
                        selected: can_save,
                        key_first: false,
                        justify: FooterHintContentJustify::Center,
                        layout: action_button_layout,
                    },
                    &self.theme,
                )
                .when(!can_save, |style| style.opacity(0.45))
                .on_click(move |event, window, cx| {
                    if can_save {
                        save_handler(event, window, cx);
                    }
                }),
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
        let modal = confirm_modal_shell(
            ConfirmModalShellConfig {
                content_id: "shortcut-modal-content",
                width: Some(RECORDER_MODAL_WIDTH),
                padding_x: RECORDER_MODAL_PADDING,
                padding_y: RECORDER_MODAL_PADDING,
                gap: 10.0,
                background: (!self.detached_window).then_some(rgba(chrome.popup_surface_rgba)),
                border: rgba(chrome.border_rgba),
                radius: CONFIRM_MODAL_RADIUS,
                offset_y: 0.0,
                opacity: 1.0,
            },
            vec![
                header.into_any_element(),
                self.render_key_display().into_any_element(),
                self.render_conflict_warning().into_any_element(),
                buttons.into_any_element(),
            ],
        )
        // Stop propagation - clicks inside modal shouldn't dismiss it
        .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
            // Empty handler stops propagation to backdrop
        });

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
