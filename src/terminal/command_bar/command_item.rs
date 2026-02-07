use super::TerminalAction;

/// A single command item in the terminal command bar.
///
/// Follows the pattern of `Action` in `src/actions/types.rs` with
/// cached lowercase fields for efficient filtering during search.
#[derive(Debug, Clone)]
pub struct TerminalCommandItem {
    /// Display name shown in the command bar list.
    pub name: String,

    /// Description shown below the name (subtitle).
    pub description: String,

    /// Keyboard shortcut hint (e.g., "⌘K", "⌃C").
    /// Displayed as keycap badges on the right side.
    pub shortcut: Option<String>,

    /// The action to execute when this command is selected.
    pub action: TerminalAction,

    /// Cached lowercase name for filtering.
    pub name_lower: String,

    /// Cached lowercase description for filtering.
    pub description_lower: String,
}

impl TerminalCommandItem {
    /// Create a new terminal command item.
    ///
    /// Automatically computes lowercase versions of name and description
    /// for efficient filtering during search operations.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        action: TerminalAction,
    ) -> Self {
        let name = name.into();
        let description = description.into();
        Self {
            name_lower: name.to_lowercase(),
            description_lower: description.to_lowercase(),
            name,
            description,
            shortcut: shortcut.map(|s| s.into()),
            action,
        }
    }

    /// Returns the action ID for this command.
    pub fn action_id(&self) -> &str {
        self.action.id()
    }

    /// Checks if this command matches the given search query.
    ///
    /// Matches against both name and description (case-insensitive).
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name_lower.contains(&query_lower) || self.description_lower.contains(&query_lower)
    }
}
