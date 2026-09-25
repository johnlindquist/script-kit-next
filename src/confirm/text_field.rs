//! Optional single-line text field for the shared confirm modal.
//!
//! A modal that asks for a short value (an alias, a name) is still the shared
//! confirm popup: same glass, placement, motion, buttons, and keyboard owner.
//! The caller keeps a handle to the field, so it can read the submitted text
//! and automation can observe or set it without reaching into the popup.

use std::{cell::RefCell, rc::Rc};

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, SharedString};

use crate::components::text_input::{
    placeholder_cursor_anchor, render_text_input_cursor_selection, TextInputRenderConfig,
    TextInputState,
};
use crate::theme::AppChromeColors;

/// Height of the field inside the compact confirm modal.
pub(crate) const CONFIRM_TEXT_FIELD_HEIGHT: f32 = 32.0;
const CONFIRM_TEXT_FIELD_PADDING_X: f32 = 10.0;
const CONFIRM_TEXT_FIELD_RADIUS: f32 = 6.0;
const CONFIRM_TEXT_FIELD_FONT_SIZE: f32 = 13.0;
const CONFIRM_TEXT_FIELD_CURSOR_HEIGHT: f32 = 16.0;

/// Explains why the current text cannot be submitted, or `None` when it can.
pub(crate) type ConfirmTextValidator = Rc<dyn Fn(&str) -> Option<SharedString>>;

pub(crate) struct ConfirmTextField {
    pub(crate) state: TextInputState,
    placeholder: SharedString,
    validate: ConfirmTextValidator,
}

/// The field shared between the popup (which edits it) and its caller.
pub(crate) type SharedConfirmTextField = Rc<RefCell<ConfirmTextField>>;

impl ConfirmTextField {
    #[allow(
        dead_code,
        reason = "the application binary's alias editor creates fields"
    )]
    pub(crate) fn shared(
        text: impl Into<String>,
        placeholder: impl Into<SharedString>,
        validate: ConfirmTextValidator,
    ) -> SharedConfirmTextField {
        let mut state = TextInputState::with_text(text);
        state.select_all();
        Rc::new(RefCell::new(Self {
            state,
            placeholder: placeholder.into(),
            validate,
        }))
    }

    pub(crate) fn text(&self) -> &str {
        self.state.text()
    }

    /// Why the text cannot be submitted yet. Empty text reports its problem
    /// too, so the primary action stays disabled until there is a value.
    pub(crate) fn problem(&self) -> Option<SharedString> {
        (self.validate)(self.state.text())
    }
}

impl std::fmt::Debug for ConfirmTextField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmTextField")
            .field("text_bytes", &self.state.text().len())
            .field("placeholder", &self.placeholder)
            .finish_non_exhaustive()
    }
}

/// The field always shows the caret: it owns typing for as long as the modal
/// is open, while Tab moves between the buttons below it.
pub(crate) fn render_confirm_text_field(
    field: &ConfirmTextField,
    chrome: &AppChromeColors,
) -> AnyElement {
    let text = field.state.text();
    let content = if text.is_empty() {
        div()
            .flex()
            .flex_row()
            .items_center()
            .child(placeholder_cursor_anchor(
                crate::panel::CURSOR_WIDTH,
                CONFIRM_TEXT_FIELD_CURSOR_HEIGHT,
                chrome.accent_hex,
                true,
                "confirm-text-field-caret",
            ))
            .child(
                div()
                    .text_color(rgba(chrome.placeholder_text_rgba))
                    .child(field.placeholder.clone()),
            )
    } else {
        render_text_input_cursor_selection(TextInputRenderConfig {
            cursor: field.state.cursor(),
            selection: Some(field.state.selection()),
            cursor_height: CONFIRM_TEXT_FIELD_CURSOR_HEIGHT,
            cursor_color: chrome.accent_hex,
            text_color: chrome.text_primary_hex,
            selection_color: chrome.accent_hex,
            selection_text_color: chrome.text_primary_hex,
            overflow_x_hidden: true,
            ..TextInputRenderConfig::default_for_prompt(text)
        })
    };
    div()
        .id("confirm-text-field")
        .debug_selector(|| "confirm-text-field".to_string())
        .w_full()
        .h(px(CONFIRM_TEXT_FIELD_HEIGHT))
        .flex_none()
        .px(px(CONFIRM_TEXT_FIELD_PADDING_X))
        .flex()
        .items_center()
        .overflow_hidden()
        .rounded(px(CONFIRM_TEXT_FIELD_RADIUS))
        .bg(rgba(chrome.input_surface_rgba))
        .border_1()
        .border_color(rgb(chrome.accent_hex))
        .text_size(px(CONFIRM_TEXT_FIELD_FONT_SIZE))
        .child(content)
        .into_any_element()
}
