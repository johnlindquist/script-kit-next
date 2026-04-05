use serde::{Deserialize, Serialize};

/// Schema version for the automation surface handshake.
pub const AUTOMATION_SURFACE_SCHEMA_VERSION: u32 = 1;

/// Machine-readable snapshot of a named automation surface.
///
/// Returned by `getAutomationSurface` so that agentic helpers can
/// resolve focus targets, capture titles, and minimum window sizes
/// from the app itself instead of hardcoding heuristics.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSurfaceSnapshot {
    /// Schema version (currently 1).
    pub schema_version: u32,
    /// Canonical surface name (e.g. `"acp"`, `"main"`).
    pub surface: String,
    /// The `AppView` variant currently active (e.g. `"AcpChatView"`).
    pub view: String,
    /// Whether the main window is visible.
    pub window_visible: bool,
    /// Whether the main window has focus.
    pub window_focused: bool,
    /// Window title substring for `screencapture` targeting.
    pub capture_title: String,
    /// Process owner name substring for Quartz enumeration.
    pub owner_substring: String,
    /// Minimum width (px) to consider a window valid for capture.
    pub min_width: u32,
    /// Minimum height (px) to consider a window valid for capture.
    pub min_height: u32,
}
