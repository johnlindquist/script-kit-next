use gpui::{App, Context, Entity, FocusHandle, Focusable, ScrollHandle, Window};
use gpui_component::input::InputState;

use crate::notes::markdown_highlighting::register_markdown_highlighter;

use super::types::{NotesEditorConfig, NotesEditorLayout};

/// Shared markdown editor used by the Notes window and future Day Page surface.
///
/// Owns markdown editing, formatting entry points, code highlighting registration,
/// preview rendering, and editor focus. The host binds document content, wires
/// save/change callbacks, and supplies chrome-specific empty states.
pub struct NotesEditor {
    pub(crate) input_state: Entity<InputState>,
    pub(crate) preview_scroll_handle: ScrollHandle,
    pub(crate) layout: NotesEditorLayout,
}

impl NotesEditor {
    pub fn new(input_state: Entity<InputState>, config: NotesEditorConfig) -> Self {
        register_markdown_highlighter();

        Self {
            input_state,
            preview_scroll_handle: ScrollHandle::new(),
            layout: config.layout,
        }
    }

    pub fn input_state(&self) -> Entity<InputState> {
        self.input_state.clone()
    }

    pub fn preview_scroll_handle(&self) -> &ScrollHandle {
        &self.preview_scroll_handle
    }

    pub fn layout(&self) -> NotesEditorLayout {
        self.layout
    }

    pub fn set_layout(&mut self, layout: NotesEditorLayout) {
        self.layout = layout;
    }

    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input_state.read(cx).focus_handle(cx)
    }

    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.focus_handle(cx).is_focused(window)
    }

    pub fn set_value(
        &mut self,
        value: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        self.input_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
    }

    pub fn set_value_with_cursor_at_end(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.input_state.update(cx, |state, inner_cx| {
            state.set_value(text.clone(), window, inner_cx);
            state.set_selection(text.len(), text.len(), window, inner_cx);
        });
    }

    pub fn set_selection(
        &mut self,
        start: usize,
        end: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.input_state.update(cx, |state, cx| {
            state.set_selection(start, end, window, cx);
        });
    }

    pub fn content(&self, cx: &App) -> String {
        self.input_state.read(cx).value().to_string()
    }

    pub fn selection(&self, cx: &App) -> std::ops::Range<usize> {
        self.input_state.read(cx).selection()
    }

    pub fn soft_wrapped_lines_len(&self, cx: &App) -> usize {
        self.input_state.read(cx).soft_wrapped_lines_len()
    }

    pub fn has_inline_completion(&self, cx: &App) -> bool {
        self.input_state.read(cx).has_inline_completion()
    }

    pub fn sync_inline_completion(&mut self, suffix: Option<String>, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| match suffix {
            Some(suffix) => state.set_inline_completion_text(suffix, cx),
            None => {
                if state.has_inline_completion() {
                    state.clear_inline_completion(cx);
                }
            }
        });
    }

    pub fn read_input<F, R>(&self, cx: &App, f: F) -> R
    where
        F: FnOnce(&InputState) -> R,
    {
        f(self.input_state.read(cx))
    }
}
