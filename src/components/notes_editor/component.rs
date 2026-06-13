use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, ScrollHandle, Window};
use gpui_component::input::InputState;

use crate::notes::markdown_highlighting::register_markdown_highlighter;

use super::types::{
    NotesEditorConfig, NotesEditorInputSizing, NotesEditorLayout, NotesEditorMarkdownConfig,
};

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
    pub fn new_markdown_pair<T>(
        window: &mut Window,
        cx: &mut Context<T>,
        config: NotesEditorMarkdownConfig,
    ) -> (Entity<InputState>, Entity<Self>)
    where
        T: 'static,
    {
        register_markdown_highlighter();

        let editor_config = config.editor;
        let placeholder = editor_config.placeholder.clone();
        let initial_content = editor_config.initial_content.clone();
        let sizing = config.sizing;
        let input_state = cx.new(|cx| {
            let state = InputState::new(window, cx)
                .code_editor("markdown")
                .code_editor_dynamic_bottom_margin(false)
                .line_number(false)
                .searchable(true)
                .placeholder(placeholder)
                .default_value(initial_content);
            match sizing {
                NotesEditorInputSizing::Rows(rows) => state.rows(rows),
                NotesEditorInputSizing::AutoGrow { min_rows, max_rows } => {
                    state.auto_grow(min_rows, max_rows)
                }
            }
        });
        let notes_editor = cx.new(|_| NotesEditor::new(input_state.clone(), editor_config));
        (input_state, notes_editor)
    }

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

    pub fn load_value_with_cursor_at_end(
        &mut self,
        value: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        let cursor = value.len();
        self.input_state.update(cx, |state, cx| {
            state.set_value(value, window, cx);
            state.set_selection(cursor, cursor, window, cx);
        });
    }

    pub fn set_value_with_cursor_at_end(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_value_with_cursor_at_end(text, window, cx);
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

    pub fn markdown_runtime_info(&self) -> crate::protocol::ElementEditorRuntimeInfo {
        crate::notes::markdown_highlighting::markdown_editor_runtime_info()
    }

    pub fn markdown_runtime_info_with_scroll(
        &self,
        cx: &App,
    ) -> crate::protocol::ElementEditorRuntimeInfo {
        let mut info = self.markdown_runtime_info();
        info.editor_scroll_metrics =
            Some(self.read_input(cx, |state| state.automation_scroll_metrics()));
        info
    }
}
