use gpui::*;
use std::rc::Rc;

use super::{
    types::{
        BUTTON_CONTENT_GAP_PX, BUTTON_ICON_PADDING_X, BUTTON_ICON_PADDING_Y,
        BUTTON_PRIMARY_PADDING_X, BUTTON_PRIMARY_PADDING_Y, BUTTON_RADIUS_PX,
        BUTTON_SHORTCUT_MARGIN_LEFT_PX,
    },
    ButtonColors, ButtonVariant, BUTTON_GHOST_HEIGHT, BUTTON_GHOST_PADDING_X,
    BUTTON_GHOST_PADDING_Y,
};

/// Callback type for button click events
pub type OnClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A reusable button component for interactive actions
///
/// Supports:
/// - Label text (required)
/// - Keyboard shortcut display (optional)
/// - Three variants: Primary, Ghost, Icon
/// - Hover states with themed colors
/// - Focus ring styling
/// - Click callback
///
#[derive(IntoElement)]
pub struct Button {
    label: SharedString,
    colors: ButtonColors,
    variant: ButtonVariant,
    shortcut: Option<String>,
    id: Option<SharedString>,
    disabled: bool,
    loading: bool,
    loading_label: Option<SharedString>,
    focused: bool,
    on_click: Option<Rc<OnClickCallback>>,
    focus_handle: Option<FocusHandle>,
}

impl Button {
    /// Create a new button with the given label and pre-computed colors
    pub fn new(label: impl Into<SharedString>, colors: ButtonColors) -> Self {
        Self {
            label: label.into(),
            colors,
            variant: ButtonVariant::default(),
            shortcut: None,
            id: None,
            disabled: false,
            loading: false,
            loading_label: None,
            focused: false,
            on_click: None,
            focus_handle: None,
        }
    }

    /// Set the button variant (Primary, Ghost, Icon)
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the keyboard shortcut display text
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set an optional shortcut (convenience for Option<String>)
    pub fn shortcut_opt(mut self, shortcut: Option<String>) -> Self {
        self.shortcut = shortcut;
        self
    }

    /// Set an optional id on the button root element
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set whether the button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the button is in loading state
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set optional loading label text
    pub fn loading_label(mut self, loading_label: impl Into<SharedString>) -> Self {
        self.loading_label = Some(loading_label.into());
        self
    }

    /// Set whether the button is focused (shows focus ring)
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the click callback
    pub fn on_click(mut self, callback: OnClickCallback) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }

    /// Set the label text
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the focus handle for keyboard accessibility
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    pub(crate) fn resolve_element_id(
        id: Option<&SharedString>,
        label: &SharedString,
    ) -> SharedString {
        id.cloned().unwrap_or_else(|| label.clone())
    }

    pub(crate) fn should_show_pointer(
        has_click_handler: bool,
        disabled: bool,
        loading: bool,
    ) -> bool {
        has_click_handler && !disabled && !loading
    }

    fn is_activation_key(key: &str) -> bool {
        matches!(
            key,
            "enter" | "return" | "Enter" | "Return" | " " | "space" | "Space"
        )
    }

    pub(crate) fn can_activate_from_key(
        key: &str,
        has_click_handler: bool,
        disabled: bool,
        loading: bool,
    ) -> bool {
        Self::should_show_pointer(has_click_handler, disabled, loading)
            && Self::is_activation_key(key)
    }

    pub(crate) fn resolve_focus_state(explicit_focus: bool, runtime_focus: Option<bool>) -> bool {
        runtime_focus.unwrap_or(explicit_focus)
    }

    pub(crate) fn should_show_focus_indicator(
        focused: bool,
        has_click_handler: bool,
        disabled: bool,
        loading: bool,
    ) -> bool {
        focused && Self::should_show_pointer(has_click_handler, disabled, loading)
    }

    pub(crate) fn hover_background_token(variant: ButtonVariant, colors: ButtonColors) -> u32 {
        match variant {
            ButtonVariant::Primary => (colors.background_hover << 8) | 0xB0,
            ButtonVariant::Ghost | ButtonVariant::Icon => colors.hover_overlay,
        }
    }
}

/// Focus ring border width
const FOCUS_BORDER_WIDTH: f32 = 2.0;

impl RenderOnce for Button {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Button {
            label,
            colors,
            variant,
            shortcut,
            id,
            disabled,
            loading,
            loading_label,
            focused,
            on_click,
            focus_handle,
        } = self;
        let on_click_callback = on_click.clone();
        let on_click_for_key = on_click;
        let has_click_handler = on_click_callback.is_some();
        let show_pointer = Self::should_show_pointer(has_click_handler, disabled, loading);
        let element_id = Self::resolve_element_id(id.as_ref(), &label);
        let label_for_log = label.clone();
        let focused = Self::resolve_focus_state(
            focused,
            focus_handle
                .as_ref()
                .map(|handle| handle.is_focused(window)),
        );
        let show_focus_indicator =
            Self::should_show_focus_indicator(focused, has_click_handler, disabled, loading);
        let label_text = if loading {
            loading_label.unwrap_or_else(|| label.clone())
        } else {
            label.clone()
        };

        // Calculate colors based on variant
        let hover_bg = rgba(Self::hover_background_token(variant, colors));

        // Focus styling colors
        // 0xA0 = 62.5% opacity for visible focus ring
        let focus_ring_color = rgba((colors.focus_ring << 8) | 0xA0);
        // 0x20 = 12.5% opacity for subtle background tint
        let focus_tint = rgba((colors.focus_tint << 8) | 0x20);
        // 0x40 = 25% opacity for unfocused border
        let unfocused_border = rgba((colors.border << 8) | 0x40);

        let (text_color, bg_color) = match variant {
            ButtonVariant::Primary => {
                // Primary: filled background with accent color
                // When focused, add subtle tint on top
                let base_bg = rgba((colors.background << 8) | 0x80);
                let bg = if focused {
                    // Brighter when focused
                    rgba((colors.background << 8) | 0xA0)
                } else {
                    base_bg
                };
                (rgb(colors.accent), bg)
            }
            ButtonVariant::Ghost => {
                // Ghost: text only (accent color), white overlay on hover
                // When focused, add subtle tint
                let bg = if focused {
                    focus_tint
                } else {
                    rgba(0x00000000)
                };
                (rgb(colors.accent), bg)
            }
            ButtonVariant::Icon => {
                // Icon: compact, accent color, white overlay on hover
                let bg = if focused {
                    focus_tint
                } else {
                    rgba(0x00000000)
                };
                (rgb(colors.accent), bg)
            }
        };

        // Build shortcut element if present - smaller than label, same accent color
        // Use flex + items_center to ensure vertical alignment with the label
        let shortcut_element = if let Some(sc) = shortcut {
            div()
                .flex()
                .items_center()
                .text_xs()
                .ml(px(BUTTON_SHORTCUT_MARGIN_LEFT_PX))
                .child(sc)
        } else {
            div()
        };

        // Determine padding based on variant using canonical button spacing tokens.
        let (padding_x, padding_y) = match variant {
            ButtonVariant::Primary => (BUTTON_PRIMARY_PADDING_X, BUTTON_PRIMARY_PADDING_Y),
            ButtonVariant::Ghost => (BUTTON_GHOST_PADDING_X, BUTTON_GHOST_PADDING_Y),
            ButtonVariant::Icon => (BUTTON_ICON_PADDING_X, BUTTON_ICON_PADDING_Y),
        };

        // Build the button element
        let mut button = div()
            .id(ElementId::Name(element_id))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(BUTTON_CONTENT_GAP_PX))
            .px(px(padding_x))
            .py(px(padding_y))
            .min_h(px(BUTTON_GHOST_HEIGHT))
            .rounded(px(BUTTON_RADIUS_PX))
            .bg(bg_color)
            .text_color(text_color)
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .font_family(crate::list_item::FONT_SYSTEM_UI)
            .cursor_default()
            .child(label_text)
            .child(shortcut_element);

        if loading {
            button = button.child(div().text_xs().opacity(0.7).child("…"));
        }

        // Apply focus ring styling
        if show_focus_indicator {
            button = button
                .border(px(FOCUS_BORDER_WIDTH))
                .border_color(focus_ring_color);
        } else {
            button = button.border_1().border_color(unfocused_border);
        }

        // Apply hover styles unless disabled
        // Keep text color the same, just add subtle background lift
        if show_pointer {
            button = button.cursor_pointer().hover(move |s| s.bg(hover_bg));
        } else if disabled {
            button = button.opacity(0.5).cursor_default();
        } else if loading {
            button = button.opacity(0.7).cursor_default();
        } else {
            button = button.cursor_default();
        }

        // Add click handler if provided
        if let Some(callback) = on_click_callback {
            if show_pointer {
                button = button.on_click(move |event, window, cx| {
                    tracing::debug!(button = %label_for_log, "Button clicked");
                    callback(event, window, cx);
                });
            }
        }

        // Add focus tracking and keyboard handler if focus_handle is provided
        if let Some(handle) = focus_handle {
            button = button.track_focus(&handle);

            if show_pointer {
                if let Some(callback) = on_click_for_key {
                    button = button.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if Button::can_activate_from_key(key, true, disabled, loading) {
                            tracing::debug!("Button activated via keyboard");
                            // Create a default click event for keyboard activation
                            let click_event = ClickEvent::default();
                            callback(&click_event, window, cx);
                        }
                    });
                }
            }
        }

        button
    }
}
