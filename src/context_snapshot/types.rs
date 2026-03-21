use serde::{Deserialize, Serialize};

/// Schema version for `AiContextSnapshot`. Bump when adding/removing/renaming fields.
pub const AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Options controlling which context sections are captured.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureContextOptions {
    pub include_selected_text: bool,
    pub include_frontmost_app: bool,
    pub include_menu_bar: bool,
    pub include_browser_url: bool,
    pub include_focused_window: bool,
}

impl CaptureContextOptions {
    /// Full capture — every provider enabled. Equivalent to `Default`.
    pub const fn all() -> Self {
        Self {
            include_selected_text: true,
            include_frontmost_app: true,
            include_menu_bar: true,
            include_browser_url: true,
            include_focused_window: true,
        }
    }

    /// Lightweight capture — omits selected text and menu bar for lower
    /// token cost and reduced permission surface.
    pub const fn minimal() -> Self {
        Self {
            include_selected_text: false,
            include_frontmost_app: true,
            include_menu_bar: false,
            include_browser_url: true,
            include_focused_window: true,
        }
    }
}

impl Default for CaptureContextOptions {
    fn default() -> Self {
        Self::all()
    }
}

/// Deterministic, schema-versioned snapshot of AI-relevant desktop context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiContextSnapshot {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmost_app: Option<FrontmostAppContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub menu_bar_items: Vec<MenuBarItemSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<BrowserContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_window: Option<FocusedWindowContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl Default for AiContextSnapshot {
    fn default() -> Self {
        Self {
            schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            selected_text: None,
            frontmost_app: None,
            menu_bar_items: Vec::new(),
            browser: None,
            focused_window: None,
            warnings: Vec::new(),
        }
    }
}

/// Frontmost application metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FrontmostAppContext {
    pub pid: i32,
    pub bundle_id: String,
    pub name: String,
}

/// Browser tab context (URL only in v1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BrowserContext {
    pub url: String,
}

/// Focused window metadata (no pixel data in v1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FocusedWindowContext {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub used_fallback: bool,
}

/// Compact summary of a menu bar item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MenuBarItemSummary {
    pub title: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<MenuBarItemSummary>,
}
