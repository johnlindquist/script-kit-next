use serde::{Deserialize, Serialize};

// ============================================================
// MENU BAR INTEGRATION
// ============================================================

/// A menu bar item with its children and metadata
///
/// Used for serializing menu bar data between the app and SDK.
/// Represents a single menu item in the application's menu bar hierarchy.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuBarItemData {
    /// The display title of the menu item
    pub title: String,
    /// Whether the menu item is enabled (clickable)
    pub enabled: bool,
    /// Keyboard shortcut string if any (e.g., "⌘S")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    /// Child menu items (for submenus)
    #[serde(default)]
    pub children: Vec<MenuBarItemData>,
    /// Path of menu titles to reach this item (e.g., ["File", "New", "Window"])
    #[serde(default)]
    pub menu_path: Vec<String>,
}

impl MenuBarItemData {
    /// Create a new MenuBarItemData
    pub fn new(title: String, enabled: bool) -> Self {
        MenuBarItemData {
            title,
            enabled,
            shortcut: None,
            children: Vec::new(),
            menu_path: Vec::new(),
        }
    }

    /// Add a keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: String) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Add child menu items
    pub fn with_children(mut self, children: Vec<MenuBarItemData>) -> Self {
        self.children = children;
        self
    }

    /// Set the menu path
    pub fn with_menu_path(mut self, path: Vec<String>) -> Self {
        self.menu_path = path;
        self
    }
}
