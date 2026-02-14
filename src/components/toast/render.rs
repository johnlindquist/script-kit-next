use gpui::*;

use crate::components::button::{Button, ButtonColors, ButtonVariant};

use super::types::{
    TOAST_ACTIONS_GAP_PX, TOAST_ACTIONS_MARGIN_TOP_PX, TOAST_BORDER_WIDTH_PX, TOAST_CONTENT_GAP_PX,
    TOAST_CONTENT_PADDING_X_PX, TOAST_CONTENT_PADDING_Y_PX, TOAST_ICON_SIZE_PX, TOAST_MAX_WIDTH_PX,
    TOAST_MESSAGE_COLUMN_GAP_PX, TOAST_RADIUS_PX,
};
use super::{Toast, ToastColors};

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        let variant = self.variant;
        let on_dismiss_callback = self.on_dismiss;
        let transition = self.transition;

        // Check vibrancy to conditionally apply shadow
        let vibrancy_enabled = crate::theme::load_theme().is_vibrancy_enabled();

        // Main toast container with transition support
        // Apply shadow conditionally BEFORE .id() to avoid Stateful<Div> type issues
        let base_toast = div()
            .flex()
            .flex_col()
            .w_full()
            .max_w(px(TOAST_MAX_WIDTH_PX))
            .bg(rgba((colors.background << 8) | 0xF0)) // 94% opacity
            .border_l(px(TOAST_BORDER_WIDTH_PX))
            .border_color(rgb(colors.border))
            .rounded(px(TOAST_RADIUS_PX));

        // Only apply shadow when vibrancy is disabled - shadows block blur
        let styled_toast = if vibrancy_enabled {
            base_toast
        } else {
            base_toast.shadow_md()
        };

        let mut toast = styled_toast
            .id(ElementId::Name(SharedString::from(format!(
                "toast-{}",
                self.message
            ))))
            .overflow_hidden()
            // Apply transition opacity
            .opacity(transition.opacity.value())
            // Apply transition offset via top margin (positive y = down, negative = up)
            .mt(px(transition.offset.y)); // Keep animation offset as px

        // Content row (icon, message, actions, dismiss)
        let content_row = div()
            .flex()
            .flex_row()
            .items_start()
            .gap(px(TOAST_CONTENT_GAP_PX))
            .px(px(TOAST_CONTENT_PADDING_X_PX))
            .py(px(TOAST_CONTENT_PADDING_Y_PX));

        let icon = div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(TOAST_ICON_SIZE_PX))
            .h(px(TOAST_ICON_SIZE_PX))
            .text_lg()
            .text_color(rgb(colors.icon))
            .font_weight(FontWeight::BOLD)
            .child(variant.icon());

        // Message and actions column
        let mut message_col = div()
            .flex()
            .flex_col()
            .flex_1()
            .gap(px(TOAST_MESSAGE_COLUMN_GAP_PX));

        // Message text
        let message_text = div()
            .text_sm()
            .text_color(rgb(colors.text))
            .font_weight(FontWeight::MEDIUM)
            .child(self.message.clone());

        message_col = message_col.child(message_text);

        // Actions row (if any)
        if !self.actions.is_empty() {
            let mut actions_row = div()
                .flex()
                .flex_row()
                .gap(px(TOAST_ACTIONS_GAP_PX))
                .mt(px(TOAST_ACTIONS_MARGIN_TOP_PX));

            for action in self.actions {
                let callback = action.callback.clone();
                let label = action.label.clone();
                // Create button colors for toast action buttons (Ghost style)
                let button_colors = ButtonColors {
                    text_color: colors.action_text,
                    text_hover: colors.action_text,
                    background: colors.action_background,
                    background_hover: colors.action_background,
                    accent: colors.action_text,
                    border: colors.border,
                    focus_ring: colors.action_text,
                    focus_tint: colors.action_background,
                    hover_overlay: ToastColors::overlay_with_alpha(colors.action_background, 0x26),
                };
                let action_btn = Button::new(label.clone(), button_colors)
                    .variant(ButtonVariant::Ghost)
                    .on_click(Box::new(move |event, window, cx| {
                        tracing::debug!(action = %label, "Toast action button clicked");
                        (callback)(event, window, cx);
                    }));

                actions_row = actions_row.child(action_btn);
            }

            message_col = message_col.child(actions_row);
        }

        // Dismiss button (if dismissible)
        let dismiss_btn = if self.dismissible {
            let dismiss_callback = on_dismiss_callback.clone();
            // Create button colors for dismiss button (Icon style)
            let button_colors = ButtonColors {
                text_color: colors.dismiss,
                text_hover: colors.text,
                background: 0x00000000, // transparent
                background_hover: colors.action_background,
                accent: colors.dismiss,
                border: 0x00000000, // no border
                focus_ring: colors.dismiss,
                focus_tint: colors.action_background,
                hover_overlay: ToastColors::overlay_with_alpha(colors.action_background, 0x26),
            };
            Some(
                Button::new("×", button_colors)
                    .variant(ButtonVariant::Icon)
                    .on_click(Box::new(move |_event, window, cx| {
                        tracing::debug!("Toast dismiss button clicked");
                        if let Some(ref callback) = dismiss_callback {
                            callback(window, cx);
                        }
                    })),
            )
        } else {
            None
        };

        // Assemble content row
        let mut assembled_row = content_row.child(icon).child(message_col);

        if let Some(dismiss) = dismiss_btn {
            assembled_row = assembled_row.child(dismiss);
        }

        toast = toast.child(assembled_row);

        // Details section (if present)
        if let Some(details_text) = self.details {
            let details_section = div()
                .w_full()
                .px(px(TOAST_CONTENT_PADDING_X_PX))
                .py(px(TOAST_CONTENT_PADDING_Y_PX))
                .bg(rgba(colors.details_bg)) // Theme-aware details background
                .border_t_1()
                .border_color(rgba((colors.border << 8) | 0x40))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text))
                        .font_family("Menlo")
                        .overflow_hidden()
                        .child(details_text),
                );

            toast = toast.child(details_section);
        }

        toast
    }
}
