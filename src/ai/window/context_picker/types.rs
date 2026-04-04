use gpui::SharedString;

/// Whether the picker was triggered by `@` (mention) or `/` (slash command).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextPickerTrigger {
    Mention,
    Slash,
}

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
    /// Right-side metadata (slash command, mention, or path).
    pub meta: SharedString,
    /// The kind of item — determines how acceptance creates a context part.
    pub kind: ContextPickerItemKind,
    /// Relevance score used for deterministic ranking (higher = better match).
    /// Ties are broken by insertion order.
    pub score: u32,
    /// Indices into `label` that matched the query (for gold highlighting).
    pub label_highlight_indices: Vec<usize>,
    /// Indices into `meta` that matched the query (for gold highlighting).
    pub meta_highlight_indices: Vec<usize>,
}

/// Mutable state for the inline context picker overlay.
///
/// Created when the user types `@` or `/` in the composer; dropped on Escape,
/// Enter (accept), or when the composer loses focus.
#[derive(Debug, Clone)]
pub struct ContextPickerState {
    /// Which trigger character opened this picker.
    pub trigger: ContextPickerTrigger,
    /// The raw query string after the trigger (e.g. `"sel"` from `@sel`).
    pub query: String,
    /// Ranked items matching the current query.
    pub items: Vec<ContextPickerItem>,
    /// Currently highlighted row index (keyboard navigation).
    pub selected_index: usize,
}

impl ContextPickerState {
    pub fn new(trigger: ContextPickerTrigger, query: String, items: Vec<ContextPickerItem>) -> Self {
        Self {
            trigger,
            query,
            items,
            selected_index: 0,
        }
    }

    /// Machine-readable snapshot of picker entries and selection state.
    /// Used by agents to verify UI state without brittle string scraping.
    pub fn snapshot(&self) -> ContextPickerSnapshot {
        ContextPickerSnapshot {
            trigger: self.trigger,
            query: self.query.clone(),
            selected_index: self.selected_index,
            items: self
                .items
                .iter()
                .map(|item| ContextPickerItemSnapshot {
                    id: item.id.to_string(),
                    label: item.label.to_string(),
                    section: match &item.kind {
                        ContextPickerItemKind::BuiltIn(_) => "builtin",
                        ContextPickerItemKind::File(_) => "file",
                        ContextPickerItemKind::Folder(_) => "folder",
                    },
                    score: item.score,
                })
                .collect(),
        }
    }
}

/// Serializable snapshot of picker state for agent verification.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextPickerSnapshot {
    pub trigger: ContextPickerTrigger,
    pub query: String,
    pub selected_index: usize,
    pub items: Vec<ContextPickerItemSnapshot>,
}

/// Serializable snapshot of a single picker item.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextPickerItemSnapshot {
    pub id: String,
    pub label: String,
    pub section: &'static str,
    pub score: u32,
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
