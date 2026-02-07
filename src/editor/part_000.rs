use gpui::{
    div, prelude::*, px, rgb, Context, Entity, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Styled, Subscription, Window,
};
use gpui_component::input::{IndentInline, Input, InputEvent, InputState, OutdentInline, Position};
use std::sync::Arc;
use crate::config::Config;
use crate::logging;
use crate::snippet::ParsedSnippet;
use crate::theme::Theme;
/// Convert a character offset to a byte offset.
///
/// CRITICAL: When char_offset equals or exceeds the character count of the text,
/// this returns text.len() (the byte length), NOT 0. This is essential for
/// correct cursor positioning at end-of-document (e.g., $0 tabstops).
///
/// # Arguments
/// * `text` - The string to convert offsets in
/// * `char_offset` - Character index (0-based)
///
/// # Returns
/// The byte offset corresponding to the character offset, or text.len() if
/// the char_offset is at or beyond the end of the string.
fn char_offset_to_byte_offset(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map(|(i, _)| i)
        .unwrap_or(text.len()) // CRITICAL: Use text.len(), not 0!
}
/// Convert a character offset to a Position (line, column)
///
/// This is needed because gpui-component's InputState uses Position (line, column)
/// for cursor placement, but our snippet parser tracks char offsets.
#[allow(dead_code)]
fn char_offset_to_position(text: &str, char_offset: usize) -> Position {
    let mut line: u32 = 0;
    let mut col: u32 = 0;

    for (current_char, ch) in text.chars().enumerate() {
        if current_char >= char_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position {
        line,
        character: col,
    }
}
/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;
/// Pending initialization state - stored until first render when window is available
struct PendingInit {
    content: String,
    language: String,
}
/// State for template/snippet navigation
///
/// Tracks the current position within a template's tabstops, allowing
/// Tab/Shift+Tab navigation through the placeholders.
#[derive(Debug, Clone)]
pub struct SnippetState {
    /// The parsed snippet with tabstop information
    pub snippet: ParsedSnippet,
    /// Current index into snippet.tabstops (0-based position in navigation order)
    pub current_tabstop_idx: usize,
    /// Current placeholder values (updated when user edits a tabstop)
    /// Index matches snippet.tabstops order
    pub current_values: Vec<String>,
    /// Tracks the last known selection range (char offsets) for each tabstop
    /// Used to detect when user has edited a tabstop and update current_values
    pub last_selection_ranges: Vec<Option<(usize, usize)>>,
}
/// State for the choice dropdown popup
/// Shown when a tabstop has multiple choices (${1|opt1,opt2,opt3|})
#[derive(Debug, Clone)]
pub struct ChoicesPopupState {
    /// The list of choices to display
    pub choices: Vec<String>,
    /// Currently highlighted index in the list
    pub selected_index: usize,
    /// The tabstop index this popup is for
    pub tabstop_idx: usize,
}
/// EditorPrompt - Full-featured code editor using gpui-component
///
/// Uses deferred initialization pattern: the InputState is created on first render
/// when the Window reference is available, not at construction time.
pub struct EditorPrompt {
    // Identity
    pub id: String,

    // gpui-component editor state (created on first render)
    editor_state: Option<Entity<InputState>>,

    // Pending initialization data (consumed on first render)
    pending_init: Option<PendingInit>,

    // Template/snippet state for tabstop navigation
    snippet_state: Option<SnippetState>,

    // Language for syntax highlighting (displayed in footer)
    language: String,

    // GPUI
    focus_handle: FocusHandle,
    on_submit: SubmitCallback,
    theme: Arc<Theme>,
    #[allow(dead_code)]
    config: Arc<Config>,

    // Layout - explicit height for proper sizing
    content_height: Option<gpui::Pixels>,

    // Subscriptions to keep alive
    #[allow(dead_code)]
    subscriptions: Vec<Subscription>,

    // When true, ignore all key events (used when actions panel is open)
    pub suppress_keys: bool,

    // Flag to request focus on next render (used for auto-focus after initialization)
    needs_focus: bool,

    // Flag to indicate we need to select the first tabstop after initialization
    needs_initial_tabstop_selection: bool,

    // Choice dropdown popup state (shown when tabstop has choices)
    choices_popup: Option<ChoicesPopupState>,
}
