use serde::{Deserialize, Serialize};

/// Schema version for `AiContextSnapshot`. Bump when adding/removing/renaming fields.
pub const AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION: u32 = 3;

/// Options controlling which context sections are captured.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureContextOptions {
    pub include_selected_text: bool,
    pub include_frontmost_app: bool,
    pub include_menu_bar: bool,
    pub include_browser_url: bool,
    pub include_focused_window: bool,
    /// When true, preserve focused-window screenshot bytes as base64 PNG.
    /// Off by default to keep `kit://context` responses lightweight.
    #[serde(default)]
    pub include_screenshot: bool,
}

impl CaptureContextOptions {
    /// Full metadata capture. Keeps screenshots off by default to preserve the
    /// existing lightweight `kit://context` contract.
    pub const fn all() -> Self {
        Self {
            include_selected_text: true,
            include_frontmost_app: true,
            include_menu_bar: true,
            include_browser_url: true,
            include_focused_window: true,
            include_screenshot: false,
        }
    }

    /// Lightweight live probe for composer recommendations.
    /// Includes selected text because recommendation quality depends on knowing
    /// whether a selection exists, but skips menu bar and focused-window capture.
    /// Focused-window is excluded because the current provider captures a screenshot
    /// (PNG-encoding overhead) even though only metadata is stored — too expensive
    /// for the typing path. Re-enable once a metadata-only provider exists.
    pub const fn recommendation() -> Self {
        Self {
            include_selected_text: true,
            include_frontmost_app: true,
            include_menu_bar: false,
            include_browser_url: true,
            include_focused_window: false,
            include_screenshot: false,
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
            include_screenshot: false,
        }
    }

    /// Submit-time Tab AI capture. This is the only built-in profile that
    /// requests pixel data — all others keep screenshots disabled.
    pub const fn tab_ai() -> Self {
        Self {
            include_selected_text: true,
            include_frontmost_app: true,
            include_menu_bar: true,
            include_browser_url: true,
            include_focused_window: true,
            include_screenshot: true,
        }
    }
}

impl Default for CaptureContextOptions {
    fn default() -> Self {
        Self::all()
    }
}

/// Base64-encoded PNG image with dimensions metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Base64PngContext {
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub base64_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
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
    /// Base64 PNG of the focused window, present only when `include_screenshot` is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_window_image: Option<Base64PngContext>,
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
            focused_window_image: None,
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

/// Focused window metadata. Pixel data lives in `Base64PngContext` (v2+).
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
