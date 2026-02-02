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
    on_click: Option<Rc<FooterButtonClickCallback>>,
}

impl FooterButton {
    /// Create a new footer button with a label
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            id: None,
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

    /// Set the click handler for the button
    pub fn on_click(mut self, callback: FooterButtonClickCallback) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }

    /// Hover background color (accent @ 15% alpha)
    pub fn hover_bg(accent: u32) -> u32 {
        (accent << 8) | 0x26
    }
}

impl RenderOnce for FooterButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = get_cached_theme();
        let accent = theme.colors.accent.selected;
        let text_muted = theme.colors.text.muted;
        let hover_bg = Self::hover_bg(accent);

        let element_id = self.id.clone().unwrap_or_else(|| self.label.clone());

        let mut button = div()
            .id(ElementId::Name(element_id))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(8.))
            .py(px(2.))
            .rounded(px(4.))
            .cursor_pointer()
            .hover(move |s| s.bg(rgba(hover_bg)))
            .child(
                div()
                    .text_sm()
                    .text_color(accent.to_rgb())
                    .child(self.label),
            );

        // Add shortcut if provided
        if let Some(shortcut) = self.shortcut {
            button = button.child(
                div()
                    .text_sm()
                    .text_color(text_muted.to_rgb())
                    .child(shortcut),
            );
        }

        if let Some(callback) = self.on_click {
            button = button.on_click(move |event, window, cx| {
                callback(event, window, cx);
            });
        }

        button
    }
}

// Note: Tests live in tests/footer_button.rs to avoid GPUI macro recursion limits.
