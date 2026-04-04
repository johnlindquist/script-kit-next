//! TextInput - Single-line text input with selection and clipboard support
//!
//! A reusable component for text input fields that supports:
//! - Text selection (shift+arrows, cmd+a, mouse drag)
//! - Clipboard operations (cmd+c, cmd+v, cmd+x)
//! - Word navigation (alt+arrows)
//! - Standard cursor movement (arrows, home/end)
//!

#[path = "text_input/core.rs"]
mod core;
#[path = "text_input/render.rs"]
mod render;
#[cfg(test)]
#[path = "text_input/tests.rs"]
mod tests;

pub use core::{TextInputState, TextSelection};
#[allow(unused_imports)]
pub(crate) use render::{
    render_text_input_cursor_selection, TextHighlightRange, TextInputRenderConfig,
    TextInputRenderIndicator,
};
