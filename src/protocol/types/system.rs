use serde::{Deserialize, Serialize};

use super::ClipboardEntryType;

/// Window bounds for window management (integer-based for system windows)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TargetWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Clipboard history entry data for list responses
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClipboardHistoryEntryData {
    #[serde(rename = "entryId")]
    pub entry_id: String,
    pub content: String,
    #[serde(rename = "contentType")]
    pub content_type: ClipboardEntryType,
    pub timestamp: String,
    pub pinned: bool,
}

/// System window information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SystemWindowInfo {
    #[serde(rename = "windowId")]
    pub window_id: u32,
    pub title: String,
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<TargetWindowBounds>,
    #[serde(rename = "isMinimized", skip_serializing_if = "Option::is_none")]
    pub is_minimized: Option<bool>,
    #[serde(rename = "isActive", skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Display/monitor information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayInfo {
    /// Display ID
    #[serde(rename = "displayId")]
    pub display_id: u32,
    /// Display name (e.g., "Built-in Retina Display")
    pub name: String,
    /// Whether this is the primary display
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    /// Full display bounds (total resolution)
    pub bounds: TargetWindowBounds,
    /// Visible bounds (excluding menu bar and dock)
    #[serde(rename = "visibleBounds")]
    pub visible_bounds: TargetWindowBounds,
    /// Scale factor (e.g., 2.0 for Retina)
    #[serde(rename = "scaleFactor", skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f64>,
}

/// File search result entry
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileSearchResultEntry {
    pub path: String,
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}
