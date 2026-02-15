//! FooterButton - Reusable footer button component
//!
//! This component renders a label + shortcut pair with consistent footer styling.

#![allow(dead_code)]

use gpui::*;
use std::rc::Rc;

use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

/// Callback type for footer button click events
pub type FooterButtonClickCallback = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A reusable footer button component
#[derive(IntoElement)]
pub struct FooterButton {
    label: SharedString,
    shortcut: Option<SharedString>,
    id: Option<SharedString>,
    disabled: bool,
    loading: bool,
    loading_label: Option<SharedString>,
    on_click: Option<Rc<FooterButtonClickCallback>>,
}

impl FooterButton {
    /// Create a new footer button with a label
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            id: None,
            disabled: false,
            loading: false,
            loading_label: None,
            on_click: None,
        }
    }

    /// Set the shortcut text (e.g., "Enter", "Cmd+K")
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
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

    /// Set whether the button is loading
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set optional loading label text
    pub fn loading_label(mut self, loading_label: impl Into<SharedString>) -> Self {
        self.loading_label = Some(loading_label.into());
        self
    }

    /// Set the click handler for the button
    pub fn on_click(mut self, callback: FooterButtonClickCallback) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }

    /// Hover background color (accent @ 15% alpha)
    pub fn hover_bg(accent: u32) -> u32 {
        (accent << 8) | 0x26
    }

    fn resolve_element_id(id: Option<&SharedString>, label: &SharedString) -> SharedString {
        id.cloned().unwrap_or_else(|| label.clone())
    }

    fn is_clickable(has_click_handler: bool, disabled: bool, loading: bool) -> bool {
        has_click_handler && !disabled && !loading
    }

    fn is_activation_key(key: &str) -> bool {
        matches!(
            key,
            "enter" | "return" | "Enter" | "Return" | " " | "space" | "Space"
        )
    }

    fn can_activate_from_key(
        key: &str,
        has_click_handler: bool,
        disabled: bool,
        loading: bool,
    ) -> bool {
        Self::is_clickable(has_click_handler, disabled, loading) && Self::is_activation_key(key)
    }
}

impl RenderOnce for FooterButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let FooterButton {
            label,
            shortcut,
            id,
            disabled,
            loading,
            loading_label,
            on_click,
        } = self;
        let theme = get_cached_theme();
        let accent = theme.colors.accent.selected;
        let text_muted = theme.colors.text.muted;
        let hover_bg = Self::hover_bg(accent);
        let ui_font_size = theme.get_fonts().ui_size;
        let button_font_size = (ui_font_size - 2.0).max(10.0);
        let has_click_handler = on_click.is_some();
        let is_clickable = Self::is_clickable(has_click_handler, disabled, loading);
        let on_click_for_key = on_click.clone();

        let element_id = Self::resolve_element_id(id.as_ref(), &label);
        let label_text = if loading {
            loading_label.unwrap_or_else(|| label.clone())
        } else {
            label.clone()
        };

        // GPUI cursor styles don't inherit to children, so child elements
        // must also set cursor_pointer when the button is interactive.
        let mut label_element = div()
            .text_size(px(button_font_size))
            .text_color(accent.to_rgb())
            .child(label_text);
        if is_clickable {
            label_element = label_element.cursor_pointer();
        }

        let mut button = div()
            .id(ElementId::Name(element_id))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(2.))
            .rounded(px(4.))
            .cursor_default()
            .child(label_element);

        if is_clickable {
            button = button.cursor_pointer().hover(move |s| s.bg(rgba(hover_bg)));
        } else if disabled {
            button = button.opacity(0.5).cursor_default();
        } else if loading {
            button = button.opacity(0.7).cursor_default();
        }

        if loading {
            button = button.child(
                div()
                    .text_size(px(button_font_size - 1.0))
                    .text_color(text_muted.to_rgb())
                    .child("…"),
            );
        }

        // Add shortcut if provided
        if let Some(shortcut) = shortcut {
            let mut shortcut_element = div()
                .text_size(px(button_font_size))
                .text_color(text_muted.to_rgb())
                .child(shortcut);
            if is_clickable {
                shortcut_element = shortcut_element.cursor_pointer();
            }
            button = button.child(shortcut_element);
        }

        if is_clickable {
            if let Some(callback) = on_click {
                button = button.on_click(move |event, window, cx| {
                    callback(event, window, cx);
                });
            }

            if let Some(callback) = on_click_for_key {
                button = button.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if FooterButton::can_activate_from_key(key, true, disabled, loading) {
                        let click_event = ClickEvent::default();
                        callback(&click_event, window, cx);
                    }
                });
            }
        }

        button
    }
}

#[cfg(test)]
mod tests {
    use super::FooterButton;
    use gpui::SharedString;

    #[test]
    fn test_is_clickable_requires_handler_and_enabled_not_loading() {
        assert!(FooterButton::is_clickable(true, false, false));
        assert!(!FooterButton::is_clickable(false, false, false));
        assert!(!FooterButton::is_clickable(true, true, false));
        assert!(!FooterButton::is_clickable(true, false, true));
    }

    #[test]
    fn test_resolve_element_id_prefers_explicit_id_when_present() {
        let label: SharedString = "Continue".into();
        let explicit_id: SharedString = "footer-continue".into();

        assert_eq!(
            FooterButton::resolve_element_id(Some(&explicit_id), &label),
            explicit_id
        );
        assert_eq!(FooterButton::resolve_element_id(None, &label), label);
    }

    #[test]
    fn test_can_activate_from_key_requires_activation_key_and_interactive_state() {
        assert!(FooterButton::can_activate_from_key(
            "enter", true, false, false
        ));
        assert!(FooterButton::can_activate_from_key(
            "Enter", true, false, false
        ));
        assert!(FooterButton::can_activate_from_key(
            "space", true, false, false
        ));
        assert!(FooterButton::can_activate_from_key(" ", true, false, false));
        assert!(!FooterButton::can_activate_from_key(
            "tab", true, false, false
        ));
        assert!(!FooterButton::can_activate_from_key(
            "enter", false, false, false
        ));
        assert!(!FooterButton::can_activate_from_key(
            "enter", true, true, false
        ));
        assert!(!FooterButton::can_activate_from_key(
            "enter", true, false, true
        ));
    }
}
