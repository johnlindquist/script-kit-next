use gpui::SharedString;

/// The kind of item in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContextPickerItemKind {
    /// A built-in context attachment (seeded from `context_attachment_specs()`).
    BuiltIn(crate::ai::context_contract::ContextAttachmentKind),
    /// A local file attachment.
    File(std::path::PathBuf),
    /// A local folder attachment.
    Folder(std::path::PathBuf),
}

/// A single row in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextPickerItem {
    /// Unique identifier for this row (e.g. `"builtin:selection"`, `"file:/path"`).
    pub(crate) id: SharedString,
    /// Display label (e.g. `"Selection"`, `"chat.rs"`).
    pub(crate) label: SharedString,
    /// Secondary text (e.g. URI, file path, description).
    pub(crate) subtitle: SharedString,
    /// The kind of item — determines how acceptance creates a context part.
    pub(crate) kind: ContextPickerItemKind,
    /// Relevance score used for deterministic ranking (higher = better match).
    /// Ties are broken by insertion order.
    pub(crate) score: u32,
}

/// Mutable state for the inline context picker overlay.
///
/// Created when the user types `@` in the composer; dropped on Escape,
/// Enter (accept), or when the composer loses focus.
#[derive(Debug, Clone)]
pub(in crate::ai::window) struct ContextPickerState {
    /// The raw query string after the `@` trigger (e.g. `"sel"` from `@sel`).
    pub(in crate::ai::window) query: String,
    /// Ranked items matching the current query.
    pub(in crate::ai::window) items: Vec<ContextPickerItem>,
    /// Currently highlighted row index (keyboard navigation).
    pub(in crate::ai::window) selected_index: usize,
}

impl ContextPickerState {
    pub(in crate::ai::window) fn new(query: String, items: Vec<ContextPickerItem>) -> Self {
        Self {
            query,
            items,
            selected_index: 0,
        }
    }
}

/// Section header for grouped picker results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextPickerSection {
    BuiltIn,
    Files,
    Folders,
}

impl ContextPickerSection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::BuiltIn => "Context",
            Self::Files => "Files",
            Self::Folders => "Folders",
        }
    }
}
