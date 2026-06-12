//! Host-facing configuration for the shared markdown notes editor.

/// Layout padding for the editor body and preview surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotesEditorLayout {
    pub padding_x: f32,
    pub padding_y: f32,
}

impl NotesEditorLayout {
    pub const fn new(padding_x: f32, padding_y: f32) -> Self {
        Self {
            padding_x,
            padding_y,
        }
    }
}

/// Initial configuration when constructing a [`super::NotesEditor`].
#[derive(Debug, Clone)]
pub struct NotesEditorConfig {
    pub placeholder: String,
    pub initial_content: String,
    pub layout: NotesEditorLayout,
}

impl NotesEditorConfig {
    pub fn new(initial_content: impl Into<String>) -> Self {
        Self {
            placeholder: "Start typing your note...".to_string(),
            initial_content: initial_content.into(),
            layout: NotesEditorLayout::new(16.0, 12.0),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn layout(mut self, layout: NotesEditorLayout) -> Self {
        self.layout = layout;
        self
    }
}
