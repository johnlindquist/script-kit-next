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

/// Schema version for the launcher surface contract snapshot in `getState`.
pub const LAUNCHER_SURFACE_CONTRACT_SCHEMA_VERSION: u32 = 1;

/// Machine-readable projection of the active launcher surface contract.
///
/// Included in main-window `stateResult` receipts so agents can verify the
/// runtime surface against the generated contract matrix without reverse-
/// engineering `promptType` strings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSurfaceContractSnapshot {
    pub schema_version: u32,
    pub surface_kind: String,
    pub family: String,
    pub input_ownership: String,
    pub preview_role: String,
    pub focus_policy: String,
    pub keyboard_policy: String,
    pub actions_policy: String,
    pub proof_policy: String,
    pub visual_policy: String,
    pub automation_semantic_surface: String,
    pub native_footer_surface: Option<String>,
}
