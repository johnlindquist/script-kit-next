use gpui::SharedString;

/// The kind of item in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextPickerItemKind {
    /// A built-in context attachment (seeded from `context_attachment_specs()`).
    BuiltIn(crate::ai::context_contract::ContextAttachmentKind),
    /// A local file attachment.
    File(std::path::PathBuf),
    /// A local folder attachment.
    Folder(std::path::PathBuf),
}

/// A single row in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPickerItem {
    /// Unique identifier for this row (e.g. `"builtin:selection"`, `"file:/path"`).
    pub id: SharedString,
    /// Display label (e.g. `"Selection"`, `"chat.rs"`).
    pub label: SharedString,
    /// Secondary text (e.g. URI, file path, description).
    pub subtitle: SharedString,
    /// The kind of item — determines how acceptance creates a context part.
    pub kind: ContextPickerItemKind,
    /// Relevance score used for deterministic ranking (higher = better match).
    /// Ties are broken by insertion order.
    pub score: u32,
}

/// Mutable state for the inline context picker overlay.
///
/// Created when the user types `@` in the composer; dropped on Escape,
/// Enter (accept), or when the composer loses focus.
#[derive(Debug, Clone)]
pub struct ContextPickerState {
    /// The raw query string after the `@` trigger (e.g. `"sel"` from `@sel`).
    pub query: String,
    /// Ranked items matching the current query.
    pub items: Vec<ContextPickerItem>,
    /// Currently highlighted row index (keyboard navigation).
    pub selected_index: usize,
}

impl ContextPickerState {
    pub fn new(query: String, items: Vec<ContextPickerItem>) -> Self {
        Self {
            query,
            items,
            selected_index: 0,
        }
    }
}

/// Section header for grouped picker results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPickerSection {
    BuiltIn,
    Files,
    Folders,
}

impl ContextPickerSection {
    pub fn label(self) -> &'static str {
        match self {
            Self::BuiltIn => "Context",
            Self::Files => "Files",
            Self::Folders => "Folders",
        }
    }
}
